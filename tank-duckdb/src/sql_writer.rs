use std::{collections::HashMap, fmt::Write};
use tank_core::{Context, Interval, SqlWriter, Value, separated_by};

#[derive(Default)]
pub struct DuckDBSqlWriter {}

impl SqlWriter for DuckDBSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_value_blob(&self, _context: Context, out: &mut dyn Write, value: &[u8]) {
        let _ = out.write_char('\'');
        for b in value {
            let _ = write!(out, "\\x{:X}", b);
        }
        let _ = out.write_char('\'');
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

    fn write_value_map(
        &self,
        context: Context,
        out: &mut dyn Write,
        value: &HashMap<Value, Value>,
    ) {
        let _ = out.write_str("MAP{");
        separated_by(
            out,
            value,
            |out, (k, v)| {
                self.write_value(context, out, k);
                let _ = out.write_char(':');
                self.write_value(context, out, v);
                true
            },
            ",",
        );
        let _ = out.write_char('}');
    }
}
