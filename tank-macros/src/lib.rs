mod column_trait;
mod decode_expression;
mod decode_fields;
mod decode_join;
mod schema_name;
mod table_name;
mod table_primary_key;

use column_trait::column_trait;
use decode_expression::decode_expression;
use decode_fields::decode_field;
use decode_join::JoinParsed;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use schema_name::schema_name;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Expr, ItemStruct};
use table_name::table_name;
use table_primary_key::table_primary_key;

#[proc_macro_derive(
    Entity,
    attributes(
        schema_name,
        table_name,
        column_name,
        column_type,
        default,
        primary_key,
        unique
    )
)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let item: ItemStruct = parse_macro_input!(input as ItemStruct);
    let name = &item.ident;
    let schema_name = schema_name(&item);
    let table_name = table_name(&item);
    let table_primary_key = table_primary_key(&item);
    let fields = item.fields.iter();
    let columns_defs = fields.clone().map(|f| {
        let mut column_def = decode_field(&f, &item);
        if column_def.primary_key && !table_primary_key.is_empty() {
            panic!(
                "Column {} cannot be declared as a primary key while the table also specifies one",
                column_def.name()
            )
        }
        if table_primary_key.contains(&column_def.name().to_string()) {
            column_def.primary_key = true;
        }
        column_def
    });
    let columns = columns_defs.clone().collect::<Punctuated<_, Comma>>();
    let primary_keys = columns_defs.clone().filter(|c| c.primary_key);
    let primary_key_tuple = primary_keys
        .clone()
        .map(|key| {
            fields
                .clone()
                .find(|f| decode_field(f, &item).name() == key.name())
                .expect(&format!(
                    "Could not find the primary key \"{}\" among the fields",
                    key.name()
                ))
                .ty
                .to_token_stream()
        })
        .collect::<Punctuated<_, Comma>>();
    let primary_keys = primary_keys.collect::<Punctuated<_, Comma>>();
    let column = column_trait(&item);
    quote! {
        #column
        impl ::tank::Entity for #name {
            type PrimaryKey = (#primary_key_tuple);

            fn table_name() -> &'static str {
                #table_name
            }

            fn schema_name() -> &'static str {
                #schema_name
            }

            fn table_ref() -> &'static ::tank::TableRef {
                static TABLE_REF: ::tank::TableRef = ::tank::TableRef {
                    name: ::std::borrow::Cow::Borrowed(#table_name),
                    schema: ::std::borrow::Cow::Borrowed(#schema_name),
                    alias: ::std::borrow::Cow::Borrowed(""),
                };
                &TABLE_REF
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
                executor.execute(::tank::Query::Raw(query.into())).await.map(|_| {()})
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
                executor.execute(::tank::Query::Raw(query.into())).await.map(|_| {()})
            }

            async fn find_by_key<E: ::tank::Executor>(
                executor: &mut E,
                primary_key: &Self::PrimaryKey,
            ) -> ::tank::Result<Self> {
                todo!("find_by_key")
            }

            async fn find_by_condition<E: ::tank::Executor, Expr: ::tank::Expression>(
                executor: &mut E,
                condition: Expr,
            ) -> ::tank::Result<Self> {
                todo!("find_by_condition")
            }
        }

    }
    .into()
}

#[proc_macro]
pub fn expr(input: TokenStream) -> TokenStream {
    let input: Expr = parse_macro_input!(input as Expr);
    let parsed = decode_expression(&input);
    quote!(#parsed).into()
}

#[proc_macro]
pub fn join(input: TokenStream) -> TokenStream {
    let result = parse_macro_input!(input as JoinParsed);
    result.0.into()
}
