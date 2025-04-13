mod decode_fields;

use std::fmt::format;

use convert_case::{Case, Casing};
use decode_fields::decode_field;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{ParseStream, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    ItemStruct, LitStr,
};

#[proc_macro_derive(
    Entity,
    attributes(table_name, column_type, default, primary_key, unique)
)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input: ItemStruct = parse_macro_input!(input as ItemStruct);
    let ref name = input.ident;
    let default_table_name = name.to_string().to_case(Case::Snake);
    let table_name = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.meta.path().is_ident("table_name") {
                let Ok(v) = attr
                    .meta
                    .require_list()
                    .and_then(|v| v.parse_args::<LitStr>())
                else {
                    panic!(
                        "Error while parsing `table_name`, use it like #[table_name(\"{}_table\")]",
                        &default_table_name
                    );
                };
                return Some(v.value());
            }
            None
        })
        .unwrap_or(default_table_name);
    let table_primary_key = input.attrs.iter().find_map(|attr| {
        if attr.meta.path().is_ident("primary_key") {
            let parser = |input: ParseStream| Punctuated::<LitStr, Comma>::parse_terminated(input);
            let Ok(v) = attr.meta.require_list().and_then(|v| {
                Ok(parser
                    .parse2(v.tokens.clone())?
                    .into_iter()
                    .map(|v| v.value())
                    .collect::<Vec<_>>())
            }) else {
                panic!("Error while parsing `primary_key`, use it like #[primary_key(\"first\", \"second\")]");
            };
            return Some(v);
        }
        None
    }).unwrap_or(Default::default());
    let iter = input.fields.iter();
    let columns_defs = iter.clone().map(|f| {
        let mut column_def = decode_field(&f);
        if column_def.primary_key && !table_primary_key.is_empty() {
            panic!(
                "Column {} cannot be declared a primary key while the table also specifies one",
                column_def.name
            )
        }
        if table_primary_key.contains(&column_def.name.to_string()) {
            column_def.primary_key = true;
        }
        column_def
    });
    let columns = columns_defs.clone().collect::<Punctuated<_, Comma>>();
    let primary_keys = columns_defs
        .clone()
        .filter(|c| c.primary_key)
        .collect::<Punctuated<_, Comma>>();
    let columns_enum = iter
        .clone()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Punctuated<_, Comma>>();
    let match_variants = iter.clone().map(|field| {
        let column_name = field.ident.as_ref().unwrap();
        let column_def = decode_field(&field);
        quote! { Column::#column_name => #column_def }
    });
    //let primary_keys = sql_list_primary_keys(&input).join(", ");
    //fields.iter().map(result, |col|)
    quote! {
        #[allow(non_camel_case_types)]
        pub enum Column {
            #columns_enum
        }
        impl ::tank::ColumnTrait for Column {
            fn def(&self) -> ::tank::ColumnDef {
                match &self {
                    #(#match_variants,)*
                    _ => panic!("Unexpected column type"),
                }
            }
        }
        impl ::tank::Entity for #name {
            type Column = Column;

            fn table_name() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock::<Vec<::tank::ColumnDef>> = ::std::sync::LazyLock::new(|| { vec![#columns] });
                &RESULT
            }

            fn primary_key() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock::<Vec<::tank::ColumnDef>> = ::std::sync::LazyLock::new(|| { vec![#primary_keys] });
                &RESULT
            }

            async fn create_table<E: ::tank::Executor>(executor: &mut E, if_not_exists: bool) -> ::tank::Result<()> {
                let mut query = String::with_capacity(512);
                ::tank::Driver::sql_writer(executor.driver()).sql_create_table::<#name>(&mut query, if_not_exists);
                executor.execute(::tank::Query::Raw(query)).await.map(|_| ())
            }

            async fn drop_table<E: ::tank::Executor>(executor: &mut E, if_exists: bool) -> ::tank::Result<()> {
                let mut query = String::with_capacity(64);
                ::tank::Driver::sql_writer(executor.driver()).sql_drop_table::<#name>(&mut query, if_exists);
                executor.execute(::tank::Query::Raw(query)).await.map(|_| ())
            }

            // fn primary_key(&self) -> Vec<gluesql::core::ast::ColumnDef> {
            //     vec![]
            // }
        }

    }
    .into()
}
