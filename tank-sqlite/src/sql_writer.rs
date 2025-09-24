use std::fmt::Write;
use tank_core::{ColumnRef, Context, Entity, SqlWriter, TableRef, Value};

pub struct SqliteSqlWriter {}

impl SqlWriter for SqliteSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_column_ref(&self, context: Context, buff: &mut dyn Write, value: &ColumnRef) {
        if context.qualify_columns && !value.table.is_empty() {
            let _ = buff.write_char('"');
            if !value.schema.is_empty() {
                self.write_escaped(context, buff, &value.schema, '"', "\"\"");
                let _ = buff.write_char('.');
            }
            self.write_escaped(context, buff, &value.table, '"', "\"\"");
            let _ = buff.write_str("\".");
        }
        self.write_identifier_quoted(context, buff, &value.name);
    }

    fn write_table_ref(&self, context: Context, buff: &mut dyn Write, value: &TableRef) {
        if self.alias_declaration(context) || value.alias.is_empty() {
            let _ = buff.write_char('"');
            if !value.schema.is_empty() {
                self.write_escaped(context, buff, &value.schema, '"', "\"\"");
                let _ = buff.write_char('.');
            }
            self.write_escaped(context, buff, &value.name, '"', "\"\"");
            let _ = buff.write_char('"');
        }
        if !value.alias.is_empty() {
            let _ = write!(buff, " {}", value.alias);
        }
    }

    fn write_column_type(&self, _context: Context, buff: &mut dyn Write, value: &Value) {
        let _ = match value {
            Value::Boolean(..) => buff.write_str("INTEGER"),
            Value::Int8(..) => buff.write_str("INTEGER"),
            Value::Int16(..) => buff.write_str("INTEGER"),
            Value::Int32(..) => buff.write_str("INTEGER"),
            Value::Int64(..) => buff.write_str("INTEGER"),
            Value::UInt8(..) => buff.write_str("INTEGER"),
            Value::UInt16(..) => buff.write_str("INTEGER"),
            Value::UInt32(..) => buff.write_str("INTEGER"),
            Value::UInt64(..) => buff.write_str("INTEGER"),
            Value::Float32(..) => buff.write_str("REAL"),
            Value::Float64(..) => buff.write_str("REAL"),
            Value::Decimal(.., precision, scale) => {
                let _ = buff.write_str("REAL");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(buff, "({},{})", precision, scale);
                }
                Ok(())
            }
            Value::Char(..) => buff.write_str("TEXT"),
            Value::Varchar(..) => buff.write_str("TEXT"),
            Value::Blob(..) => buff.write_str("BLOB"),
            Value::Date(..) => buff.write_str("TEXT"),
            Value::Time(..) => buff.write_str("TEXT"),
            Value::Timestamp(..) => buff.write_str("TEXT"),
            Value::TimestampWithTimezone(..) => buff.write_str("TEXT"),
            Value::Uuid(..) => buff.write_str("TEXT"),
            _ => panic!(
                "Unexpected tank::Value, cannot get the sql type from {:?} variant",
                value
            ),
        };
    }

    fn write_value_infinity(&self, _context: Context, buff: &mut dyn Write, negative: bool) {
        if negative {
            let _ = buff.write_char('-');
        }
        let _ = buff.write_str("1.0e+10000");
    }

    fn write_value_blob(&self, _context: Context, buff: &mut dyn Write, value: &[u8]) {
        let _ = buff.write_str("X'");
        for b in value {
            let _ = write!(buff, "{:X}", b);
        }
        let _ = buff.write_char('\'');
    }

    fn write_create_schema<E, Buff>(&self, _out: &mut Buff, _if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
        Buff: Write,
    {
        // Sqlite does not support schema
    }

    fn write_drop_schema<E, Buff>(&self, _out: &mut Buff, _if_exists: bool)
    where
        Self: Sized,
        E: Entity,
        Buff: Write,
    {
        // Sqlite does not support schema
    }
}
