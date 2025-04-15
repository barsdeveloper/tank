mod column_enum;
mod decode_fields;
mod table_name;
mod table_primary_key;

use column_enum::column_enum;
use decode_fields::decode_field;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, ItemStruct};
use table_name::table_name;
use table_primary_key::table_primary_key;

#[proc_macro_derive(
    Entity,
    attributes(table_name, column_type, default, primary_key, unique)
)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let item: ItemStruct = parse_macro_input!(input as ItemStruct);
    let ref name = item.ident;
    let table_name = table_name(&item);
    let table_primary_key = table_primary_key(&item);
    let iter = item.fields.iter();
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
    let primary_keys = columns_defs.clone().filter(|c| c.primary_key);
    let primary_key_tuple = primary_keys
        .clone()
        .map(|k| {
            iter.clone()
                .find(|f| *f.ident.as_ref().unwrap() == *k.name)
                .unwrap()
                .ty
                .to_token_stream()
        })
        .collect::<Punctuated<_, Comma>>();
    let primary_keys = primary_keys.collect::<Punctuated<_, Comma>>();
    let column = column_enum(&item);
    quote! {
        #column
        impl ::tank::Entity for #name {
            type Column = Column;
            type PrimaryKey = (#primary_key_tuple);

            fn table_name() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock::<Vec<::tank::ColumnDef>> =
                    ::std::sync::LazyLock::new(|| { vec![#columns] });
                &RESULT
            }

            fn primary_key() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock::<Vec<::tank::ColumnDef>> =
                    ::std::sync::LazyLock::new(|| { vec![#primary_keys] });
                &RESULT
            }

            async fn create_table<E: ::tank::Executor>(
                executor: &mut E,
                if_not_exists: bool
            ) -> ::tank::Result<()> {
                let mut query = String::with_capacity(512);
                ::tank::SqlWriter::sql_create_table::<#name>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    if_not_exists
                );
                executor.execute(::tank::Query::Raw(query)).await.map(|_| {()})
            }

            async fn drop_table<E: ::tank::Executor>(
                executor: &mut E,
                if_exists: bool
            ) -> ::tank::Result<()> {
                let mut query = String::with_capacity(64);
                ::tank::SqlWriter::sql_drop_table::<#name>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    if_exists
                );
                executor.execute(::tank::Query::Raw(query)).await.map(|_| {()})
            }

            async fn find_by_pk<E: ::tank::Executor>(
                executor: &mut E,
                primary_key: &Self::PrimaryKey,
            ) -> ::tank::Result<Self> {
                todo!()
            }
        }

    }
    .into()
}
