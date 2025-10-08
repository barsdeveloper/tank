use std::fmt::Write;
use tank_core::{ColumnRef, Context, Entity, SqlWriter, TableRef, Value};

pub struct SqliteSqlWriter {}

impl SqlWriter for SqliteSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_column_ref(&self, context: Context, buff: &mut String, value: &ColumnRef) {
        if context.qualify_columns && !value.table.is_empty() {
            buff.push('"');
            if !value.schema.is_empty() {
                self.write_escaped(context, buff, &value.schema, '"', "\"\"");
                buff.push('.');
            }
            self.write_escaped(context, buff, &value.table, '"', "\"\"");
            buff.push_str("\".");
        }
        self.write_identifier_quoted(context, buff, &value.name);
    }

    fn write_table_ref(&self, context: Context, buff: &mut String, value: &TableRef) {
        if self.alias_declaration(context) || value.alias.is_empty() {
            buff.push('"');
            if !value.schema.is_empty() {
                self.write_escaped(context, buff, &value.schema, '"', "\"\"");
                buff.push('.');
            }
            self.write_escaped(context, buff, &value.name, '"', "\"\"");
            buff.push('"');
        }
        if !value.alias.is_empty() {
            let _ = write!(buff, " {}", value.alias);
        }
    }

    fn write_column_type(&self, _context: Context, buff: &mut String, value: &Value) {
        match value {
            Value::Boolean(..) => buff.push_str("INTEGER"),
            Value::Int8(..) => buff.push_str("INTEGER"),
            Value::Int16(..) => buff.push_str("INTEGER"),
            Value::Int32(..) => buff.push_str("INTEGER"),
            Value::Int64(..) => buff.push_str("INTEGER"),
            Value::UInt8(..) => buff.push_str("INTEGER"),
            Value::UInt16(..) => buff.push_str("INTEGER"),
            Value::UInt32(..) => buff.push_str("INTEGER"),
            Value::UInt64(..) => buff.push_str("INTEGER"),
            Value::Float32(..) => buff.push_str("REAL"),
            Value::Float64(..) => buff.push_str("REAL"),
            Value::Decimal(.., precision, scale) => {
                buff.push_str("REAL");
                if (precision, scale) != (&0, &0) {
                    let _ = write!(buff, "({},{})", precision, scale);
                }
            }
            Value::Char(..) => buff.push_str("TEXT"),
            Value::Varchar(..) => buff.push_str("TEXT"),
            Value::Blob(..) => buff.push_str("BLOB"),
            Value::Date(..) => buff.push_str("TEXT"),
            Value::Time(..) => buff.push_str("TEXT"),
            Value::Timestamp(..) => buff.push_str("TEXT"),
            Value::TimestampWithTimezone(..) => buff.push_str("TEXT"),
            Value::Uuid(..) => buff.push_str("TEXT"),
            _ => log::error!(
                "Unexpected tank::Value, Sqlite does not support {:?}",
                value
            ),
        };
    }

    fn write_value_infinity(&self, _context: Context, buff: &mut String, negative: bool) {
        if negative {
            buff.push('-');
        }
        buff.push_str("1.0e+10000");
    }

    fn write_value_blob(&self, _context: Context, buff: &mut String, value: &[u8]) {
        buff.push_str("X'");
        for b in value {
            let _ = write!(buff, "{:X}", b);
        }
        buff.push('\'');
    }

    fn write_create_schema<E>(&self, _buff: &mut String, _if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        // Sqlite does not support schema
    }

    fn write_drop_schema<E>(&self, _buff: &mut String, _if_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        // Sqlite does not support schema
    }

    fn write_column_comments<E>(&self, _context: Context, _buff: &mut String)
    where
        Self: Sized,
        E: Entity,
    {
    }
}
