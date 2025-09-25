use std::{collections::HashMap, fmt::Write};
use tank_core::{Context, Interval, SqlWriter, Value, separated_by};

#[derive(Default)]
pub struct DuckDBSqlWriter {}

impl SqlWriter for DuckDBSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_value_blob(&self, _context: Context, buff: &mut String, value: &[u8]) {
        buff.push('\'');
        for b in value {
            let _ = write!(buff, "\\x{:X}", b);
        }
        buff.push('\'');
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

    fn write_value_map(&self, context: Context, buff: &mut String, value: &HashMap<Value, Value>) {
        buff.push_str("MAP{");
        separated_by(
            buff,
            value,
            |buff, (k, v)| {
                self.write_value(context, buff, k);
                buff.push(':');
                self.write_value(context, buff, v);
            },
            ",",
        );
        buff.push('}');
    }
}
