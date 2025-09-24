use std::fmt::Write;
use tank_core::{ColumnRef, Context, Entity, SqlWriter, TableRef, Value};

pub struct SqliteSqlWriter {}

impl SqlWriter for SqliteSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_column_ref(&self, context: Context, out: &mut dyn Write, value: &ColumnRef) {
        if context.qualify_columns && !value.table.is_empty() {
            let _ = out.write_char('"');
            if !value.schema.is_empty() {
                self.write_escaped(context, out, &value.schema, '"', "\"\"");
                let _ = out.write_char('.');
            }
            self.write_escaped(context, out, &value.table, '"', "\"\"");
            let _ = out.write_str("\".");
        }
        self.write_identifier_quoted(context, out, &value.name);
    }

    fn write_table_ref(&self, context: Context, out: &mut dyn Write, value: &TableRef) {
        if self.alias_declaration(context) || value.alias.is_empty() {
            let _ = out.write_char('"');
            if !value.schema.is_empty() {
                self.write_escaped(context, out, &value.schema, '"', "\"\"");
                let _ = out.write_char('.');
            }
            self.write_escaped(context, out, &value.name, '"', "\"\"");
            let _ = out.write_char('"');
        }
        if !value.alias.is_empty() {
            let _ = write!(out, " {}", value.alias);
        }
    }

    fn write_column_type(&self, _context: Context, out: &mut dyn Write, value: &Value) {
        let _ = match value {
            Value::Boolean(..) => out.write_str("INTEGER"),
            Value::Int8(..) => out.write_str("INTEGER"),
            Value::Int16(..) => out.write_str("INTEGER"),
            Value::Int32(..) => out.write_str("INTEGER"),
            Value::Int64(..) => out.write_str("INTEGER"),
            Value::UInt8(..) => out.write_str("INTEGER"),
            Value::UInt16(..) => out.write_str("INTEGER"),
            Value::UInt32(..) => out.write_str("INTEGER"),
            Value::UInt64(..) => out.write_str("INTEGER"),
            Value::Float32(..) => out.write_str("REAL"),
            Value::Float64(..) => out.write_str("REAL"),
            Value::Decimal(.., precision, scale) => {
                let _ = out.write_str("REAL");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(out, "({},{})", precision, scale);
                }
                Ok(())
            }
            Value::Char(..) => out.write_str("TEXT"),
            Value::Varchar(..) => out.write_str("TEXT"),
            Value::Blob(..) => out.write_str("BLOB"),
            Value::Date(..) => out.write_str("TEXT"),
            Value::Time(..) => out.write_str("TEXT"),
            Value::Timestamp(..) => out.write_str("TEXT"),
            Value::TimestampWithTimezone(..) => out.write_str("TEXT"),
            Value::Uuid(..) => out.write_str("TEXT"),
            _ => panic!(
                "Unexpected tank::Value, cannot get the sql type from {:?} variant",
                value
            ),
        };
    }

    fn write_value_infinity(&self, _context: Context, out: &mut dyn Write, negative: bool) {
        if negative {
            let _ = out.write_char('-');
        }
        let _ = out.write_str("1.0e+10000");
    }

    fn write_value_blob(&self, _context: Context, out: &mut dyn Write, value: &[u8]) {
        let _ = out.write_str("X'");
        for b in value {
            let _ = write!(out, "{:X}", b);
        }
        let _ = out.write_char('\'');
    }

    fn write_create_schema<E>(&self, _out: &mut dyn Write, _if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        // Sqlite does not support schema
    }

    fn write_drop_schema<E>(&self, _out: &mut dyn Write, _if_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        // Sqlite does not support schema
    }
}
