use std::fmt::Write;
use tank_core::{Entity, SqlWriter, TableRef, Value};

pub struct SqliteSqlWriter {}

impl SqlWriter for SqliteSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_table_ref(&self, out: &mut String, value: &TableRef, is_declaration: bool) {
        if !is_declaration && !value.alias.is_empty() {
            out.push_str(&value.alias);
        } else {
            out.push('"');
            if !value.schema.is_empty() {
                self.write_escaped(out, &value.schema, '"', r#""""#);
                out.push('.');
            }
            self.write_escaped(out, &value.name, '"', r#""""#);
            out.push('"');
        }
        if is_declaration {
            out.push(' ');
            out.push_str(&value.alias);
        }
    }

    fn write_column_type(&self, out: &mut String, value: &Value) {
        match value {
            Value::Boolean(..) => out.push_str("INTEGER"),
            Value::Int8(..) => out.push_str("INTEGER"),
            Value::Int16(..) => out.push_str("INTEGER"),
            Value::Int32(..) => out.push_str("INTEGER"),
            Value::Int64(..) => out.push_str("INTEGER"),
            Value::UInt8(..) => out.push_str("INTEGER"),
            Value::UInt16(..) => out.push_str("INTEGER"),
            Value::UInt32(..) => out.push_str("INTEGER"),
            Value::UInt64(..) => out.push_str("INTEGER"),
            Value::Float32(..) => out.push_str("REAL"),
            Value::Float64(..) => out.push_str("REAL"),
            Value::Decimal(.., precision, scale) => {
                out.push_str("REAL");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(out, "({},{})", precision, scale);
                }
            }
            Value::Char(..) => out.push_str("TEXT"),
            Value::Varchar(..) => out.push_str("TEXT"),
            Value::Blob(..) => out.push_str("BLOB"),
            Value::Date(..) => out.push_str("TEXT"),
            Value::Time(..) => out.push_str("TEXT"),
            Value::Timestamp(..) => out.push_str("TEXT"),
            Value::TimestampWithTimezone(..) => out.push_str("TEXT"),
            Value::Uuid(..) => out.push_str("TEXT"),
            _ => panic!(
                "Unexpected tank::Value, cannot get the sql type from {:?} variant",
                value
            ),
        };
    }

    fn write_value_blob(&self, out: &mut String, value: &[u8]) {
        out.push_str("X'");
        for b in value {
            let _ = write!(out, "{:X}", b);
        }
        out.push('\'');
    }

    fn write_create_schema<E>(&self, _out: &mut String, _if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        // Sqlite does not support schema
    }

    fn write_drop_schema<E>(&self, _out: &mut String, _if_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        // Sqlite does not support schema
    }
}
