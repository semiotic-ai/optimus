
use std::{
    borrow::Cow,
    env, fs,
    io::{Write, read_to_string},
    path::{Path, PathBuf},
};
use anyhow::{Ok, Result,format_err};
use build_mod::CodeGeneration;
use substreams_ethereum::{AbiExtension, Abigen, EventExtension};
use ethabi::{Contract, Error, ParamType};
use serde::Deserialize;
use serde_json::from_reader;


const CODE_PATH:&str = "src/abi";
const OUTPUT_PATH:&str = "output";

const SQL_TABLE_END:&[u8; 330] = 
b"\t`evt_block_number` UInt64,
\t`evt_tx_hash` FixedString(64),
\t`evt_index` UInt32,
\t`evt_block_time` DateTime,
\t`tx_to` FixedString(40),
\t`tx_from` FixedString(40),
)
ENGINE = MergeTree
PRIMARY KEY (evt_block_time,
\tevt_block_number,
\tevt_tx_hash,
\tevt_index)
ORDER BY (evt_block_time,
\tevt_block_number,
\tevt_tx_hash,
\tevt_index);
";

#[derive(Deserialize)]
struct SubstreamContract {
    name:String,
    abi_file:String,
    table_prefix:Option<String>,
    address:Option<String>,
}

#[derive(Deserialize)]
struct SubstreamConfig {
    name:String,
    version:String,
    network:String,
    database:String,
    contracts:Vec<SubstreamContract>,
}

fn normalize_path<S: AsRef<Path>>(relative_path: S) -> Result<PathBuf, anyhow::Error> {
    // workaround for https://github.com/rust-lang/rust/issues/43860
    let cargo_toml_directory =
        env::var("CARGO_MANIFEST_DIR").map_err(|_| format_err!("Cannot find manifest file"))?;
    let mut path: PathBuf = cargo_toml_directory.into();
    path.push(relative_path);
    Ok(path)
}

fn write_param_type_sql(param_type: &ParamType,writer:&mut impl Write) -> Result<(), anyhow::Error> {
    match param_type {
        ParamType::String  | ParamType::Bytes  => write!(writer,"String")?,
        ParamType::Address => write!(writer,"FixedString(40)")?,
        ParamType::FixedBytes(size) => write!(writer,"FixedString({})", size)?,
        ParamType::Int(size) => write!(writer,"Int{}", size)?,
        ParamType::Uint(size) => write!(writer, "UInt{}", size)?,
        ParamType::Bool => write!(writer,"Boolean")?,
        ParamType::Array(item_type)  | ParamType::FixedArray(item_type, _) => {
            write!(writer,"Array(")?;
            write_param_type_sql(item_type.as_ref(),writer)?;
            write!(writer,")")?;
        },
        ParamType::Tuple(item_types) =>  {
            write!(writer,"Tuple(")?;
            for (index,item_type) in item_types.iter().enumerate() {
                write_param_type_sql(item_type,writer)?;
                if index < item_types.len() - 1 {
                    write!(writer,",")?;
                }
            }
            write!(writer,")")?
        }
    };
    Ok(())
}

fn write_contract_sql(
    contract: Contract,
    database_name: &str,
    table_prefix:&Option<String>,
    writer: &mut impl Write,
) -> Result<()> {

    for (_,events) in contract.events {
        for event in events {

            let table_name = match table_prefix {
                Some(prefix) => format!("{}{}",prefix,event.name.to_lowercase()),
                None => event.name.to_lowercase()
            };

            write!(writer,"\nCREATE TABLE IF NOT EXISTS {}.{} (\n", database_name, table_name)?;
            for param in event.inputs {
                write!(writer,"\t`{}` ", param.name)?;
                write_param_type_sql(&param.kind,writer)?;
                write!(writer,",\n")?;
            }
            writer.write(SQL_TABLE_END)?;
        }
    }

    Ok(())
}

fn write_database_sql(
    database_name:&str,
    writer: &mut impl Write,
) -> Result<()> {

    write!(writer,"CREATE DATABASE IF NOT EXISTS {};\n", database_name)?;
    write!(writer,"
CREATE TABLE IF NOT EXISTS {}.cursors (
\tid String,
\tcursor String,
\tblock_num Int64,
\tblock_id String
)
ENGINE = ReplacingMergeTree
ORDER BY id;
", database_name)?;

    Ok(())
}

pub fn write_abi_sql<S: AsRef<str>>(
    path: S,
    database_name: &str,
    table_prefix:&Option<String>,
    writer: &mut impl Write,
) -> Result<(), anyhow::Error> {
    let normalized_path = normalize_path(path.as_ref())?;
    let source_file = fs::File::open(&normalized_path).map_err(|_| {
        Error::Other(Cow::Owned(format!(
            "Cannot load contract abi from `{}`",
            normalized_path.display()
        )))
    })?;
    let contract = Contract::load(source_file)?;
    write_contract_sql(contract, database_name, table_prefix, writer)?;
    Ok(())
}

fn main() -> Result<(), anyhow::Error> {

    fs::remove_dir_all(CODE_PATH)?;
    fs::create_dir(CODE_PATH)?;

    fs::remove_dir_all(OUTPUT_PATH)?;
    fs::create_dir(OUTPUT_PATH)?;

    let substream_config:SubstreamConfig = from_reader(fs::File::open("substream_config.json")?)?; 

    generate_code(&substream_config)?;
    generate_sql_schema(&substream_config)?;
    generate_substreams(&substream_config)?;

    Ok(())

}

fn generate_code(config: &SubstreamConfig) -> Result<()> {

    for contract in &config.contracts {
        generate_for(contract)?;
    }

    let abi_files = config.contracts.iter().map(|contract| contract.name.clone()).collect();

    CodeGeneration::new(abi_files)
        .generate_code()?
        .write_to_file(format!("{}/mod.rs",CODE_PATH))?;
    Ok(())
}

fn generate_for(contract: &SubstreamContract) -> Result<()> {
    let abigen = Abigen::new(&contract.name,contract.address.clone(), &contract.abi_file)?;

    let mut event_extension = EventExtension::new();
    event_extension.extend_event_derive("to_table_derive::ToTableChange");
    if let Some(prefix) = &contract.table_prefix {
        event_extension.extend_event_attribute(format!("table_prefix=\"{}\"",prefix).as_str());
    }
    let extension = AbiExtension::new(event_extension);
    abigen
        .add_extension(extension)
        .generate()?
        .write_to_file(format!("src/abi/{}.rs", contract.name))?;
    Ok(())
}

fn generate_sql_schema(config: &SubstreamConfig) -> Result<()> {

    let mut writer = fs::File::create(format!("{}/schema.sql",OUTPUT_PATH))?;

    write_database_sql(&config.database, &mut writer)?;

    for contract in &config.contracts {
        write_abi_sql(&contract.abi_file, &config.database, &contract.table_prefix, &mut writer)?;
    }

    Ok(())
}


fn generate_substreams(config: &SubstreamConfig) -> Result<()> {

    let mut template = read_to_string(fs::File::open("configs/substreams_template.yaml")?)?;

    template = template.replace("{{NAME}}",&config.name);
    template = template.replace("{{VERSION}}",&config.version);
    template = template.replace("{{NETWORK}}",&config.network);

    fs::File::create(format!("{}/substreams.yaml",OUTPUT_PATH))?.write_all(template.as_bytes())?;

    Ok(())
}

