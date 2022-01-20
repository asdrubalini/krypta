use convert_case::{Casing, Case};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TableName)]
pub fn derive_table_name(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let table_name = format!("{ident}").to_case(Case::Snake);

    let output = quote! {
        impl TableName for #ident {
            fn table_name() -> &'static str {
                #table_name
            }
        }
    };
    
    output.into()
}
