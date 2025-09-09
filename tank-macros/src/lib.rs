mod column_trait;
mod decode_column;
mod decode_expression;
mod decode_join;
mod decode_table;
mod encode_column_def;
mod encode_column_ref;
mod frag_evaluated;
mod from_row_trait;

use crate::{
    decode_column::ColumnMetadata,
    decode_table::{TableMetadata, decode_table},
    encode_column_def::encode_column_def,
    from_row_trait::from_row_trait,
};
use column_trait::column_trait;
use decode_column::decode_column;
use decode_expression::decode_expression;
use decode_join::JoinParsed;
use frag_evaluated::flag_evaluated;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Expr, Ident, Index, ItemStruct, parse_macro_input, punctuated::Punctuated, token::AndAnd,
};
use tank_core::PrimaryKeyType;

#[proc_macro_derive(Entity, attributes(tank))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let table = decode_table(parse_macro_input!(input as ItemStruct));
    let ident = &table.item.ident;
    let name = &table.name;
    let schema = &table.schema;
    let metadata_and_filter = table
        .columns
        .iter()
        .map(|metadata| {
            let filter_passive = if let Some(ref filter_passive) = metadata.check_passive {
                let field = &metadata.ident;
                filter_passive(quote!(self.#field))
            } else {
                quote!(true)
            };
            (metadata, filter_passive)
        })
        .collect::<Vec<_>>();
    let (from_row_factory, from_row) = from_row_trait(&table);
    let primary_keys: Vec<_> = metadata_and_filter
        .iter()
        .enumerate()
        .filter_map(|(i, (m, ..))| {
            if matches!(
                m.primary_key,
                PrimaryKeyType::PrimaryKey | PrimaryKeyType::PartOfPrimaryKey
            ) {
                Some((i, m))
            } else {
                None
            }
        })
        .collect();
    let primary_key = primary_keys
        .iter()
        .map(|(_i, c)| c.ident.clone())
        .map(|ident| quote!(self.#ident));
    let primary_key_def = primary_keys.iter().map(|(i, _)| quote!(columns[#i]));
    let unique_defs = &table
        .unique
        .iter()
        .map(|v| {
            if v.is_empty() {
                quote!()
            } else {
                let i = v.iter();
                quote!(vec![#(&columns[#i]),*].into_boxed_slice())
            }
        })
        .collect::<Vec<_>>();
    let unique_defs = quote!(vec![#(#unique_defs),*].into_boxed_slice());
    let primary_key_types = primary_keys.iter().map(|(_, c)| c.ty.clone());
    let column = column_trait(&table);
    let label_value_and_filter = metadata_and_filter.iter().map(|(column, filter)| {
        let name = &column.name;
        let field = &column.ident;
        quote!((#name.into(), self.#field.clone().into(), #filter))
    });
    let row_full = metadata_and_filter
        .iter()
        .map(|(ColumnMetadata { ident, .. }, _)| quote!(self.#ident.clone().into()));
    let columns = metadata_and_filter.iter().map(|(c, _)| {
        let field = &c.ident;
        encode_column_def(&c, quote!(#ident::#field))
    });
    let primary_key_condition = primary_keys.iter().enumerate().map(|(i, (_, c))| {
        (
            &c.ident,
            Index::from(i),
            Ident::new(&format!("pk{}", i), c.ident.span()),
        )
    });
    let primary_key_condition_declaration = primary_key_condition
        .clone()
        .map(|(_, i, pk)| quote! { let #pk = primary_key.#i.to_owned(); })
        .collect::<TokenStream2>();
    let primary_key_condition_expression = primary_key_condition
        .clone()
        .map(|(field, _i, pk)| quote!(#ident::#field == # #pk))
        .collect::<Punctuated<_, AndAnd>>();
    quote! {
        #from_row
        #column
        impl ::tank::Entity for #ident {
            type PrimaryKey<'a> = (#(&'a #primary_key_types,)*);

            fn table_ref() -> &'static ::tank::TableRef {
                static TABLE_REF: ::tank::TableRef = ::tank::TableRef {
                    name: #name,
                    schema: #schema,
                    alias: ::std::borrow::Cow::Borrowed(""),
                };
                &TABLE_REF
            }

            fn columns() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock<Box<[::tank::ColumnDef]>> =
                    ::std::sync::LazyLock::new(|| vec![#(#columns),*].into_boxed_slice());
                &RESULT
            }

            fn primary_key_def() -> impl ExactSizeIterator<Item = &'static ::tank::ColumnDef> {
                static RESULT: ::std::sync::LazyLock<Box<[&::tank::ColumnDef]>> =
                    ::std::sync::LazyLock::new(|| {
                        let columns = #ident::columns();
                        vec![#(&#primary_key_def),*].into_boxed_slice()
                    });
                RESULT.iter().copied()
            }

            fn unique_defs()
            -> impl ExactSizeIterator<Item = impl ExactSizeIterator<Item = &'static ::tank::ColumnDef>> {
                static RESULT: ::std::sync::LazyLock<Box<[Box<[&'static ::tank::ColumnDef]>]>> =
                    ::std::sync::LazyLock::new(|| {
                        let columns = #ident::columns();
                        #unique_defs
                    });
                RESULT.iter().map(|v| v.iter().copied())
            }

            fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<Self> {
                #from_row_factory::<Self>::from_row(row)
            }

            async fn create_table<Exec: ::tank::Executor>(
                executor: &mut Exec,
                if_not_exists: bool,
                create_schema: bool,
            ) -> ::tank::Result<()> {
                let mut query = String::with_capacity(2048);
                if create_schema && !#schema.is_empty() {
                    ::tank::SqlWriter::write_create_schema::<#ident>(
                        &::tank::Driver::sql_writer(executor.driver()),
                        &mut query,
                        true,
                    );
                    query.push('\n');
                }
                ::tank::SqlWriter::write_create_table::<#ident>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    if_not_exists,
                );
                executor
                    .execute(::tank::Query::Raw(query.into()))
                    .await
                    .map(|_| ())
            }

            async fn drop_table<Exec: ::tank::Executor>(
                executor: &mut Exec,
                if_exists: bool,
                drop_schema: bool,
            ) -> ::tank::Result<()> {
                let mut query = String::with_capacity(256);
                if drop_schema && !#schema.is_empty() {
                    ::tank::SqlWriter::write_drop_schema::<#ident>(
                        &::tank::Driver::sql_writer(executor.driver()),
                        &mut query,
                        true,
                    );
                    query.push('\n');
                }
                ::tank::SqlWriter::write_drop_table::<#ident>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    if_exists,
                );
                executor
                    .execute(::tank::Query::Raw(query.into()))
                    .await
                    .map(|_| ())
            }

            fn insert_one<Exec: ::tank::Executor, E: ::tank::Entity>(
                executor: &mut Exec,
                entity: &E,
            ) -> impl ::std::future::Future<Output = ::tank::Result<::tank::RowsAffected>> + Send {
                let mut query = String::with_capacity(128);
                ::tank::SqlWriter::write_insert(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    [entity],
                    false,
                );
                executor.execute(::tank::Query::Raw(query.into()))
            }

            fn insert_many<'a, Exec, It>(
                executor: &mut Exec,
                entities: It,
            ) -> impl ::std::future::Future<Output = ::tank::Result<::tank::RowsAffected>> + Send
            where
                Self: 'a,
                Exec: ::tank::Executor,
                It: IntoIterator<Item = &'a Self>
            {
                let mut query = String::with_capacity(128);
                ::tank::SqlWriter::write_insert(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    entities,
                    false,
                );
                executor.execute(::tank::Query::Raw(query.into()))
            }

            fn find_pk<E: ::tank::Executor>(
                executor: &mut E,
                primary_key: &Self::PrimaryKey<'_>,
            ) -> impl ::std::future::Future<Output = ::tank::Result<Option<Self>>> {
                #primary_key_condition_declaration
                async move {
                    let condition = ::tank::expr!(#primary_key_condition_expression);
                    let stream = ::tank::DataSet::select(
                        Self::table_ref(),
                        Self::columns()
                            .iter()
                            .map(|c| &c.column_ref as &dyn ::tank::Expression),
                        executor,
                        &condition,
                        Some(1),
                    );
                    let mut stream = std::pin::pin!(stream);
                    ::tank::stream::StreamExt::next(&mut stream)
                        .await
                        .map(|v| v.and_then(Self::from_row))
                        .transpose()
                }
            }

            fn find_many<Exec: ::tank::Executor, Expr: ::tank::Expression>(
                executor: &mut Exec,
                condition: &Expr,
                limit: Option<u32>,
            ) -> impl ::tank::stream::Stream<Item = ::tank::Result<Self>> {
                ::tank::stream::StreamExt::map(
                    ::tank::DataSet::select(
                        Self::table_ref(),
                        Self::columns()
                            .iter()
                            .map(|c| &c.column_ref as &dyn ::tank::Expression),
                        executor,
                        condition,
                        limit,
                    ),
                    |result| result.and_then(Self::from_row),
                )
            }

            fn delete_one<Exec: ::tank::Executor>(
                executor: &mut Exec,
                primary_key: Self::PrimaryKey<'_>,
            ) -> impl ::std::future::Future<Output = ::tank::Result<::tank::RowsAffected>> + Send
            where
                Self: Sized
            {
                #primary_key_condition_declaration
                let condition = ::tank::expr!(#primary_key_condition_expression);
                let mut query = String::with_capacity(128);
                ::tank::SqlWriter::write_delete::<Self, _>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    &condition,
                );
                executor.execute(::tank::Query::Raw(query.into()))
            }

            fn delete_many<Exec: ::tank::Executor, Expr: ::tank::Expression>(
                executor: &mut Exec,
                condition: &Expr,
            ) -> impl ::std::future::Future<Output = ::tank::Result<::tank::RowsAffected>> + Send
            where
                Self: Sized
            {
                let mut query = String::with_capacity(128);
                ::tank::SqlWriter::write_delete::<Self, _>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    condition,
                );
                executor.execute(::tank::Query::Raw(query.into()))
            }

            fn row_filtered(&self) -> Box<[(&'static str, ::tank::Value)]> {
                [#(#label_value_and_filter),*]
                    .into_iter()
                    .filter_map(|(n, v, f)| if f { Some((n, v)) } else { None })
                    .collect()
            }

            fn row_full(&self) -> ::tank::Row {
                [#(#row_full),*].into()
            }

            fn primary_key<'a>(&'a self) -> Self::PrimaryKey<'a> {
                (#(&#primary_key,)*)
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn expr(input: TokenStream) -> TokenStream {
    let mut input: TokenStream = flag_evaluated(input.into()).into();
    if input.is_empty() {
        input = quote!(false).into();
    }
    let expr = parse_macro_input!(input as Expr);
    let parsed = decode_expression(&expr);
    quote!(#parsed).into()
}

#[proc_macro]
pub fn join(input: TokenStream) -> TokenStream {
    let result = parse_macro_input!(input as JoinParsed);
    result.0.into()
}
