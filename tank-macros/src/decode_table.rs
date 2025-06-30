use std::convert::identity;

use crate::decode_column;
use crate::decode_column::ColumnMetadata;
use convert_case::{Case, Casing};
use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{parse::ParseBuffer, Error, Expr, ExprLit, ExprPath, ItemStruct, Lit, LitStr, Result};
use tank_core::matches_path;

pub(crate) struct TableMetadata {
    pub(crate) columns: Vec<ColumnMetadata>,
    pub(crate) name: String,
    pub(crate) item: ItemStruct,
    pub(crate) schema: String,
    pub(crate) primary_key: Vec<String>,
    pub(crate) unique: Vec<Vec<String>>,
}

fn decode_set_columns<'a, I: Iterator<Item = &'a ColumnMetadata> + Clone>(
    item: &ItemStruct,
    col: Expr,
    mut columns: I,
) -> Result<Vec<String>> {
    Ok(match col {
        Expr::Lit(ExprLit {
            lit: Lit::Str(v), ..
        }) => {
            let v = v.value();
            if columns.find(|c| c.name == v).is_none() {
                return Err(Error::new(
                    v.span(),
                    format!("Column `{}` does not exist in the table", v),
                ));
            }
            vec![v]
        }
        Expr::Path(ExprPath { path, .. }) => {
            let Some(v) = columns.find(|c| {
                let c = c.ident.to_string();
                matches_path(&path, &["Self", &c])
                    || matches_path(&path, &[&item.ident.to_string(), &c])
            }) else {
                return Err(Error::new(
                    path.span(),
                    format!(
                        "Field `{}` does not exist in the entity",
                        path.to_token_stream().to_string()
                    ),
                ));
            };
            vec![v.name.to_string()]
        }
        Expr::Tuple(tuple) => {
            let elems: Vec<_> = tuple
                .elems
                .iter()
                .map(|v| decode_set_columns(&item, v.clone(), columns.clone()))
                .collect::<Result<_>>()?;
            if elems.iter().any(|v| v.len() != 1) {
                return Err(Error::new(
                    tuple.span(),
                    "Fields list inside tuple must either be a string literal column name or a column reference path",
                ));
            }
            elems.into_iter().flat_map(identity).collect()
        }
        _ => return Err(Error::new(Span::call_site(), "")),
    })
}

pub fn decode_table(item: ItemStruct) -> TableMetadata {
    let columns: Vec<_> = item
        .fields
        .iter()
        .map(|f| decode_column(f, &item))
        .collect();
    let mut name = item.ident.to_string().to_case(Case::Snake);
    let mut schema = String::new();
    let mut primary_key = vec![];
    let mut unique = vec![];
    if name.starts_with('_') {
        name.remove(0);
    }
    for attr in &item.attrs {
        let meta = &attr.meta;
        if meta.path().is_ident("tank") {
            let Ok(list) = meta.require_list() else {
                panic!("Error while parsing `tank`, use it like: `#[tank(attribute = value, ..)]`",);
            };
            let _ = list.parse_nested_meta(|arg| {
                if arg.path.is_ident("name") {
                    let Ok(value) = arg.value().and_then(ParseBuffer::parse::<LitStr>) else {
                        panic!(
                            "Error while parsing `name`, use it like: `#[tank(name = \"my_table\")]`"
                        );
                    };
                    name = value.value();
                } else if arg.path.is_ident("schema") {
                    let Ok(value) = arg.value().and_then(ParseBuffer::parse::<LitStr>) else {
                        panic!(
                            "Error while parsing `schema`, use it like: `#[tank(schema = \"my_schema\")]`"
                        );
                    };
                    schema = value.value();
                } else if arg.path.is_ident("primary_key") {
                    let Ok(value) = arg
                        .value()
                        .and_then(ParseBuffer::parse::<Expr>)
                        .and_then(|v| decode_set_columns(&item, v, columns.iter())) else {
                        panic!("Error while parsing `primary_key`, use it like: `#[tank(primary_key = (\"k1\", \"k2\", ..))]`");
                    };
                    if !primary_key.is_empty() {
                        panic!("Primary key attribute can appear just once on a table");
                    }
                    primary_key = value
                } else if arg.path.is_ident("unique") {
                    let Ok(value) = arg
                        .value()
                        .and_then(ParseBuffer::parse::<Expr>)
                        .and_then(|v| decode_set_columns(&item, v, columns.iter())) else {
                        panic!("Error while parsing `unique`, use it like: `#[tank(unique = (\"k1\", \"k2\", ..))]`, you can specify more than one");
                    };
                    unique.push(value);
                } else {
                    panic!("Unknown attribute `{}` inside tank macro", arg.path.to_token_stream().to_string());
                }
                Ok(())
            });
        }
    }
    TableMetadata {
        columns,
        name,
        item,
        schema,
        primary_key,
        unique,
    }
}
