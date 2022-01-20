use convert_case::{Case, Casing};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive TableName trait for model struct. Table name is based on the struct
/// name converted to snake_case
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

/// Derive TryFromRow trait for model struct. try_from_row method builds Self struct
/// based on rusqlite::Row::get(&str: field_name)
#[proc_macro_derive(TryFromRow)]
pub fn derive_from_row(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let fields = match data {
        syn::Data::Struct(fields) => fields.fields,
        syn::Data::Enum(_) => panic!("expecting struct, found enum"),
        syn::Data::Union(_) => panic!("expecting struct, found union"),
    };

    let mut fields_tokens = quote!();

    match fields {
        syn::Fields::Named(named) => {
            for field in named.named.into_iter() {
                let field_name = field.ident.unwrap();

                fields_tokens.extend(quote! {
                    #field_name: row.get(stringify!(#field_name))?,
                });
            }
        }
        _ => panic!("expecting field of type named"),
    };

    let output = quote! {
        impl crate::traits::TryFromRow for #ident {
            fn try_from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                Ok(Self { #fields_tokens })
            }
        }
    };

    output.into()
}
