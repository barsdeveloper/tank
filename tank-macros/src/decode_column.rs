use crate::{decode_table, expr};
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::fmt::Debug;
use syn::{parse::ParseBuffer, Field, Ident, ItemStruct, LitStr, Type};
use tank_core::{decode_type, CheckPassive, PrimaryKeyType, TypeDecoded, Value};

pub(crate) struct ColumnMetadata {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) name: String,
    pub(crate) table: String,
    pub(crate) schema: String,
    pub(crate) column_type: String,
    pub(crate) value: Value,
    pub(crate) nullable: bool,
    pub(crate) default: Option<TokenStream>,
    pub(crate) primary_key: PrimaryKeyType,
    pub(crate) unique: bool,
    pub(crate) auto_increment: bool,
    pub(crate) passive: bool,
    pub(crate) check_passive: Option<CheckPassive>,
}

impl Debug for ColumnMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColumnMetadata")
            .field("ident", &self.ident)
            .field("ty", &"..")
            .field("name", &self.name)
            .field("table", &self.table)
            .field("schema", &self.schema)
            .field("column_type", &self.column_type)
            .field("value", &self.value)
            .field("nullable", &self.nullable)
            .field("default", &self.default)
            .field("primary_key", &self.primary_key)
            .field("unique", &self.unique)
            .field("auto_increment", &self.auto_increment)
            .field("passive", &self.passive)
            .field("check_passive", &"..")
            .finish()
    }
}

pub fn decode_column(field: &Field, item: &ItemStruct) -> ColumnMetadata {
    let (
        TypeDecoded {
            value,
            nullable,
            passive,
        },
        check_passive,
    ) = if let Type::Path(..) = &field.ty {
        decode_type(&field.ty)
    } else {
        Default::default()
    };
    let ident = field
        .ident
        .clone()
        .expect("Field is expected to have a name");
    let name = ident.to_string();
    let table = decode_table(item);
    let mut metadata = ColumnMetadata {
        ident,
        ty: field.ty.clone(),
        name,
        table: table.name,
        schema: table.schema,
        column_type: "".into(),
        value,
        nullable,
        default: None,
        primary_key: PrimaryKeyType::None,
        unique: false,
        auto_increment: false,
        passive,
        check_passive,
    };
    if metadata.name.starts_with('_') {
        metadata.name.remove(0);
    }
    for attr in &field.attrs {
        let meta = &attr.meta;
        if meta.path().is_ident("tank") {
            let Ok(list) = meta.require_list() else {
                panic!(
                    "Error while parsing `tank`, use it like: `#[tank(attribute = value, ...)]`",
                );
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
                        panic!("Error while parsing `type`, use it like: `#[tank(type = \"VARCHAR\")]`"
                        );
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
                } else if arg.path.is_ident("unique") {
                    let Err(..) = arg.value() else {
                        panic!("Error while parsing `unique`, use it like: `#[tank(unique)]`");
                    };
                    metadata.unique = true;
                } else if arg.path.is_ident("auto_increment") {
                    let Err(..) = arg.value() else {
                        // value() is Err for Meta::Path
                        panic!("Error while parsing `auto_increment`, use it like: `#[tank(auto_increment)]`");
                    };
                    metadata.auto_increment = true;
                } else {
                    panic!("Unknown attribute `{}` inside tank macro", arg.path.to_token_stream().to_string());
                }
                Ok(())
            });
        }
    }
    metadata
}
