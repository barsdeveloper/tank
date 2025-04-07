use metadata::{decode_type, ColumnDef, Value};
use syn::{Field, Type};

pub fn decode_field(field: &Field) -> ColumnDef {
    let name = field.ident.as_ref().unwrap().to_string().into();
    let (value, nullable) = if let Type::Path(type_path) = &field.ty {
        decode_type(type_path)
    } else {
        (Value::Varchar(None), true)
    };
    ColumnDef {
        name,
        value,
        nullable,
        // default,
        // unique,
        // comment,
    }
}
