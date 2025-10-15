use crate::{Error, FixedDecimal, Interval, Parse, Passive, Result, Value};
use atoi::FromRadix10;
use quote::ToTokens;
use rust_decimal::{Decimal, prelude::FromPrimitive, prelude::ToPrimitive};
use std::{
    any,
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, LinkedList, VecDeque},
    hash::Hash,
    rc::Rc,
    sync::{Arc, RwLock},
};

pub trait AsValue {
    fn as_empty_value() -> Value;
    fn as_value(self) -> Value;
    fn try_from_value(value: Value) -> Result<Self>
    where
        Self: Sized;
}

impl<T: AsValue> From<T> for Value {
    fn from(value: T) -> Self {
        value.as_value()
    }
}

impl From<&'static str> for Value {
    fn from(value: &'static str) -> Self {
        Value::Varchar(Some(value.into()))
    }
}

macro_rules! impl_as_value {
    ($source:ty, $value:path => $expr:expr $(, $pat_rest:path => $expr_rest:expr)* $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $value(None)
            }
            fn as_value(self) -> Value {
                $value(Some(self.into()))
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    $value(Some(v), ..) => $expr(v),
                    $($pat_rest(Some(v), ..) => $expr_rest(v),)*
                    #[allow(unreachable_patterns)]
                    Value::Int64(Some(v), ..) => {
                        if (v as i128).clamp(<$source>::MIN as i128, <$source>::MAX as i128) != v as i128 {
                            return Err(Error::msg(format!(
                                "Value {}: i64 does not fit into {}",
                                v,
                                stringify!($source),
                            )));
                        }
                        Ok(v as $source)
                    }
                    Value::Unknown(Some(ref v), ..) => {
                        let (r, len) = <$source>::from_radix_10(v.as_bytes());
                        if len != v.len() {
                            return Err(Error::msg(format!(
                                "Cannot decode {:?} into {}",
                                value,
                                stringify!($source),
                            )))
                        }
                        Ok(r)
                    }
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        stringify!($source),
                    ))),
                }
            }
        }
    };
}
impl_as_value!(
    i8,
    Value::Int8 => |v| Ok(v),
    Value::UInt8 => |v| Ok(v as i8),
    Value::Int16 => |v: i16| {
        let result = v as i8;
        if result as i16 != v {
            return Err(Error::msg(format!("Value {}: i64 does not fit into i8", v)));
        }
        Ok(result)
    },
);
impl_as_value!(
    i16,
    Value::Int16 => |v| Ok(v),
    Value::Int8 => |v| Ok(v as i16),
    Value::UInt16 => |v| Ok(v as i16),
    Value::UInt8 => |v| Ok(v as i16),
);
impl_as_value!(
    i32,
    Value::Int32 => |v| Ok(v),
    Value::Int16 => |v| Ok(v as i32),
    Value::Int8 => |v| Ok(v as i32),
    Value::UInt32 => |v| Ok(v as i32),
    Value::UInt16 => |v| Ok(v as i32),
    Value::UInt8 => |v| Ok(v as i32),
    Value::Decimal => |v: Decimal| {
        let error = Error::msg(format!("Value {}: Decimal does not fit into i32", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_i32().ok_or(error)
    }
);
impl_as_value!(
    i64,
    Value::Int64 => |v| Ok(v),
    Value::Int32 => |v| Ok(v as i64),
    Value::Int16 => |v| Ok(v as i64),
    Value::Int8 => |v| Ok(v as i64),
    Value::UInt64 => |v| Ok(v as i64),
    Value::UInt32 => |v| Ok(v as i64),
    Value::UInt16 => |v| Ok(v as i64),
    Value::UInt8 => |v| Ok(v as i64),
    Value::Decimal => |v: Decimal| {
        let error = Error::msg(format!("Value {}: Decimal does not fit into i64", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_i64().ok_or(error)
    }
);
impl_as_value!(
    i128,
    Value::Int128 => |v| Ok(v),
    Value::Int64 => |v| Ok(v as i128),
    Value::Int32 => |v| Ok(v as i128),
    Value::Int16 => |v| Ok(v as i128),
    Value::Int8 => |v| Ok(v as i128),
    Value::UInt128 => |v| Ok(v as i128),
    Value::UInt64 => |v| Ok(v as i128),
    Value::UInt32 => |v| Ok(v as i128),
    Value::UInt16 => |v| Ok(v as i128),
    Value::UInt8 => |v| Ok(v as i128),
    Value::Decimal => |v: Decimal| {
        let error = Error::msg(format!("Value {}: Decimal does not fit into i128", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_i128().ok_or(error)
    }
);
impl_as_value!(
    u8,
    Value::UInt8 => |v| Ok(v),
    Value::Int16 => |v: i16| {
        v.to_u8().ok_or(Error::msg(format!("Value {}: i16 does not fit into u8", v)))
    }
);
impl_as_value!(
    u16,
    Value::UInt16 => |v| Ok(v),
    Value::UInt8 => |v| Ok(v as u16),
    Value::Int32 => |v: i32| {
        let result = v as u16;
        if result as i32 != v {
            return Err(Error::msg(format!("Value {}: i32 does not fit into u16", v)));
        }
        Ok(result)
    }
);
impl_as_value!(
    u32,
    Value::UInt32 => |v| Ok(v),
    Value::UInt16 => |v| Ok(v as u32),
    Value::UInt8 => |v| Ok(v as u32),
);
impl_as_value!(
    u64,
    Value::UInt64 => |v| Ok(v),
    Value::UInt32 => |v| Ok(v as u64),
    Value::UInt16 => |v| Ok(v as u64),
    Value::UInt8 => |v| Ok(v as u64),
    Value::Decimal => |v: Decimal| {
        let error = Error::msg(format!("Value {}: Decimal does not fit into u64", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_u64().ok_or(error)
    }
);
impl_as_value!(
    u128,
    Value::UInt128 => |v| Ok(v),
    Value::UInt64 => |v| Ok(v as u128),
    Value::UInt32 => |v| Ok(v as u128),
    Value::UInt16 => |v| Ok(v as u128),
    Value::UInt8 => |v| Ok(v as u128),
    Value::Decimal => |v: Decimal| {
        let error = Error::msg(format!("Value {}: Decimal does not fit into u128", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_u128().ok_or(error)
    }
);

macro_rules! impl_as_value {
    ($source:ty, $value:path => $expr:expr $(, $pat_rest:path => $expr_rest:expr)* $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $value(None)
            }
            fn as_value(self) -> Value {
                $value(Some(self.into()))
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    $value(Some(v), ..) => $expr(v),
                    $($pat_rest(Some(v), ..) => $expr_rest(v),)*
                    #[allow(unreachable_patterns)]
                    Value::Unknown(Some(ref v)) => {
                        v.parse::<$source>().map_err(|_| Error::msg(format!(
                            "Cannot decode {:?} into {}",
                            value,
                            stringify!($source)
                        )))
                    }
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        stringify!($source),
                    ))),
                }
            }
        }
    };
}
impl_as_value!(
    bool,
    Value::Boolean => |v| Ok(v),
    Value::Int8 => |v| Ok(v != 0),
    Value::Int16 => |v| Ok(v != 0),
    Value::Int32 => |v| Ok(v != 0),
    Value::Int64 => |v| Ok(v != 0),
    Value::Int128 => |v| Ok(v != 0),
    Value::UInt8 => |v| Ok(v != 0),
    Value::UInt16 => |v| Ok(v != 0),
    Value::UInt32 => |v| Ok(v != 0),
    Value::UInt64 => |v| Ok(v != 0),
    Value::UInt128 => |v| Ok(v != 0),
);
impl_as_value!(
    f32,
    Value::Float32 => |v| Ok(v),
    Value::Float64 => |v| Ok(v as f32),
    Value::Decimal => |v: Decimal| Ok(v.try_into()?),
);
impl_as_value!(
    f64,
    Value::Float64 => |v| Ok(v),
    Value::Float32 => |v| Ok(v as f64),
    Value::Decimal => |v: Decimal| Ok(v.try_into()?),
);
impl_as_value!(
    char,
    Value::Char => |v| Ok(v),
    Value::Varchar => |v: String| {
        if v.len() != 1 {
            return Err(Error::msg("Cannot convert Value::Varchar containing more then one character into a char"))
        }
        Ok(v.chars().next().unwrap())
    }
);
impl_as_value!(
    String,
    Value::Varchar => |v| Ok(v),
    Value::Char => |v: char| Ok(v.into()),
    Value::Unknown => |v: String| Ok(v),
);
impl<'a> AsValue for Cow<'a, str> {
    fn as_empty_value() -> Value {
        Value::Varchar(None)
    }
    fn as_value(self) -> Value {
        Value::Varchar(Some(self.into()))
    }
    fn try_from_value(value: Value) -> Result<Self>
    where
        Self: Sized,
    {
        let Value::Varchar(Some(value)) = value else {
            return Err(Error::msg(format!(
                "Cannot convert {} to Cow<'a, str>",
                value.to_token_stream().to_string(),
            )));
        };
        Ok(value.into())
    }
}

macro_rules! impl_as_value {
    ($source:ty, $value:path => $expr:expr $(, $pat_rest:path => $expr_rest:expr)* $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $value(None)
            }
            fn as_value(self) -> Value {
                $value(Some(self.into()))
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    $value(Some(v), ..) => $expr(v),
                    $($pat_rest(Some(v), ..) => $expr_rest(v),)*
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        stringify!($source),
                    ))),
                }
            }
        }
    };
}
impl_as_value!(
    Box<[u8]>,
    Value::Blob => |v| Ok(v),
);
impl_as_value!(
    time::Date,
    Value::Date => |v| Ok(v),
    Value::Varchar => Parse::parse,
    Value::Unknown => Parse::parse,
);
impl_as_value!(
    time::Time,
    Value::Time => |v| Ok(v),
    Value::Varchar => Parse::parse,
    Value::Unknown => Parse::parse,
);
impl_as_value!(
    time::PrimitiveDateTime,
    Value::Timestamp => |v| Ok(v),
    Value::Varchar => Parse::parse,
    Value::Unknown => Parse::parse,
);
impl_as_value!(
    time::OffsetDateTime,
    Value::TimestampWithTimezone => |v| Ok(v),
    Value::Varchar => Parse::parse,
    Value::Unknown => Parse::parse,
);
impl_as_value!(std::time::Duration, Value::Interval => |v: Interval| Ok(v.into()));
impl_as_value!(Interval, Value::Interval => |v| Ok(v));
impl_as_value!(time::Duration, Value::Interval => |v: Interval| Ok(v.into()));
impl_as_value!(
    uuid::Uuid,
    Value::Uuid => |v| Ok(v),
    Value::Varchar => |v: String| Ok(uuid::Uuid::parse_str(&v)?),
);

impl AsValue for Decimal {
    fn as_empty_value() -> Value {
        Value::Decimal(None, 0, 0)
    }
    fn as_value(self) -> Value {
        Value::Decimal(Some(self), 0, self.scale() as _)
    }
    fn try_from_value(value: Value) -> Result<Self> {
        match value {
            Value::Decimal(Some(v), ..) => Ok(v),
            Value::Int8(Some(v), ..) => Ok(Decimal::new(v as i64, 0)),
            Value::Int16(Some(v), ..) => Ok(Decimal::new(v as i64, 0)),
            Value::Int32(Some(v), ..) => Ok(Decimal::new(v as i64, 0)),
            Value::Int64(Some(v), ..) => Ok(Decimal::new(v, 0)),
            Value::UInt8(Some(v), ..) => Ok(Decimal::new(v as i64, 0)),
            Value::UInt16(Some(v), ..) => Ok(Decimal::new(v as i64, 0)),
            Value::UInt32(Some(v), ..) => Ok(Decimal::new(v as i64, 0)),
            Value::UInt64(Some(v), ..) => Ok(Decimal::new(v as i64, 0)),
            Value::Float32(Some(v), ..) => Ok(Decimal::from_f32(v)
                .ok_or(Error::msg(format!("Cannot convert {:?} to Decimal", value)))?),
            Value::Float64(Some(v), ..) => Ok(Decimal::from_f64(v)
                .ok_or(Error::msg(format!("Cannot convert {:?} to Decimal", value)))?),
            _ => Err(Error::msg(
                format!("Cannot convert {:?} to Decimal", value,),
            )),
        }
    }
}

impl<const W: u8, const S: u8> AsValue for FixedDecimal<W, S> {
    fn as_empty_value() -> Value {
        Decimal::as_empty_value()
    }
    fn as_value(self) -> Value {
        Value::Decimal(Some(self.0), W, S)
    }
    fn try_from_value(value: Value) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(Decimal::try_from_value(value)?))
    }
}

impl<T: AsValue, const N: usize> AsValue for [T; N] {
    fn as_empty_value() -> Value {
        Value::Array(None, Box::new(T::as_empty_value()), N as u32)
    }
    fn as_value(self) -> Value {
        Value::Array(
            Some(self.into_iter().map(AsValue::as_value).collect()),
            Box::new(T::as_empty_value()),
            N as u32,
        )
    }
    fn try_from_value(value: Value) -> Result<Self> {
        let convert_iter = |iter: Vec<Value>| -> Result<[T; N]> {
            iter.into_iter()
                .map(T::try_from_value)
                .collect::<Result<Vec<_>>>()?
                .try_into()
                .map_err(|v: Vec<T>| {
                    Error::msg(format!(
                        "Expected array of length {}, got {} elements ({})",
                        N,
                        v.len(),
                        any::type_name::<[T; N]>()
                    ))
                })
        };
        match value {
            Value::List(Some(v), ..) if v.len() == N => convert_iter(v),
            Value::Array(Some(v), ..) if v.len() == N => convert_iter(v.into()),
            _ => Err(Error::msg(format!(
                "Cannot convert {:?} to array {}",
                value,
                any::type_name::<Self>()
            ))),
        }
    }
}

macro_rules! impl_as_value {
    ($list:ident) => {
        impl<T: AsValue> AsValue for $list<T> {
            fn as_empty_value() -> Value {
                Value::List(None, Box::new(T::as_empty_value()))
            }
            fn as_value(self) -> Value {
                Value::List(
                    Some(self.into_iter().map(AsValue::as_value).collect()),
                    Box::new(T::as_empty_value()),
                )
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    Value::List(Some(v), ..) => Ok(v
                        .into_iter()
                        .map(|v| Ok::<_, Error>(<T as AsValue>::try_from_value(v)?))
                        .collect::<Result<_>>()?),
                    Value::List(None, ..) => Ok($list::<T>::new()),
                    Value::Array(Some(v), ..) => Ok(v
                        .into_iter()
                        .map(|v| Ok::<_, Error>(<T as AsValue>::try_from_value(v)?))
                        .collect::<Result<_>>()?),
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        stringify!($list<T>),
                    ))),
                }
            }
        }
    };
}
impl_as_value!(Vec);
impl_as_value!(VecDeque);
impl_as_value!(LinkedList);

macro_rules! impl_as_value {
    ($map:ident, $($key_trait:ident),+) => {
        impl<K: AsValue $(+ $key_trait)+, V: AsValue> AsValue for $map<K, V> {
            fn as_empty_value() -> Value {
                Value::Map(None, K::as_empty_value().into(), V::as_empty_value().into())
            }
            fn as_value(self) -> Value {
                Value::Map(
                    Some(
                        self.into_iter()
                            .map(|(k, v)| (k.as_value(), v.as_value()))
                            .collect(),
                    ),
                    K::as_empty_value().into(),
                    V::as_empty_value().into(),
                )
            }
            fn try_from_value(value: Value) -> Result<Self> {
                if let Value::Map(Some(v), ..) = value {
                    Ok(v.into_iter()
                        .map(|(k, v)| {
                            Ok((
                                <K as AsValue>::try_from_value(k)?,
                                <V as AsValue>::try_from_value(v)?,
                            ))
                        })
                        .collect::<Result<_>>()?)
                } else {
                    Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        stringify!($map<K, V>),
                    )))
                }
            }
        }
    }
}
impl_as_value!(BTreeMap, Ord);
impl_as_value!(HashMap, Eq, Hash);

impl<T: AsValue> AsValue for Passive<T> {
    fn as_empty_value() -> Value {
        T::as_empty_value()
    }
    fn as_value(self) -> Value {
        match self {
            Passive::Set(v) => v.as_value(),
            Passive::NotSet => T::as_empty_value(),
        }
    }
    fn try_from_value(value: Value) -> Result<Self> {
        Ok(Passive::Set(<T as AsValue>::try_from_value(value)?))
    }
}

impl<T: AsValue> AsValue for Option<T> {
    fn as_empty_value() -> Value {
        T::as_empty_value()
    }
    fn as_value(self) -> Value {
        match self {
            Some(v) => v.as_value(),
            None => T::as_empty_value(),
        }
    }
    fn try_from_value(value: Value) -> Result<Self> {
        Ok(if value.is_null() {
            None
        } else {
            Some(<T as AsValue>::try_from_value(value)?)
        })
    }
}

// TODO: USe the macro below once box_into_inner is stabilized
impl<T: AsValue> AsValue for Box<T> {
    fn as_empty_value() -> Value {
        T::as_empty_value()
    }
    fn as_value(self) -> Value {
        (*self).as_value()
    }
    fn try_from_value(value: Value) -> Result<Self> {
        Ok(Box::new(<T as AsValue>::try_from_value(value)?))
    }
}

macro_rules! impl_as_value {
    ($wrapper:ident) => {
        impl<T: AsValue + ToOwned<Owned = impl AsValue>> AsValue for $wrapper<T> {
            fn as_empty_value() -> Value {
                T::as_empty_value()
            }
            fn as_value(self) -> Value {
                $wrapper::<T>::into_inner(self).as_value()
            }
            fn try_from_value(value: Value) -> Result<Self> {
                Ok($wrapper::new(<T as AsValue>::try_from_value(value)?))
            }
        }
    };
}
impl_as_value!(Cell);
impl_as_value!(RefCell);

impl<T: AsValue> AsValue for RwLock<T> {
    fn as_empty_value() -> Value {
        T::as_empty_value()
    }
    fn as_value(self) -> Value {
        self.into_inner()
            .expect("Error occurred while trying to take the content of the RwLock")
            .as_value()
    }
    fn try_from_value(value: Value) -> Result<Self> {
        Ok(RwLock::new(<T as AsValue>::try_from_value(value)?))
    }
}

macro_rules! impl_as_value {
    ($wrapper:ident) => {
        impl<T: AsValue + ToOwned<Owned = impl AsValue>> AsValue for $wrapper<T> {
            fn as_empty_value() -> Value {
                T::as_empty_value()
            }
            fn as_value(self) -> Value {
                $wrapper::try_unwrap(self)
                    .map(|v| v.as_value())
                    .unwrap_or_else(|v| v.as_ref().to_owned().as_value())
            }
            fn try_from_value(value: Value) -> Result<Self> {
                Ok($wrapper::new(<T as AsValue>::try_from_value(value)?))
            }
        }
    };
}
impl_as_value!(Arc);
impl_as_value!(Rc);
