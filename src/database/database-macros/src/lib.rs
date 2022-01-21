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

#[proc_macro_derive(Insert)]
pub fn derive_insert(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let fields = match data {
        syn::Data::Struct(fields) => fields.fields,
        syn::Data::Enum(_) => panic!("expecting struct, found enum"),
        syn::Data::Union(_) => panic!("expecting struct, found union"),
    };

    let mut query_args = String::new();
    let mut query_values = String::new();
    let mut named_params = quote!();

    match fields {
        syn::Fields::Named(named) => {
            for field in named.named.into_iter() {
                let field_name = field.ident.unwrap();

                let field_str = field_name.to_string();

                query_args.push_str(&format!("`{field_str}`,"));
                query_values.push_str(&format!(":{field_str},"));

                let mut arg_name = ":".to_string();
                arg_name.push_str(&field_name.to_string());

                named_params.extend(quote! {
                    (#arg_name, &self.#field_name as &dyn rusqlite::ToSql),
                });
            }
        }
        _ => panic!("expecting field of type named"),
    };

    // Remove trailing comma
    query_args.pop();
    query_values.pop();

    let output = quote! {
        impl crate::traits::Insert for #ident {
            fn insert(self, db: &crate::Database) -> crate::errors::DatabaseResult<Self> {
                use crate::traits::{TableName, TryFromRow};

                let table_name = #ident::table_name();
                let query = format!("INSERT INTO `{}` ({}) VALUES ({}) RETURNING *;", table_name, #query_args, #query_values);

                let item = db.query_row(
                    &query,
                    &[ #named_params ],
                    |row| #ident::try_from_row(row)
                )?;

                Ok(item)
            }
        }
    };

    output.into()
}
