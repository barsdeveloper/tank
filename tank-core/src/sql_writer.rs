use crate::Entity;
use std::fmt::Write;
use tank_metadata::{ColumnDef, Value};

pub trait SqlWriter {
    fn sql_create_table<'a, E: Entity>(
        &self,
        out: &'a mut String,
        if_not_exists: bool,
    ) -> &'a mut String {
        out.push_str("CREATE TABLE ");
        if if_not_exists {
            out.push_str("IF NOT EXISTS ");
        }
        out.push_str(E::table_name());
        out.push('(');
        let mut first = true;
        E::columns().iter().for_each(|c| {
            if !first {
                out.push_str(", ");
            }
            self.sql_create_table_column_fragment(out, c);
            first = false;
        });
        out.push(')');
        out
    }

    fn sql_create_table_column_fragment<'a>(
        &self,
        out: &'a mut String,
        column: &ColumnDef,
    ) -> &'a mut String {
        out.push_str(&column.name);
        out.push(' ');
        if !column.column_type.is_empty() {
            out.push_str(&column.column_type);
        } else {
            self.sql_type(out, &column.value);
        }
        if !column.nullable {
            out.push_str(" NOT NULL");
        }
        out
    }

    fn sql_drop_table<E: Entity>(&self, query: &mut String, if_exists: bool) {
        query.push_str("DROP TABLE ");
        if if_exists {
            query.push_str("IF EXISTS ");
        }
        query.push_str(E::table_name());
    }

    fn sql_type<'a>(&self, out: &'a mut String, value: &Value) -> &'a mut String {
        match value {
            Value::Boolean(..) => out.push_str("BOOLEAN"),
            Value::Int8(..) => out.push_str("TINYINT"),
            Value::Int16(..) => out.push_str("SMALLINT"),
            Value::Int32(..) => out.push_str("INTEGER"),
            Value::Int64(..) => out.push_str("BIGINT"),
            Value::Int128(..) => out.push_str("HUGEINT"),
            Value::UInt8(..) => out.push_str("UTINYINT"),
            Value::UInt16(..) => out.push_str("USMALLINT"),
            Value::UInt32(..) => out.push_str("UINTEGER"),
            Value::UInt64(..) => out.push_str("UBIGINT"),
            Value::UInt128(..) => out.push_str("UHUGEINT"),
            Value::Float32(..) => out.push_str("FLOAT"),
            Value::Float64(..) => out.push_str("DOUBLE"),
            Value::Decimal(.., precision, scale) => {
                out.push_str("DECIMAL");
                if (precision, scale) != (&0, &0) {
                    write!(out, "({}, {})", precision, scale).unwrap();
                }
            }
            Value::Varchar(..) => out.push_str("VARCHAR"),
            Value::Blob(..) => out.push_str("BLOB"),
            Value::Date(..) => out.push_str("DATE"),
            Value::Time(..) => out.push_str("TIME"),
            Value::Timestamp(..) => out.push_str("TIMESTAMP"),
            Value::TimestampWithTimezone(..) => out.push_str("TIMESTAMP WITH TIME ZONE"),
            Value::Interval(..) => out.push_str("INTERVAL"),
            Value::Uuid(..) => out.push_str("UUID"),
            Value::Array(.., inner, size) => {
                self.sql_type(out, inner);
                write!(out, "[{}]", size).unwrap();
            }
            Value::List(.., inner) => {
                self.sql_type(out, inner);
                out.push_str("[]");
            }
            Value::Map(.., key, value) => {
                out.push_str("MAP(");
                self.sql_type(out, key);
                out.push_str(", ");
                self.sql_type(out, value);
                out.push(')');
            }
            _ => panic!("Unexpected tank::Value, cannot get the sql type"),
        };
        out
    }

    fn sql_select<'a, E: Entity>(
        &self,
        out: &'a mut String,
        condition: &[Value],
        limit: u32,
    ) -> &'a mut String {
        out.push_str("SELECT * FROM ");
        out.push_str(E::table_name());
        out.push_str(" WHERE ");
        out
    }
}
