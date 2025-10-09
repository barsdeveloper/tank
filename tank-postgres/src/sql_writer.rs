use std::fmt::Write;
use tank_core::{Context, SqlWriter, Value};

pub struct PostgresSqlWriter {}

impl SqlWriter for PostgresSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_column_type(&self, context: Context, buff: &mut String, value: &Value) {
        match value {
            Value::Boolean(..) => buff.push_str("BOOLEAN"),
            Value::Int8(..) => buff.push_str("SMALLINT"),
            Value::Int16(..) => buff.push_str("SMALLINT"),
            Value::Int32(..) => buff.push_str("INTEGER"),
            Value::Int64(..) => buff.push_str("BIGINT"),
            Value::Int128(..) => buff.push_str("NUMERIC(38)"),
            Value::UInt8(..) => buff.push_str("SMALLINT"),
            Value::UInt16(..) => buff.push_str("INTEGER"),
            Value::UInt32(..) => buff.push_str("BIGINT"),
            Value::UInt64(..) => buff.push_str("NUMERIC(19)"),
            Value::UInt128(..) => buff.push_str("NUMERIC(38)"),
            Value::Float32(..) => buff.push_str("REAL"),
            Value::Float64(..) => buff.push_str("DOUBLE"),
            Value::Decimal(.., precision, scale) => {
                buff.push_str("NUMERIC");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(buff, "({},{})", precision, scale);
                }
            }
            Value::Char(..) => buff.push_str("CHARACTER(1)"),
            Value::Varchar(..) => buff.push_str("TEXT"),
            Value::Blob(..) => buff.push_str("BYTEA"),
            Value::Date(..) => buff.push_str("DATE"),
            Value::Time(..) => buff.push_str("TIME"),
            Value::Timestamp(..) => buff.push_str("TIMESTAMP"),
            Value::TimestampWithTimezone(..) => buff.push_str("TIMESTAMP WITH TIME ZONE"),
            Value::Interval(..) => buff.push_str("INTERVAL"),
            Value::Uuid(..) => buff.push_str("UUID"),
            Value::Array(.., inner, size) => {
                self.write_column_type(context, buff, inner);
                let _ = write!(buff, "[{}]", size);
            }
            Value::List(.., inner) => {
                self.write_column_type(context, buff, inner);
                buff.push_str("[]");
            }
            _ => log::error!(
                "Unexpected tank::Value, variant {:?} is not supported",
                value
            ),
        };
    }
}
