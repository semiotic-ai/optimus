use anyhow::{format_err, Context, Error};
use futures03::StreamExt;
use pb::sf::substreams::rpc::v2::{BlockScopedData, BlockUndoSignal};
use pb::sf::substreams::v1::Package;

use prost::Message;
use std::{process::exit, sync::Arc};
use substreams::SubstreamsEndpoint;
use substreams_stream::{BlockResponse, SubstreamsStream};
use substreams_ethereum::pb::eth;
use clap::{Parser};
use optimus_price::graph_out


mod pb;
mod substreams;
mod substreams_stream;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLI {
    
    /// Substreams endpoint ie mainnet.eth.streamingfast.io:443
    #[arg(env)]
    substreams_endpoint: String,
    
    /// Substreams API key
    #[arg(env)]
    substreams_api_token: String,

    #[arg(short,default_value = "0")]
    start: i64,

    #[arg(short)]
    end: Option<u64>,

    /// Block forwarder substream
    #[arg(short, default_value="./forwarder/substreams.spkg")]
    forwarder: String
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = CLI::parse();
    
    let mut  forwarder = read_package(&cli.forwarder).await?;

    let endpoint = Arc::new(SubstreamsEndpoint::new(cli.substreams_endpoint, Some(cli.substreams_api_token)).await?);

    let cursor: Option<String> = load_persisted_cursor()?;

    let copy = forwarder.modules.map(|mut modules| {
        modules.modules[0].initial_block = cli.start.unsigned_abs();
        modules
    });

    let mut stream = SubstreamsStream::new(
        endpoint.clone(),
        cursor,
        copy,
        "forward".to_string(),
        cli.start,
        cli.end.unwrap_or(cli.start.unsigned_abs() + 10000u64)
    );

    loop {
        match stream.next().await {
            None => {
                println!("Stream consumed");
                break;
            }
            Some(Ok(BlockResponse::New(data))) => {
                process_block_scoped_data(&data)?;
                persist_cursor(data.cursor)?;
            }
            Some(Ok(BlockResponse::Undo(undo_signal))) => {
                process_block_undo_signal(&undo_signal)?;
                persist_cursor(undo_signal.last_valid_cursor)?;
            }
            Some(Err(err)) => {
                println!();
                println!("Stream terminated with error");
                println!("{:?}", err);
                exit(1);
            }
        }
    }

    Ok(())
}

fn process_block_scoped_data(data: &BlockScopedData) -> Result<(), Error> {
    let output = data.output.as_ref().unwrap().map_output.as_ref().unwrap();
    let block = eth::v2::Block::decode(output.value.as_slice())?;

    let result = transform(block)?;
 
    println!(
        "Block #{} - Payload {} ({} changes)",
        data.clock.as_ref().unwrap().number,
        output.type_url.replace("type.googleapis.com/", ""),
        result.table_changes.len()
    );

    Ok(())
}

fn process_block_undo_signal(_undo_signal: &BlockUndoSignal) -> Result<(), anyhow::Error> {
    // `BlockUndoSignal` must be treated as "delete every data that has been recorded after
    // block height specified by block in BlockUndoSignal". In the example above, this means
    // you must delete changes done by `Block #7b` and `Block #6b`. The exact details depends
    // on your own logic. If for example all your added record contain a block number, a
    // simple way is to do `delete all records where block_num > 5` which is the block num
    // received in the `BlockUndoSignal` (this is true for append only records, so when only `INSERT` are allowed).
    unimplemented!("you must implement some kind of block undo handling, or request only final blocks (tweak substreams_stream.rs)")
}

fn persist_cursor(_cursor: String) -> Result<(), anyhow::Error> {
    // FIXME: Handling of the cursor is missing here. It should be saved each time
    // a full block has been correctly processed/persisted. The saving location
    // is your responsibility.
    //
    // By making it persistent, we ensure that if we crash, on startup we are
    // going to read it back from database and start back our SubstreamsStream
    // with it ensuring we are continuously streaming without ever losing a single
    // element.
    Ok(())
}

fn load_persisted_cursor() -> Result<Option<String>, anyhow::Error> {
    // FIXME: Handling of the cursor is missing here. It should be loaded from
    // somewhere (local file, database, cloud storage) and then `SubstreamStream` will
    // be able correctly resume from the right block.
    Ok(None)
}

async fn read_package(input: &str) -> Result<Package, anyhow::Error> {
    let content =
        std::fs::read(input).context(format_err!("read package from file '{}'", input))?;
    Package::decode(content.as_ref()).context("decode command")
}
