use std::fmt::Write;
use tank_core::{GenericSqlWriter, Interval, SqlWriter, Value};

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

    fn value_interval_units(&self) -> &[(&str, i128)] {
        static UNITS: &[(&str, i128)] = &[
            ("DAY", Interval::NANOS_IN_DAY),
            ("HOUR", Interval::NANOS_IN_SEC * 3600),
            ("MINUTE", Interval::NANOS_IN_SEC * 60),
            ("SECOND", Interval::NANOS_IN_SEC),
            ("MICROSECOND", 1_000),
        ];
        UNITS
    }

    fn write_value(&self, out: &mut String, value: &Value) {
        let generic_writer = GenericSqlWriter::new();
        let _ = match value {
            Value::Blob(Some(v), ..) => {
                out.push('\'');
                for b in v.iter() {
                    let _ = write!(out, "\\x{:X}", b);
                }
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
