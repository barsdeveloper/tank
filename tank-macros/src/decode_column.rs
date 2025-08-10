use crate::expr;
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::fmt::Debug;
use syn::{
    Expr, ExprCall, ExprLit, ExprMethodCall, ExprPath, Field, Ident, Lit, LitStr, Type,
    parse::ParseBuffer,
};
use tank_core::{CheckPassive, PrimaryKeyType, TypeDecoded, Value, decode_type};

pub(crate) struct ColumnMetadata {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) name: String,
    pub(crate) column_type: String,
    pub(crate) value: Value,
    pub(crate) nullable: bool,
    pub(crate) default: Option<TokenStream>,
    pub(crate) primary_key: PrimaryKeyType,
    pub(crate) references: Option<(String, String)>,
    pub(crate) unique: bool,
    pub(crate) passive: bool,
    pub(crate) check_passive: Option<CheckPassive>,
    pub(crate) comment: String,
}

impl Debug for ColumnMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColumnMetadata")
            .field("ident", &self.ident)
            .field("ty", &"..")
            .field("name", &self.name)
            .field("column_type", &self.column_type)
            .field("value", &self.value)
            .field("nullable", &self.nullable)
            .field("default", &self.default)
            .field("primary_key", &self.primary_key)
            .field("references", &self.references)
            .field("unique", &self.unique)
            .field("passive", &self.passive)
            .field("check_passive", &"..")
            .field("comment", &self.comment)
            .finish()
    }
}

pub fn decode_column(field: &Field) -> ColumnMetadata {
    let (
        TypeDecoded {
            value,
            nullable,
            passive,
        },
        check_passive,
    ) = if let Type::Path(..) = &field.ty {
        decode_type(&field.ty)
    } else if let Type::Array(..) = &field.ty {
        decode_type(&field.ty)
    } else {
        Default::default()
    };
    let ident = field
        .ident
        .clone()
        .expect("Field is expected to have a name");
    let name = ident.to_string();
    let mut metadata = ColumnMetadata {
        ident,
        ty: field.ty.clone(),
        name,
        column_type: "".into(),
        value,
        nullable,
        default: None,
        primary_key: PrimaryKeyType::None,
        references: None,
        unique: false,
        passive,
        check_passive,
        comment: String::new(),
    };
    if metadata.name.starts_with('_') {
        metadata.name.remove(0);
    }
    for attr in &field.attrs {
        let meta = &attr.meta;
        if meta.path().is_ident("tank") {
            let Ok(list) = meta.require_list() else {
                panic!("Error while parsing `tank`, use it like: `#[tank(attribute = value, ..)]`",);
            };
            let _ = list.parse_nested_meta(|arg| {
                if arg.path.is_ident("default") {
                    let Ok(v) = arg.value().and_then(ParseBuffer::parse::<TokenTree>)
                    else {
                        panic!("Error while parsing `default`, use it like: `#[tank(default = some_expression)]`");
                    };
                    metadata.default = Some(expr(v.to_token_stream().into()).into());
                } else if arg.path.is_ident("name") {
                    let Ok(v) = arg.value().and_then(ParseBuffer::parse::<LitStr>) else {
                      panic!("Error while parsing `name`, use it like: `#[tank(name = \"my_column\")]`");
                    };
                    metadata.name = v.value();
                } else if arg.path.is_ident("type") {
                    let Ok(v) = arg.value().and_then(ParseBuffer::parse::<LitStr>) else {
                        panic!("Error while parsing `type`, use it like: `#[tank(type = \"VARCHAR\")]`");
                    };
                    metadata.column_type = v.value();
                } else if arg.path.is_ident("primary_key") {
                    let Err(..) = arg.value() else {
                        // value() is Err for Meta::Path
                        panic!(
                            "Error while parsing `primary_key`, use it like: `#[tank(primary_key)]`"
                        );
                    };
                    metadata.primary_key = PrimaryKeyType::PrimaryKey;
                    metadata.nullable = false;
                } else if arg.path.is_ident("references") {
                    let Ok(reference) = arg
                        .value()
                        .and_then(|v| {
                            if let Ok(v) = v.fork().parse::<ExprMethodCall>().map(|v| {
                                if !v.attrs.is_empty() {
                                    panic!("Table reference cannot have attributes");
                                }
                                if v.turbofish.is_some() {
                                    panic!("Table reference cannot have template arguments");
                                }
                                (format!("{}.{}", v.receiver.to_token_stream(), v.method.to_token_stream().to_string()), v.args)
                            }) {
                                return Ok(v);
                            }
                            if let Ok(v) = v.fork().parse::<ExprCall>().map(|v| {
                                if !v.attrs.is_empty() {
                                    panic!("Table reference cannot have attributes");
                                }
                                (v.func.to_token_stream().to_string(), v.args)
                            }) {
                                return Ok(v);
                            }
                            panic!("Unexpected expression syntax for `references` {:?}, use it like: `MyEntity::column` or `schema.table_name(column_name)`", v.to_string());
                        })
                            .map(|(func, args)| {
                                if args.len() != 1 {
                                    panic!("Expected references to have a single argument");
                                }
                                (
                                    func,
                                    args
                                        .iter()
                                        .map(|v| match v {
                                            Expr::Lit(ExprLit{lit: Lit::Str(v), ..}) => v.value(),
                                            Expr::Path(ExprPath {path,..}) if path.segments.len() == 1 => path.get_ident().unwrap().to_string(),
                                            _ => panic!("Expected the referred column name to be either a string or a identifier")
                                        })
                                        .take(1)
                                        .next()
                                        .unwrap(),
                                )
                            }
                        ) else {
                            panic!("Error while parsing `references` use it like: `MyEntity::column` or `schema.table_name(column_name)`");
                        };
                    metadata.references = reference.into();
                } else if arg.path.is_ident("unique") {
                    let Err(..) = arg.value() else {
                        panic!("Error while parsing `unique`, use it like: `#[tank(unique)]`");
                    };
                    metadata.unique = true;
                } else {
                    panic!("Unknown attribute `{}` inside tank macro", arg.path.to_token_stream().to_string());
                }
                Ok(())
            });
        } else if meta.path().is_ident("doc") {
            let Ok(&Expr::Lit(ExprLit {
                lit: Lit::Str(ref v),
                ..
            })) = meta.require_name_value().and_then(|v| Ok(&v.value))
            else {
                panic!("Error while parsing the comment, use it like: `/// Column comment");
            };
            if !metadata.comment.is_empty() {
                metadata.comment.push('\n');
            }
            metadata.comment.push_str(v.value().trim());
        }
    }
    metadata
}
