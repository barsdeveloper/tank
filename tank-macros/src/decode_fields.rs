use crate::{schema_name, table_name};
use std::borrow::Cow;
use syn::{Field, ItemStruct, LitStr, Type};
use tank_core::{decode_type, CheckPassive, ColumnDef, ColumnRef, TypeDecoded};

pub fn decode_field(field: &Field, item: &ItemStruct) -> (ColumnDef, Option<CheckPassive>) {
    let (
        TypeDecoded {
            value,
            nullable,
            passive,
        },
        filter_passive,
    ) = if let Type::Path(..) = &field.ty {
        decode_type(&field.ty)
    } else {
        Default::default()
    };
    let mut column_def = ColumnDef {
        reference: ColumnRef {
            name: field
                .ident
                .as_ref()
                .expect("Field is expected to have a name")
                .to_string()
                .into(),
            table: table_name(item).into(),
            schema: schema_name(item).into(),
        },
        value,
        nullable,
        passive,
        ..Default::default()
    };
    if column_def.reference.name.starts_with('_') {
        match column_def.reference.name {
            Cow::Borrowed(v) => {
                column_def.reference.name = Cow::Borrowed(&v[1..]);
            }
            Cow::Owned(ref mut v) => {
                v.remove(0);
            }
        }
    }
    for attr in &field.attrs {
        let meta = &attr.meta;
        if meta.path().is_ident("default_value") {
            let Ok(v) = meta.require_list().and_then(|v| v.parse_args::<LitStr>()) else {
                panic!(
                    "Error while parsing `default_value`, use it like `#[default_value(\"some\")]`",
                );
            };
            column_def.default = Some(v.value());
        } else if meta.path().is_ident("column_name") {
            let Ok(v) = meta.require_list().and_then(|v| v.parse_args::<LitStr>()) else {
                panic!(
                    "Error while parsing `column_name`, use it like `#[column_name(\"my_column\")]`"
                );
            };
            column_def.reference.name = v.value().into();
        } else if meta.path().is_ident("column_type") {
            let Ok(v) = meta.require_list().and_then(|v| v.parse_args::<LitStr>()) else {
                panic!(
                    "Error while parsing `column_type`, use it like `#[column_type(\"VARCHAR\")]`"
                );
            };
            column_def.column_type = v.value().into();
        } else if meta.path().is_ident("primary_key") {
            let Ok(..) = meta.require_path_only() else {
                panic!(
                    "Error while parsing `primary_key`, use it like `#[primary_key]` on a field"
                );
            };
            column_def.primary_key = true;
            column_def.nullable = false;
        } else if meta.path().is_ident("unique") {
            let Ok(..) = meta.require_path_only() else {
                panic!("Error while parsing `unique`, use it like `#[unique]` on a field");
            };
            column_def.unique = true;
        } else if meta.path().is_ident("auto_increment") {
            let Ok(..) = meta.require_path_only() else {
                panic!("Error while parsing `auto_increment`, use it like `#[auto_increment]` on a field");
            };
            column_def.auto_increment = true;
        }
    }
    (column_def, filter_passive)
}
