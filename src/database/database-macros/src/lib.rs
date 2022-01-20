use convert_case::{Casing, Case};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TableName)]
pub fn derive_table_name(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let table_name = ident.to_string().to_case(Case::Snake);

    let output = quote! {
        impl TableName for #ident {
            fn table_name() -> &'static str {
                #table_name
            }
        }
    };
    
    output.into()
}

#[proc_macro_derive(TryFromRow)]
pub fn derive_try_from_row(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let output = quote! {
        impl TryFromRow for #ident {
            fn try_from_row(row: &Row) -> Result<Self, rusqlite::Error> {
                Ok(#ident {
                    id: row.get(0)?,
                })
            }
        }
    };
    
    output.into()
}
