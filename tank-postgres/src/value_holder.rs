use crate::PostgresSqlWriter;
use postgres_types::Type;
use rust_decimal::Decimal;
use std::{error::Error, fmt::Write, io::Read};
use tank_core::{SqlWriter, Value};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use tokio_postgres::types::{FromSql, IsNull, ToSql, private::BytesMut};
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct ValueHolder(pub(crate) Value);

impl From<Value> for ValueHolder {
    fn from(value: Value) -> Self {
        ValueHolder(value)
    }
}

impl<'a> FromSql<'a> for ValueHolder {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Self::from_sql_nullable(ty, Some(raw))
    }
    fn from_sql_null(ty: &Type) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Self::from_sql_nullable(ty, None)
    }
    fn from_sql_nullable(
        ty: &Type,
        raw: Option<&'a [u8]>,
    ) -> Result<Self, Box<dyn Error + Sync + Send>> {
        macro_rules! to_value {
            ($ty_var:ident, $raw:ident, $($($ty:path)|+ => ( $value:path, $source:ty $(, $additional:expr)* ) ,)+) => {
                match *$ty_var {
                    $($($ty)|+ => $value(if let Some($raw) = $raw { Some(<$source>::from_sql($ty_var, $raw)?.into()) } else { None } $(, $additional)*),)+
                    _ => {
                        if let Some(mut raw) = $raw {
                            let mut buf = String::new();
                            let _ = raw.read_to_string(&mut buf);
                            return Err(tank_core::Error::msg(format!("Unknown value type {}: `{}`", $ty_var, buf)).into());
                        }
                        Value::Null
                    }
                }
            };
        }
        let value = to_value!(ty, raw,
            Type::BOOL => (Value::Boolean, bool),
            Type::INT2 => (Value::Int16, i16),
            Type::INT4 => (Value::Int32, i32),
            Type::INT8 => (Value::Int64, i64),
            Type::FLOAT4 => (Value::Float32, f32),
            Type::FLOAT8 => (Value::Float64, f64),
            Type::NUMERIC => (Value::Decimal, Decimal, 0, 0),
            Type::OID => (Value::UInt32, u32),
            Type::CHAR => (Value::Int8, i8),
            Type::VARCHAR | Type::TEXT | Type::NAME | Type::JSON | Type::XML => (Value::Varchar, String),
            Type::BYTEA => (Value::Blob, Vec<u8>),
            Type::DATE => (Value::Date, Date),
            Type::TIME => (Value::Time, Time),
            Type::TIMESTAMP => (Value::Timestamp, PrimitiveDateTime),
            Type::TIMESTAMPTZ => (Value::TimestampWithTimezone, OffsetDateTime),
            Type::UUID => (Value::Uuid, Uuid),
        );
        Ok(value.into())
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

impl ToSql for ValueHolder {
    fn to_sql(&self, _ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        let mut sql = String::new();
        PostgresSqlWriter {}.write_value(Default::default(), &mut sql, &self.0);
        out.write_str(&sql)?;
        Ok(if self.0.is_null() {
            IsNull::Yes
        } else {
            IsNull::No
        })
    }

    fn accepts(_ty: &Type) -> bool
    where
        Self: Sized,
    {
        todo!()
    }

    fn to_sql_checked(
        &self,
        _ty: &Type,
        _out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        todo!()
    }
}

pub fn value_to_type(value: &Value) -> Type {
    match value {
        Value::Null => Type::UNKNOWN,
        Value::Boolean(..) => Type::BOOL,
        Value::Int8(..) => Type::CHAR,
        Value::Int16(..) => Type::INT2,
        Value::Int32(..) => Type::INT4,
        Value::Int64(..) => Type::INT8,
        Value::Int128(..) => Type::UNKNOWN,
        Value::UInt8(..) => Type::INT2,
        Value::UInt16(..) => Type::INT4,
        Value::UInt32(..) => Type::INT8,
        Value::UInt64(..) => Type::INT8,
        Value::UInt128(..) => Type::UNKNOWN,
        Value::Float32(..) => Type::FLOAT4,
        Value::Float64(..) => Type::FLOAT8,
        Value::Decimal(..) => Type::NUMERIC,
        Value::Char(..) => Type::CHAR,
        Value::Varchar(..) => Type::TEXT,
        Value::Blob(..) => Type::BYTEA,
        Value::Date(..) => Type::DATE,
        Value::Time(..) => Type::TIME,
        Value::Timestamp(..) => Type::TIMESTAMP,
        Value::TimestampWithTimezone(..) => Type::TIMESTAMPTZ,
        Value::Interval(..) => Type::INTERVAL,
        Value::Uuid(..) => Type::UUID,
        Value::Array(.., ty, _) => match **ty {
            Value::Null => Type::UNKNOWN,
            Value::Boolean(..) => Type::BOOL_ARRAY,
            Value::Int8(..) => Type::INT2_ARRAY,
            Value::Int16(..) => Type::INT2_ARRAY,
            Value::Int32(..) => Type::INT4_ARRAY,
            Value::Int64(..) => Type::INT8_ARRAY,
            Value::Int128(..) => Type::UNKNOWN,
            Value::UInt8(..) => Type::INT2_ARRAY,
            Value::UInt16(..) => Type::INT4_ARRAY,
            Value::UInt32(..) => Type::INT8_ARRAY,
            Value::UInt64(..) => Type::INT8_ARRAY,
            Value::UInt128(..) => Type::UNKNOWN,
            Value::Float32(..) => Type::FLOAT4_ARRAY,
            Value::Float64(..) => Type::FLOAT8_ARRAY,
            Value::Decimal(..) => Type::NUMERIC_ARRAY,
            Value::Char(..) => Type::CHAR_ARRAY,
            Value::Varchar(..) => Type::TEXT_ARRAY,
            Value::Blob(..) => Type::BYTEA_ARRAY,
            Value::Date(..) => Type::DATE_ARRAY,
            Value::Time(..) => Type::TIME_ARRAY,
            Value::Timestamp(..) => Type::TIMESTAMP_ARRAY,
            Value::TimestampWithTimezone(..) => Type::TIMESTAMPTZ_ARRAY,
            Value::Interval(..) => Type::INTERVAL_ARRAY,
            Value::Uuid(..) => Type::UUID_ARRAY,
            _ => Type::ANYARRAY,
        },
        _ => Type::UNKNOWN,
    }
}
