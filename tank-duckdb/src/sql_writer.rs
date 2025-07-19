use std::fmt::Write;
use tank_core::{GenericSqlWriter, SqlWriter, Value};

#[derive(Default)]
pub struct DuckDBSqlWriter {}

impl DuckDBSqlWriter {
    pub const fn new() -> Self {
        Self {}
    }
}

impl SqlWriter for DuckDBSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_value(&self, out: &mut String, value: &Value) {
        let generic_writer = GenericSqlWriter::new();
        let _ = match value {
            Value::Blob(Some(v), ..) => {
                out.push('\'');
                v.iter().for_each(|b| {
                    let _ = write!(out, "\\x{:X}", b);
                });
                out.push('\'');
            }
            Value::Map(Some(_v), ..) => {
                out.push_str("MAP");
                generic_writer.write_value(out, value);
            }
            _ => {
                generic_writer.write_value(out, value);
            }
        };
    }
}
