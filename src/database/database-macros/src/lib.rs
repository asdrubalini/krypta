use convert_case::{Case, Casing};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TableName)]
pub fn derive_table_name(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let table_name = ident.to_string().to_case(Case::Snake);

    let output = quote! {
        impl crate::traits::TableName for #ident {
            fn table_name() -> &'static str {
                #table_name
            }
        }
    };

    output.into()
}

#[proc_macro_derive(FromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let output = quote! {
        impl crate::traits::FromRow for #ident {
            fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                todo!()
            }
        }
    };

    output.into()
}
