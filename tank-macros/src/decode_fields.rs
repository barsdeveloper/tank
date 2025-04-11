use syn::{Expr, ExprLit, Field, Lit, Meta, Type};
use tank_metadata::{decode_type, ColumnDef, Value};

pub fn decode_field(field: &Field) -> ColumnDef {
    let name = field.ident.as_ref().unwrap().to_string().into();
    let (default, column_type) = field.attrs.iter().fold((None, None), |mut acc, cur| {
        let meta = &cur.meta;
        if meta.path().is_ident("default") {
            if let Meta::NameValue(v) = &meta {
                let default = match &v.value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(v), ..
                    }) => v.value(),
                    _ => {
                        panic!("Error while parsing `default`, use it like #[default=\"some\"]",);
                    }
                };
                acc.0.replace(default);
            }
        } else if meta.path().is_ident("column_type") {
            if let Meta::NameValue(v) = &meta {
                let column_type = match &v.value {
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(v), ..
                    }) => v.value(),
                    _ => {
                        panic!("Error while parsing `type`, use it like #[type=\"VARCHAR\"]",);
                    }
                };
                acc.1.replace(column_type);
            }
        }
        acc
    });
    let (value, nullable) = if let Type::Path(type_path) = &field.ty {
        decode_type(&type_path.path)
    } else {
        (Value::Varchar(None), true)
    };
    ColumnDef {
        name,
        value,
        nullable,
        default,
        // unique,
        // comment,
        column_type: column_type.unwrap_or("".to_string()).into(),
    }
}
