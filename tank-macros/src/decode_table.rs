use convert_case::{Case, Casing};
use quote::ToTokens;
use syn::{parse::ParseBuffer, Expr, ExprLit, ExprTuple, Ident, ItemStruct, Lit, LitStr, Result};

#[derive(Debug)]
pub(crate) struct TableMetadata {
    pub(crate) ident: Ident,
    pub(crate) name: String,
    pub(crate) schema: String,
    pub(crate) primary_key: Option<Vec<String>>,
}

pub fn decode_table(item: &ItemStruct) -> TableMetadata {
    let mut metadata = TableMetadata {
        ident: item.ident.clone(),
        name: item.ident.to_string().to_case(Case::Snake),
        schema: String::new(),
        primary_key: None,
    };
    if metadata.name.starts_with('_') {
        metadata.name.remove(0);
    }
    for attr in &item.attrs {
        let meta = &attr.meta;
        if meta.path().is_ident("tank") {
            let Ok(list) = meta.require_list() else {
                panic!(
                    "Error while parsing `tank`, use it like: `#[tank(attribute = value, ...)]`",
                );
            };
            let _ = list.parse_nested_meta(|arg| {
                if arg.path.is_ident("name") {
                    let Ok(v) = arg.value().and_then(ParseBuffer::parse::<LitStr>) else {
                        panic!(
                            "Error while parsing `name`, use it like: `#[tank(name = \"my_table\")]`"
                        );
                    };
                    metadata.name = v.value();
                } else if arg.path.is_ident("schema") {
                    let Ok(v) = arg.value().and_then(ParseBuffer::parse::<LitStr>) else {
                        panic!(
                            "Error while parsing `schema`, use it like: `#[tank(schema = \"my_schema\")]`"
                        );
                    };
                    metadata.schema = v.value();
                } else if arg.path.is_ident("primary_key") {
                    let Ok(primary_key) = arg.value().and_then(|v| v.parse::<Expr>()) else {
                        panic!("Error while parsing `primary_key`, use it like: `#[tank(primary_key = (\"k1\", \"k2\", ...))]`");
                    };
                    metadata.primary_key = Some(  match primary_key  {
                            Expr::Lit(ExprLit {lit:Lit::Str(str) ,..}) => vec![str.value()],
                            Expr::Tuple(ExprTuple{elems,..}) => elems.iter().map(|v| {
                            let Expr::Lit(ExprLit {lit:Lit::Str(str) ,..}) = v else {
                                panic!("Error while parsing `primary_key`, use it like: `#[tank(primary_key = (\"k1\", \"k2\", ...))]`");
                            };
                            Ok(str.value())
                        }).collect::<Result<Vec<_>>>()?,
                            _ => todo!(),
                        })
                } else {
                    panic!("Unknown attribute `{}` inside tank macro", arg.path.to_token_stream().to_string());
                }
                Ok(())
            });
        }
    }
    metadata
}
