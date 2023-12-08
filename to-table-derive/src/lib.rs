use crate::syn::spanned::Spanned;
use proc_macro2::{Ident, TokenStream};
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use std::unimplemented;

use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ToTableChange,attributes(table_prefix))]
pub fn to_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Construct a string representation of the type definition

    // Parse the string representation
    let input = parse_macro_input!(input as DeriveInput);
      
    // Build the impl
    let expanded = impl_to_table(&input);

    // Return the generated impl
    proc_macro::TokenStream::from(expanded)
}

fn impl_to_table(ast: &syn::DeriveInput) -> TokenStream {

    let name = &ast.ident;

    let table_name = ast.attrs.iter().filter(|attr| attr.path().is_ident("table_prefix")).map(|attr| {
        if let syn::Meta::NameValue(name_value) = &attr.meta {
            if let syn::Expr::Lit(lit)  = &name_value.value {
                if let syn::Lit::Str(lit_str) = &lit.lit {
                    return Some(lit_str.value());
                }
            }
        }
        None
    }).next().flatten();
    
    let mut table_name = table_name.unwrap_or(String::new());
    table_name.push_str(name.to_string().to_lowercase().as_str());

    let changes = add_table_changes_method(name, &ast.data);
    let table_name = add_table_name_method(name, &ast.data,table_name);
    let contract_name = add_contract_name_method(name, &ast.data);

    quote! {
        impl crate::ToTableChange for #name {

            #changes

            #table_name

            #contract_name
            
        }
    }
}

fn add_table_name_method(name: &Ident, data: &Data,table_name:String) -> TokenStream {
    match *data {
        Data::Union(_) | Data::Struct(_) => {
            quote! {
                fn get_table_name(&self) -> &'static str {
                    #table_name
                }
            }
        }
        Data::Enum(ref data) => {
            let recurse = data.variants.iter().map(|f| {
                match (f.fields.len(), f.fields.iter().collect::<Vec<_>>().first()) {
                    (1, Some(_)) => {
                        let ident = &f.ident;
                        quote_spanned! {f.span()=>
                            #name::#ident(e) => e.get_table_name()
                        }
                    }
                    _ => unimplemented!(),
                }
            });
            quote! {
                fn get_table_name(&self) -> &'static str {
                    match self {
                        #(#recurse,)*
                    }
                }
            }
        }
    }
}


fn add_contract_name_method(name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Union(_) | Data::Struct(_) => {
            quote! {
                fn get_contract_name(&self) -> &'static str {
                    super::CONTRACT_NAME
                }
            }
        }
        Data::Enum(ref data) => {
            let recurse = data.variants.iter().map(|f| {
                match (f.fields.len(), f.fields.iter().collect::<Vec<_>>().first()) {
                    (1, Some(_)) => {
                        let ident = &f.ident;
                        quote_spanned! {f.span()=>
                            #name::#ident(e) => e.get_contract_name()
                        }
                    }
                    _ => unimplemented!(),
                }
            });
            quote! {
                fn get_contract_name(&self) -> &'static str {
                    match self {
                        #(#recurse,)*
                    }
                }
            }
        }
    }
}

// Generate an expression to sum up the heap size of each field.
fn add_table_changes_method(name: &Ident, data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let quoted_name = name.as_ref().unwrap().to_string();

                    quote_spanned! {f.span()=>
                        .change(#quoted_name, (None, &self.#name.get_value()))
                    }
                });
                quote! {
                    fn add_table_changes(&self, table_change: &mut substreams_database_change::pb::database::TableChange) {
                        use crate::events::TableField;
                        table_change
                            #(#recurse)*;
                    }
                }
            }
            Fields::Unnamed(_) | Fields::Unit => unimplemented!(),
        },
        Data::Enum(ref data) => {
            let recurse = data.variants.iter().map(|f| {
                match (f.fields.len(), f.fields.iter().collect::<Vec<_>>().first()) {
                    (1, Some(_)) => {
                        let ident = &f.ident;
                        quote_spanned! {f.span()=>
                            #name::#ident(e) => e.add_table_changes(table_change)
                        }
                    }
                    _ => unimplemented!(),
                }
            });
            quote! {
                fn add_table_changes(&self, table_change: &mut substreams_database_change::pb::database::TableChange) {
                    match self {
                        #(#recurse,)*
                    }
                }
            }
        }
        Data::Union(_) => unimplemented!(),
    }
}
