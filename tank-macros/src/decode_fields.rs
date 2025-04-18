use syn::{Field, ItemStruct, LitStr, Type};
use tank_core::{decode_type, ColumnDef, Value};

use crate::{schema_name, table_name};

pub fn decode_field(field: &Field, item: &ItemStruct) -> ColumnDef {
    let (value, nullable) = if let Type::Path(type_path) = &field.ty {
        decode_type(&type_path.path)
    } else {
        (Value::Varchar(None), true)
    };
    let mut result = ColumnDef {
        name: field.ident.as_ref().unwrap().to_string().into(),
        table_name: table_name(item).into(),
        schema_name: schema_name(item).into(),
        value,
        nullable,
        ..Default::default()
    };
    for attr in &field.attrs {
        let meta = &attr.meta;
        if meta.path().is_ident("default_value") {
            let Ok(v) = meta.require_list().and_then(|v| v.parse_args::<LitStr>()) else {
                panic!(
                    "Error while parsing `default_value`, use it like #[default_value(\"some\")]",
                );
            };
            result.default = Some(v.value());
        } else if meta.path().is_ident("column_type") {
            let Ok(v) = meta.require_list().and_then(|v| v.parse_args::<LitStr>()) else {
                panic!(
                    "Error while parsing `column_type`, use it like #[column_type(\"VARCHAR\")]"
                );
            };
            result.column_type = v.value();
        } else if meta.path().is_ident("primary_key") {
            let Ok(v) = meta.require_path_only() else {
                panic!("Error while parsing `primary_key`, use it like #[primary_key]");
            };
            result.primary_key = true;
        } else if meta.path().is_ident("unique") {
            let Ok(v) = meta.require_path_only() else {
                panic!("Error while parsing `unique`, use it like #[unique]");
            };
            result.unique = true;
        }
    }
    result
}
