use syn::{Field, LitStr, Meta, Type};
use tank_metadata::{decode_type, ColumnDef, Value};

pub fn decode_field(field: &Field) -> ColumnDef {
    let name = field.ident.as_ref().unwrap().to_string().into();
    let (value, nullable) = if let Type::Path(type_path) = &field.ty {
        decode_type(type_path)
    } else {
        (Value::Varchar(None), true)
    };
    let [default] = field.attrs.iter().fold([None], |mut acc, cur| {
        if cur.meta.path().is_ident("default") {
            if let Meta::List(v) = &cur.meta {
                let default = match v.parse_args::<LitStr>() {
                    Ok(lit_str) => lit_str.value(),
                    Err(e) => {
                        panic!(
                            "Error while parsing `default`: {}, use it like #[default(\"some\")]",
                            e
                        );
                    }
                };
                acc[0].replace(default);
            }
        }
        acc
    });
    ColumnDef {
        name,
        value,
        nullable,
        default,
        // unique,
        // comment,
    }
}
