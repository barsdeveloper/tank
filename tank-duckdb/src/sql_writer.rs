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

    fn sql_value<'a>(&self, out: &'a mut String, value: &Value) -> &'a mut String {
        let generic_writer = GenericSqlWriter::new();
        let _ = match value {
            Value::Blob(Some(v), ..) => {
                out.push('\'');
                v.iter().for_each(|b| {
                    let _ = write!(out, "\\x{:X}", b);
                });
                out.push('\'');
            }
            Value::Map(Some(v), ..) => {
                out.push_str("MAP");
                generic_writer.sql_value(out, value);
            }
            _ => {
                generic_writer.sql_value(out, value);
            }
        };
        out
    }
}
