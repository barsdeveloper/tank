use crate::Entity;
use std::fmt::Write;
use tank_metadata::{ColumnDef, Value};

pub trait SqlWriter {
    fn sql_create_table<'a, E: Entity>(
        &self,
        query: &'a mut String,
        if_not_exists: bool,
    ) -> &'a mut String {
        query.push_str("CREATE TABLE ");
        if if_not_exists {
            query.push_str("IF NOT EXISTS ");
        }
        query.push_str(E::table_name());
        query.push('(');
        let mut first = true;
        E::columns().iter().for_each(|c| {
            if !first {
                query.push_str(", ");
            }
            self.sql_create_table_column_fragment(query, c);
            first = false;
        });
        query.push(')');
        query
    }

    fn sql_create_table_column_fragment<'a>(
        &self,
        query: &'a mut String,
        column: &ColumnDef,
    ) -> &'a mut String {
        query.push_str(&column.name);
        query.push(' ');
        self.sql_type(query, &column.value);
        if !column.nullable {
            query.push_str(" NOT NULL");
        }
        query
    }

    fn sql_type<'a>(&self, query: &'a mut String, value: &Value) -> &'a mut String {
        match value {
            Value::Boolean(..) => query.push_str("BOOLEAN"),
            Value::Int8(..) => query.push_str("TINYINT"),
            Value::Int16(..) => query.push_str("SMALLINT"),
            Value::Int32(..) => query.push_str("INTEGER"),
            Value::Int64(..) => query.push_str("BIGINT"),
            Value::Int128(..) => query.push_str("HUGEINT"),
            Value::UInt8(..) => query.push_str("UTINYINT"),
            Value::UInt16(..) => query.push_str("USMALLINT"),
            Value::UInt32(..) => query.push_str("UINTEGER"),
            Value::UInt64(..) => query.push_str("UBIGINT"),
            Value::UInt128(..) => query.push_str("UHUGEINT"),
            Value::Float32(..) => query.push_str("FLOAT"),
            Value::Float64(..) => query.push_str("DOUBLE"),
            Value::Decimal(.., precision, scale) => {
                write!(query, "DECIMAL({}, {})", precision, scale).unwrap();
            }
            Value::Varchar(..) => query.push_str("VARCHAR"),
            Value::Blob(..) => query.push_str("BLOB"),
            Value::Date(..) => query.push_str("DATE"),
            Value::Time(..) => query.push_str("TIME"),
            Value::Timestamp(..) => query.push_str("TIMESTAMP"),
            Value::TimestampWithTimezone(..) => query.push_str("TIMESTAMP WITH TIME ZONE"),
            Value::Interval(..) => query.push_str("INTERVAL"),
            Value::Uuid(..) => query.push_str("UUID"),
            Value::Array(.., inner, size) => {
                self.sql_type(query, inner);
                write!(query, "[{}]", size).unwrap();
            }
            Value::List(.., inner) => {
                self.sql_type(query, inner);
                query.push_str("[]");
            }
            Value::Map(.., key, value) => {
                query.push_str("MAP(");
                self.sql_type(query, key);
                query.push_str(", ");
                self.sql_type(query, value);
                query.push(')');
            }
        };
        query
    }
}
