use anyhow::format_err;
use anyhow::Context;
use heck::ToUpperCamelCase;
use quote::quote;
use std::{
    env,
    path::{Path, PathBuf},
};

pub struct CodeGeneration {
    abi_files: Vec<String>,
}

impl CodeGeneration {
    pub fn new(abi_files: Vec<String>) -> Self {
        Self { abi_files }
    }

    pub fn generate_code(&self) -> anyhow::Result<GeneratedBindings> {
        let mods: Vec<_> = self
            .abi_files
            .iter()
            .map(|file| syn::Ident::new(file, proc_macro2::Span::call_site()))
            .collect();

        let camel_case: Vec<_> = self
            .abi_files
            .iter()
            .map(|file| 
                 syn::Ident::new(&file.to_upper_camel_case(), proc_macro2::Span::call_site()))
            .collect();

        let if_clauses = camel_case.iter().zip(mods.clone()).map(|(e, m)| {
            quote! {
                if let Some(event) = #m::events::Events::match_and_decode(log) {
                    return Some(Events::#e(event));
                }
            }
        });

        let code = quote! {
            use to_table_derive::ToTableChange;

            #(pub mod #mods;)*

            #[derive(ToTableChange)]
            pub enum Events {
                #(#camel_case(#mods::events::Events),)*
            }

            impl Events {
                pub fn match_and_decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Option<Events> {
                    #( #if_clauses )*
                    return None;
                }
            }
        };

        let file = syn::parse_file(&code.to_string())?;
        let code = prettyplease::unparse(&file);
        Ok(GeneratedBindings {
            code 
        })
    }
}

fn normalize_path<S: AsRef<Path>>(relative_path: S) -> Result<PathBuf, anyhow::Error> {
    // workaround for https://github.com/rust-lang/rust/issues/43860
    let cargo_toml_directory =
        env::var("CARGO_MANIFEST_DIR").map_err(|_| format_err!("Cannot find manifest file"))?;
    let mut path: PathBuf = cargo_toml_directory.into();
    path.push(relative_path);
    Ok(path)
}

pub struct GeneratedBindings {
    code: String,
}

impl GeneratedBindings {
    pub fn write_to_file<P: AsRef<Path>>(&self, p: P) -> Result<(), anyhow::Error> {
        let path = normalize_path(p.as_ref()).context("normalize path")?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating directories for {}", parent.to_string_lossy()))?
        }

        std::fs::write(path, &self.code)
            .with_context(|| format!("writing file {}", p.as_ref().to_string_lossy()))
    }
}
