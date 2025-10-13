use bytes::BytesMut;
use postgres_types::{FromSql, IsNull, ToSql, Type, to_sql_checked};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::{error::Error, io::Read};
use tank_core::Value;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
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
                            return Err(tank_core::Error::msg(format!("Cannot decode sql type: `{}`, value: `{}`", $ty_var, buf)).into());
                        }
                        Value::Null
                    }
                }
            };
        }
        let value = to_value!(ty, raw,
            Type::BOOL => (Value::Boolean, bool),
            Type::CHAR => (Value::Int8, i8),
            Type::INT2 => (Value::Int16, i16),
            Type::INT4 => (Value::Int32, i32),
            Type::INT8 => (Value::Int64, i64),
            Type::FLOAT4 => (Value::Float32, f32),
            Type::FLOAT8 => (Value::Float64, f64),
            Type::NUMERIC => (Value::Decimal, Decimal, 0, 0),
            Type::OID => (Value::UInt32, u32),
            Type::VARCHAR
            | Type::TEXT
            | Type::NAME
            | Type::BPCHAR
            | Type::JSON
            | Type::XML => (Value::Varchar, String),
            Type::BYTEA => (Value::Blob, Vec<u8>),
            Type::DATE => (Value::Date, Date),
            Type::TIME => (Value::Time, Time),
            Type::TIMESTAMP => (Value::Timestamp, PrimitiveDateTime),
            Type::TIMESTAMPTZ => (Value::TimestampWithTimezone, OffsetDateTime),
            Type::UUID => (Value::Uuid, Uuid),
            Type::INT2_ARRAY => (Value::List, VecWrap<ValueHolder>, Box::new(Value::Int16(None))),
            Type::INT4_ARRAY => (Value::List, VecWrap<ValueHolder>, Box::new(Value::Int32(None))),
            Type::INT8_ARRAY => (Value::List, VecWrap<ValueHolder>, Box::new(Value::Int64(None))),
            Type::FLOAT4_ARRAY => (Value::List, VecWrap<ValueHolder>, Box::new(Value::Float32(None))),
            Type::FLOAT8_ARRAY => (Value::List, VecWrap<ValueHolder>, Box::new(Value::Float64(None))),
            Type::BPCHAR_ARRAY => (Value::List, VecWrap<ValueHolder>, Box::new(Value::Varchar(None))),
            Type::UNKNOWN => (Value::Unknown, String),
        );
        Ok(value.into())
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

impl ToSql for ValueHolder {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized,
    {
        match &self.0 {
            Value::Null => None::<String>.to_sql(ty, out),
            Value::Boolean(v) => v.to_sql(ty, out),
            Value::Int8(v) => v.to_sql(ty, out),
            Value::Int16(v) => v.to_sql(ty, out),
            Value::Int32(v) => v.to_sql(ty, out),
            Value::Int64(v) => v.to_sql(ty, out),
            Value::Int128(v) => v.map(|v| Decimal::from_i128(v)).to_sql(ty, out),
            Value::UInt8(v) => v.map(|v| v as i16).to_sql(ty, out),
            Value::UInt16(v) => v.map(|v| v as i32).to_sql(ty, out),
            Value::UInt32(v) => v.to_sql(ty, out),
            Value::UInt64(v) => v.map(|v| Decimal::from_u64(v)).to_sql(ty, out),
            Value::UInt128(v) => v.map(|v| Decimal::from_u128(v)).to_sql(ty, out),
            Value::Float32(v) => v.to_sql(ty, out),
            Value::Float64(v) => v.to_sql(ty, out),
            Value::Decimal(v, _, _) => v.to_sql(ty, out),
            Value::Char(v) => v.map(|v| v.to_string()).to_sql(ty, out),
            Value::Varchar(v) => v.to_sql(ty, out),
            Value::Blob(v) => v.as_deref().to_sql(ty, out),
            Value::Date(v) => v.to_sql(ty, out),
            Value::Time(v) => v.to_sql(ty, out),
            Value::Timestamp(v) => v.to_sql(ty, out),
            Value::TimestampWithTimezone(v) => v.to_sql(ty, out),
            Value::Uuid(v) => v.to_sql(ty, out),
            Value::Array(v, ..) => v
                .as_ref()
                .map(|v| v.clone().into_iter().map(ValueHolder).collect::<Vec<_>>())
                .to_sql(ty, out),
            Value::List(v, ..) => v
                .as_ref()
                .map(|v| v.clone().into_iter().map(ValueHolder).collect::<Vec<_>>())
                .to_sql(ty, out),
            _ => {
                return Err(tank_core::Error::msg(format!(
                    "tank::Value variant `{:?}` is not supported by Postgres",
                    &self.0
                ))
                .into());
            }
        }
    }

    fn accepts(_ty: &Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    to_sql_checked!();
}
pub fn postgres_type_to_value(ty: &Type) -> Value {
    match *ty {
        Type::BOOL => Value::Boolean(None),
        Type::CHAR => Value::Int8(None),
        Type::INT2 => Value::Int16(None),
        Type::INT4 => Value::Int32(None),
        Type::INT8 => Value::Int64(None),
        Type::FLOAT4 => Value::Float32(None),
        Type::FLOAT8 => Value::Float64(None),
        Type::NUMERIC => Value::Decimal(None, 0, 0),
        Type::VARCHAR | Type::TEXT | Type::BPCHAR | Type::JSON | Type::XML => Value::Varchar(None),
        Type::BYTEA => Value::Blob(None),
        Type::DATE => Value::Date(None),
        Type::TIME => Value::Time(None),
        Type::TIMESTAMP => Value::Timestamp(None),
        Type::TIMESTAMPTZ => Value::TimestampWithTimezone(None),
        Type::INTERVAL => Value::Interval(None),
        Type::UUID => Value::Uuid(None),
        Type::BOOL_ARRAY => Value::List(None, Box::new(Value::Boolean(None))),
        Type::INT2_ARRAY => Value::List(None, Box::new(Value::Int16(None))),
        Type::INT4_ARRAY => Value::List(None, Box::new(Value::Int32(None))),
        Type::INT8_ARRAY => Value::List(None, Box::new(Value::Int64(None))),
        Type::FLOAT4_ARRAY => Value::List(None, Box::new(Value::Float32(None))),
        Type::FLOAT8_ARRAY => Value::List(None, Box::new(Value::Float64(None))),
        Type::NUMERIC_ARRAY => Value::List(None, Box::new(Value::Decimal(None, 0, 0))),
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY => Value::List(None, Box::new(Value::Varchar(None))),
        Type::BYTEA_ARRAY => Value::List(None, Box::new(Value::Blob(None))),
        Type::DATE_ARRAY => Value::List(None, Box::new(Value::Date(None))),
        Type::TIME_ARRAY => Value::List(None, Box::new(Value::Time(None))),
        Type::TIMESTAMP_ARRAY => Value::List(None, Box::new(Value::Timestamp(None))),
        Type::TIMESTAMPTZ_ARRAY => Value::List(None, Box::new(Value::TimestampWithTimezone(None))),
        Type::INTERVAL_ARRAY => Value::List(None, Box::new(Value::Interval(None))),
        Type::UUID_ARRAY => Value::List(None, Box::new(Value::Uuid(None))),
        _ => Value::Null,
    }
}

struct VecWrap<T>(pub Vec<T>);

impl<'a, T: FromSql<'a>> FromSql<'a> for VecWrap<T> {
    fn from_sql_null(ty: &Type) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Vec::<T>::from_sql_null(ty).map(VecWrap)
    }
    fn from_sql_nullable(
        ty: &Type,
        raw: Option<&'a [u8]>,
    ) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Vec::<T>::from_sql_nullable(ty, raw).map(VecWrap)
    }
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Vec::<T>::from_sql(ty, raw).map(VecWrap)
    }
    fn accepts(ty: &Type) -> bool {
        Vec::<T>::accepts(ty)
    }
}

impl From<VecWrap<ValueHolder>> for Vec<Value> {
    fn from(value: VecWrap<ValueHolder>) -> Self {
        value.0.into_iter().map(|v| v.0).collect()
    }
}
