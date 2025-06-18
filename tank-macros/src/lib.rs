mod column_trait;
mod decode_column;
mod decode_expression;
mod decode_join;
mod encode_column_def;
mod encode_column_ref;
mod from_row_trait;
mod schema_name;
mod table_name;
mod table_primary_key;

use crate::{encode_column_def::encode_column_def, from_row_trait::from_row_trait};
use column_trait::column_trait;
use decode_column::decode_column;
use decode_expression::decode_expression;
use decode_join::JoinParsed;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use schema_name::schema_name;
use std::iter::zip;
use syn::{parse_macro_input, Expr, ItemStruct};
use table_name::table_name;
use table_primary_key::table_primary_key;
use tank_core::{flag_evaluated, PrimaryKeyType};

#[proc_macro_derive(
    Entity,
    attributes(
        schema_name,
        table_name,
        column_name,
        column_type,
        default_value,
        primary_key,
        unique,
        auto_increment,
    )
)]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let item: ItemStruct = parse_macro_input!(input as ItemStruct);
    let name = &item.ident;
    let schema_name = schema_name(&item);
    let table_name = table_name(&item);
    let primary_keys = table_primary_key(&item);
    let fields = item.fields.iter();
    let metadata_and_filter: Vec<(decode_column::ColumnMetadata, proc_macro2::TokenStream)> = fields
        .clone()
        .map(|f| {
            let mut metadata = decode_column(&f, &item);
            if metadata.primary_key == PrimaryKeyType::PrimaryKey && !primary_keys.is_empty() {
                panic!(
                    "Column `{}` cannot be declared as a primary key while the table also specifies one",
                    metadata.name
                )
            }
            if primary_keys
                .iter()
                .find(|pk| **pk == metadata.name)
                .is_some()
            {
                metadata.primary_key = if primary_keys.len() == 1 {
                    PrimaryKeyType::PrimaryKey
                } else {
                    PrimaryKeyType::PartOfPrimaryKey
                };
            }
            let filter_passive = if let Some(ref filter_passive) = metadata.check_passive {
                let field = &f.ident;
                filter_passive(quote!(self.#field))
            } else {
                quote!(true)
            };
            (metadata, filter_passive)
        })
        .collect::<Vec<_>>();
    let from_row = from_row_trait(&item);
    let primary_keys_indexes: Vec<_> = metadata_and_filter
        .iter()
        .enumerate()
        .filter_map(|(i, (m, ..))| {
            if matches!(
                m.primary_key,
                PrimaryKeyType::PrimaryKey | PrimaryKeyType::PartOfPrimaryKey
            ) {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    let primary_key = primary_keys_indexes.iter().map(|i| {
        let field = fields
            .clone()
            .nth(*i)
            .as_ref()
            .expect(&format!("Field {} is expected", i))
            .ident
            .as_ref()
            .expect("Field must have a name")
            .clone();
        quote!(self.#field)
    });
    let primary_key_defs = primary_keys_indexes.iter().map(|i| quote!(columns[#i]));
    let primary_key_types = primary_keys_indexes.iter().map(|i| {
        item.fields
            .iter()
            .nth(*i)
            .expect("The primary key should be present")
            .ty
            .to_token_stream()
    });
    let column = column_trait(&item);
    let value_and_filter =
        zip(fields.clone(), metadata_and_filter.iter()).map(|(field, (_, filter))| {
            let name = &field.ident;
            quote!((self.#name.clone().into(), #filter))
        });
    let label_and_filter = metadata_and_filter.iter().map(|(column, filter)| {
        let name = &column.name;
        quote!((#name.into(), #filter))
    });
    let columns_defs = metadata_and_filter.iter().map(|(c, _)| {
        let field = &c.ident;
        encode_column_def(&c, quote!(#name::#field))
    });
    quote! {
        #from_row
        #column
        impl ::tank::Entity for #name {
            type PrimaryKey<'a> = (#(&'a #primary_key_types),*);

            fn table_name() -> &'static str {
                #table_name
            }

            fn schema_name() -> &'static str {
                #schema_name
            }

            fn table_ref() -> &'static ::tank::TableRef {
                static TABLE_REF: ::tank::TableRef = ::tank::TableRef {
                    name: #table_name,
                    schema: #schema_name,
                    alias: ::std::borrow::Cow::Borrowed(""),
                };
                &TABLE_REF
            }

            fn columns_defs() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock<Box<[::tank::ColumnDef]>> =
                    ::std::sync::LazyLock::new(|| vec![#(#columns_defs),*].into_boxed_slice());
                &RESULT
            }

            fn primary_key_defs() -> &'static [&'static ::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock<Box<[&::tank::ColumnDef]>> =
                    ::std::sync::LazyLock::new(|| {
                        let columns = #name::columns_defs();
                        vec![#(&#primary_key_defs),*].into_boxed_slice()
                    });
                &RESULT
            }

            async fn create_table<E: ::tank::Executor>(
                executor: &mut E,
                if_not_exists: bool,
            ) -> ::tank::Result<()> {
                let mut query = String::with_capacity(512);
                ::tank::SqlWriter::sql_create_table::<#name>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    if_not_exists,
                );
                executor
                    .execute(query.into())
                    .await
                    .map(|_| ())
            }

            async fn drop_table<E: ::tank::Executor>(
                executor: &mut E,
                if_exists: bool,
            ) -> ::tank::Result<()> {
                let mut query = String::with_capacity(64);
                ::tank::SqlWriter::sql_drop_table::<#name>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    if_exists,
                );
                executor
                    .execute(::tank::Query::Raw(query.into()))
                    .await
                    .map(|_| ())
            }

            fn find_one<E: ::tank::Executor>(
                executor: &mut E,
                primary_key: &Self::PrimaryKey<'_>,
            ) -> impl ::std::future::Future<Output = ::tank::Result<Option<Self>>> {
                async {
                    todo!("will do")
                }
            }

            fn find_many<E: ::tank::Executor, Expr: ::tank::Expression>(
                executor: &mut E,
                condition: Expr,
            ) -> impl ::tank::stream::Stream<Item = ::tank::Result<Self>> {
                ::tank::stream::empty()
            }

            fn row_labeled(&self) -> ::tank::RowLabeled {
                ::tank::RowLabeled {
                    labels: [#(#label_and_filter),*]
                        .into_iter()
                        .filter_map(|(v, f)| if f { Some(v) } else { None })
                        .collect::<::tank::RowNames>(),
                    values: self.row(),
                }
            }

            fn row(&self) -> ::tank::Row {
                [#(#value_and_filter),*]
                    .into_iter()
                    .filter_map(|(v, f)| if f { Some(v) } else { None })
                    .collect::<::tank::Row>()
            }

            fn primary_key<'a>(&'a self) -> Self::PrimaryKey<'a> {
                (#(&#primary_key),*)
            }

            async fn save<E: ::tank::Executor>(&self, executor: &mut E) -> ::tank::Result<()> {
                Ok(())
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn expr(input: TokenStream) -> TokenStream {
    let input: TokenStream = flag_evaluated(input.into()).into();
    let expr = parse_macro_input!(input as Expr);
    let parsed = decode_expression(&expr);
    quote!(#parsed).into()
}

#[proc_macro]
pub fn join(input: TokenStream) -> TokenStream {
    let result = parse_macro_input!(input as JoinParsed);
    result.0.into()
}
