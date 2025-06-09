mod column_trait;
mod decode_column;
mod decode_expression;
mod decode_join;
mod encode_column_def;
mod encode_column_ref;
mod schema_name;
mod table_name;
mod table_primary_key;

use crate::encode_column_def::encode_column_def;
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
use tank_core::flag_evaluated;

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
    let metadata_and_filter = fields.clone().map(|f| {
       let mut metadata = decode_column(&f, &item);
        if metadata.primary_key && !primary_keys.is_empty() {
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
            if primary_keys.len() == 1 {
                metadata.primary_key = true;
            }
        }
        let filter_passive = if let Some(ref filter_passive) = metadata.check_passive {
            let field = &f.ident;
            filter_passive(quote!(self.#field))
        } else {
            quote!(true)
        };
       (metadata, filter_passive)
    }).collect::<Vec<_>>();
    let primary_keys: Vec<_> = if primary_keys.is_empty() {
        metadata_and_filter
            .iter()
            .filter_map(|(m, f)| if m.primary_key { Some(m) } else { None })
            .collect()
    } else {
        primary_keys
            .iter()
            .map(|v| {
                &metadata_and_filter.iter().find(|m| *v == m.0.name).expect(
                    "Primary keys must exist here, it is being checked in `table_primary_key()`",
                ).0
            })
            .collect()
    };
    let primary_key_types = primary_keys.iter().map(|key| {
        fields
            .clone()
            .find(|f| decode_column(f, &item).name == key.name)
            .expect(&format!(
                "Could not find the primary key `{}` among the fields",
                key.name
            ))
            .ty
            .to_token_stream()
    });
    let primary_keys = primary_keys.iter().map(|v| encode_column_def(&v));
    let column = column_trait(&item);
    let value_and_filter =
        zip(fields.clone(), metadata_and_filter.iter()).map(|(field, (column, filter))| {
            let name = &field.ident;
            quote!((self.#name.clone().into(), #filter))
        });
    let label_and_filter = metadata_and_filter.iter().map(|(column, filter)| {
        let name = &column.name;
        quote!((#name.into(), #filter))
    });
    let columns = metadata_and_filter
        .iter()
        .map(|(c, _)| encode_column_def(&c));
    quote! {
        #column
        impl ::tank::Entity for #name {
            type PrimaryKey = (#(#primary_key_types),*);

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

            fn columns() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock::<Box<[::tank::ColumnDef]>> =
                    ::std::sync::LazyLock::new(|| { vec![#(#columns),*].into_boxed_slice() });
                &RESULT
            }

            fn primary_key() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock::<Box<[::tank::ColumnDef]>> =
                    ::std::sync::LazyLock::new(|| { vec![#(#primary_keys),*].into_boxed_slice() });
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

            async fn save<E: ::tank::Executor>(
                &self,
                executor: &mut E,
            ) -> ::tank::Result<()> {
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
