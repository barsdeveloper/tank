use crate::{schema_name, table_name};
use syn::{Field, Ident, ItemStruct, LitStr, Type};
use tank_core::{decode_type, CheckPassive, PrimaryKeyType, TypeDecoded, Value};

pub(crate) struct ColumnMetadata {
    pub(crate) ident: Ident,
    pub(crate) name: String,
    pub(crate) table: String,
    pub(crate) schema: String,
    pub(crate) column_type: String,
    pub(crate) value: Value,
    pub(crate) nullable: bool,
    pub(crate) default: Option<String>,
    pub(crate) primary_key: PrimaryKeyType,
    pub(crate) unique: bool,
    pub(crate) auto_increment: bool,
    pub(crate) passive: bool,
    pub(crate) check_passive: Option<CheckPassive>,
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
    let mut metadata = ColumnMetadata {
        ident,
        name: field
            .ident
            .as_ref()
            .expect("Field is expected to have a name")
            .to_string()
            .into(),
        table: table_name(item).into(),
        schema: schema_name(item).into(),
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
        /*if meta.path().is_ident("default_value") {
            let Ok(v) = meta.require_list().and_then(|v| v.parse_args::<LitStr>()) else {
                panic!(
                    "Error while parsing `default_value`, use it like `#[default_value(SOME_EXPRESSION)]`",
                );
            };
            column_def.default = Some(v.value());
        } else */
        if meta.path().is_ident("column_name") {
            let Ok(v) = meta.require_list().and_then(|v| v.parse_args::<LitStr>()) else {
                panic!(
                    "Error while parsing `column_name`, use it like `#[column_name(\"my_column\")]`"
                );
            };
            metadata.name = v.value();
        } else if meta.path().is_ident("column_type") {
            let Ok(v) = meta.require_list().and_then(|v| v.parse_args::<LitStr>()) else {
                panic!(
                    "Error while parsing `column_type`, use it like `#[column_type(\"VARCHAR\")]`"
                );
            };
            metadata.column_type = v.value();
        } else if meta.path().is_ident("primary_key") {
            let Ok(..) = meta.require_path_only() else {
                panic!(
                    "Error while parsing `primary_key`, use it like `#[primary_key]` on a field"
                );
            };
            metadata.primary_key = PrimaryKeyType::PrimaryKey;
            metadata.nullable = false;
        } else if meta.path().is_ident("unique") {
            let Ok(..) = meta.require_path_only() else {
                panic!("Error while parsing `unique`, use it like `#[unique]` on a field");
            };
            metadata.unique = true;
        } else if meta.path().is_ident("auto_increment") {
            let Ok(..) = meta.require_path_only() else {
                panic!("Error while parsing `auto_increment`, use it like `#[auto_increment]` on a field");
            };
            metadata.auto_increment = true;
        }
    }
    metadata
}
