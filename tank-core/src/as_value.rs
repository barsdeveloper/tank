use crate::{Error, FixedDecimal, Interval, Passive, Result, Value};
use atoi::FromRadix10;
use quote::ToTokens;
use rust_decimal::{Decimal, prelude::FromPrimitive, prelude::ToPrimitive};
use std::{
    any, array,
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, LinkedList, VecDeque},
    hash::Hash,
    mem,
    rc::Rc,
    sync::{Arc, RwLock},
};
use time::format_description::parse_borrowed;
use uuid::Uuid;

pub trait AsValue {
    fn as_empty_value() -> Value;
    fn as_value(self) -> Value;
    fn try_from_value(value: Value) -> Result<Self>
    where
        Self: Sized;
    fn parse(value: impl AsRef<str>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut value = value.as_ref();
        let result = Self::extract(&mut value)?;
        if !value.is_empty() {
            return Err(Error::msg(format!(
                "Value \"{}\" parsed correctly as `{}` but it did not consume all the input",
                value,
                any::type_name::<Self>()
            )));
        }
        Ok(result)
    }
    fn extract(value: &mut &str) -> Result<Self>
    where
        Self: Sized,
    {
        Err(Error::msg(format!(
            "Cannot parse '{}' as {}",
            value,
            any::type_name::<Self>()
        )))
    }
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
    ($source:ty, $destination:path $(, $pat_rest:pat => $expr_rest:expr)* $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $destination(None)
            }
            fn as_value(self) -> Value {
                $destination(Some(self.into()))
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    $destination(Some(v), ..) => Ok(v),
                    $($pat_rest => $expr_rest,)*
                    #[allow(unreachable_patterns)]
                    Value::Int64(Some(v), ..) => {
                        if (v as i128).clamp(<$source>::MIN as i128, <$source>::MAX as i128) != v as i128 {
                            return Err(Error::msg(format!(
                                "Value {}: i64 does not fit into {}",
                                v,
                                any::type_name::<Self>(),
                            )));
                        }
                        Ok(v as $source)
                    }
                    Value::Unknown(Some(ref v), ..) => Self::parse(v),
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        any::type_name::<Self>(),
                    ))),
                }
            }
            fn extract(value: &mut &str) -> Result<Self> {
                let (r, tail) = <$source>::from_radix_10(value.as_bytes());
                *value = &value[tail..];
                Ok(r)
            }
        }
    };
}
impl_as_value!(
    i8,
    Value::Int8,
    Value::UInt8(Some(v), ..) => Ok(v as i8),
    Value::Int16(Some(v), ..) => {
        let result = v as i8;
        if result as i16 != v {
            return Err(Error::msg(format!("Value {}: i64 does not fit into i8", v)));
        }
        Ok(result)
    },
);
impl_as_value!(
    i16,
    Value::Int16,
    Value::Int8(Some(v), ..) => Ok(v as i16),
    Value::UInt16(Some(v), ..) => Ok(v as i16),
    Value::UInt8(Some(v), ..) => Ok(v as i16),
);
impl_as_value!(
    i32,
    Value::Int32,
    Value::Int16(Some(v), ..) => Ok(v as i32),
    Value::Int8(Some(v), ..) => Ok(v as i32),
    Value::UInt32(Some(v), ..) => Ok(v as i32),
    Value::UInt16(Some(v), ..) => Ok(v as i32),
    Value::UInt8(Some(v), ..) => Ok(v as i32),
    Value::Decimal(Some(v), ..) => {
        let error = Error::msg(format!("Value {}: Decimal does not fit into i32", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_i32().ok_or(error)
    }
);
impl_as_value!(
    i64,
    Value::Int64,
    Value::Int32(Some(v), ..) => Ok(v as i64),
    Value::Int16(Some(v), ..) => Ok(v as i64),
    Value::Int8(Some(v), ..) => Ok(v as i64),
    Value::UInt64(Some(v), ..) => Ok(v as i64),
    Value::UInt32(Some(v), ..) => Ok(v as i64),
    Value::UInt16(Some(v), ..) => Ok(v as i64),
    Value::UInt8(Some(v), ..) => Ok(v as i64),
    Value::Decimal(Some(v), ..) => {
        let error = Error::msg(format!("Value {}: Decimal does not fit into i64", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_i64().ok_or(error)
    }
);
impl_as_value!(
    i128,
    Value::Int128,
    Value::Int64(Some(v), ..) => Ok(v as i128),
    Value::Int32(Some(v), ..) => Ok(v as i128),
    Value::Int16(Some(v), ..) => Ok(v as i128),
    Value::Int8(Some(v), ..) => Ok(v as i128),
    Value::UInt128(Some(v), ..) => Ok(v as i128),
    Value::UInt64(Some(v), ..) => Ok(v as i128),
    Value::UInt32(Some(v), ..) => Ok(v as i128),
    Value::UInt16(Some(v), ..) => Ok(v as i128),
    Value::UInt8(Some(v), ..) => Ok(v as i128),
    Value::Decimal(Some(v), ..) => {
        let error = Error::msg(format!("Value {}: Decimal does not fit into i128", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_i128().ok_or(error)
    }
);
impl_as_value!(
    u8,
    Value::UInt8,
    Value::Int16(Some(v), ..) => {
        v.to_u8().ok_or(Error::msg(format!("Value {}: i16 does not fit into u8", v)))
    }
);
impl_as_value!(
    u16,
    Value::UInt16,
    Value::UInt8(Some(v), ..) => Ok(v as u16),
    Value::Int32(Some(v), ..) => {
        let result = v as u16;
        if result as i32 != v {
            return Err(Error::msg(format!("Value {}: i32 does not fit into u16", v)));
        }
        Ok(result)
    }
);
impl_as_value!(
    u32,
    Value::UInt32,
    Value::UInt16(Some(v), ..) => Ok(v as u32),
    Value::UInt8(Some(v), ..) => Ok(v as u32),
);
impl_as_value!(
    u64,
    Value::UInt64,
    Value::UInt32(Some(v), ..) => Ok(v as u64),
    Value::UInt16(Some(v), ..) => Ok(v as u64),
    Value::UInt8(Some(v), ..) => Ok(v as u64),
    Value::Decimal(Some(v), ..) => {
        let error = Error::msg(format!("Value {}: Decimal does not fit into u64", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_u64().ok_or(error)
    }
);
impl_as_value!(
    u128,
    Value::UInt128,
    Value::UInt64(Some(v), ..) => Ok(v as u128),
    Value::UInt32(Some(v), ..) => Ok(v as u128),
    Value::UInt16(Some(v), ..) => Ok(v as u128),
    Value::UInt8(Some(v), ..) => Ok(v as u128),
    Value::Decimal(Some(v), ..) => {
        let error = Error::msg(format!("Value {}: Decimal does not fit into u128", v));
        if !v.is_integer() {
            return Err(error.context("The value is not integer"));
        }
        v.to_u128().ok_or(error)
    }
);

macro_rules! impl_as_value {
    ($source:ty, $dest:path $(, $pat_rest:pat => $expr_rest:expr)* $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $dest(None)
            }
            fn as_value(self) -> Value {
                $dest(Some(self.into()))
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    $dest(Some(v), ..) => Ok(v),
                    $($pat_rest => $expr_rest,)*
                    #[allow(unreachable_patterns)]
                    Value::Unknown(Some(ref v)) => Self::parse(v),
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        any::type_name::<Self>(),
                    ))),
                }
            }
            fn extract(value: &mut &str) -> Result<Self> {
                value.parse::<$source>().map_err(|_| Error::msg(format!(
                    "Cannot decode {:?} into {}",
                    value,
                    any::type_name::<Self>()
                )))
            }
        }
    };
}
impl_as_value!(
    bool,
    Value::Boolean,
    Value::Int8(Some(v), ..) => Ok(v != 0),
    Value::Int16(Some(v), ..) => Ok(v != 0),
    Value::Int32(Some(v), ..) => Ok(v != 0),
    Value::Int64(Some(v), ..) => Ok(v != 0),
    Value::Int128(Some(v), ..) => Ok(v != 0),
    Value::UInt8(Some(v), ..) => Ok(v != 0),
    Value::UInt16(Some(v), ..) => Ok(v != 0),
    Value::UInt32(Some(v), ..) => Ok(v != 0),
    Value::UInt64(Some(v), ..) => Ok(v != 0),
    Value::UInt128(Some(v), ..) => Ok(v != 0),
);
impl_as_value!(
    f32,
    Value::Float32,
    Value::Float64(Some(v), ..) => Ok(v as f32),
    Value::Decimal(Some(v), ..) => Ok(v.try_into()?),
);
impl_as_value!(
    f64,
    Value::Float64,
    Value::Float32(Some(v), ..) => Ok(v as f64),
    Value::Decimal(Some(v), ..) => Ok(v.try_into()?),
);
impl_as_value!(
    char,
    Value::Char,
    Value::Varchar(Some(v), ..) | Value::Unknown(Some(v), ..) => {
        if v.len() != 1 {
            return Err(Error::msg("Cannot convert Value::Varchar containing more then one character into a char"))
        }
        Ok(v.chars().next().unwrap())
    }
);
impl_as_value!(
    String,
    Value::Varchar,
    Value::Char(Some(v), ..) => Ok(v.into()),
    Value::Unknown(Some(v), ..) => Ok(v),
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
        match value {
            Value::Varchar(Some(value)) | Value::Unknown(Some(value)) => Ok(value.into()),
            _ => Err(Error::msg(format!(
                "Cannot convert {} to Cow<'a, str>",
                value.to_token_stream().to_string(),
            ))),
        }
    }
}

macro_rules! impl_as_value {
    ($source:ty, $dest:path, $parser:expr $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $dest(None)
            }
            fn as_value(self) -> Value {
                $dest(Some(self.into()))
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    $dest(Some(v), ..) => Ok(v.into()),
                    Value::Unknown(Some(v), ..) => Self::parse(v),
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        any::type_name::<Self>(),
                    ))),
                }
            }
            fn extract(value: &mut &str) -> Result<Self> {
                $parser(value)
            }
        }
    };
}
impl_as_value!(Box<[u8]>, Value::Blob, |v| Err(Error::msg("")));
impl_as_value!(std::time::Duration, Value::Interval, |v| Err(Error::msg(
    ""
)));
impl_as_value!(Interval, Value::Interval, |v| Err(Error::msg("")));
impl_as_value!(time::Duration, Value::Interval, |v| {
    <time::Time as AsValue>::extract(v).map(|v| v.duration_since(time::Time::MIDNIGHT))
});

macro_rules! impl_as_value {
    ($source:ty, $dest:path, $parser:expr $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $dest(None)
            }
            fn as_value(self) -> Value {
                $dest(Some(self.into()))
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    $dest(Some(v), ..) => Ok(v.into()),
                    Value::Varchar(Some(v), ..) | Value::Unknown(Some(v), ..) => Self::parse(v),
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        any::type_name::<Self>(),
                    ))),
                }
            }
            fn extract(value: &mut &str) -> Result<Self> {
                $parser(value)
            }
        }
    };
}
impl_as_value!(Uuid, Value::Uuid, |v: &mut &str| {
    let result = Ok(Uuid::parse_str(&v[0..36])?);
    *v = &v[36..];
    result
});

macro_rules! impl_as_value {
    ($source:ty, $dest:path $(, $formats:literal)+ $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $dest(None)
            }
            fn as_value(self) -> Value {
                $dest(Some(self.into()))
            }
            fn try_from_value(value: Value) -> Result<Self> {
                match value {
                    $dest(Some(v), ..) => Ok(v.into()),
                    Value::Varchar(Some(v), ..) | Value::Unknown(Some(v), ..) => {
                        <Self as AsValue>::parse(v)
                    }
                    _ => Err(Error::msg(format!(
                        "Cannot convert {:?} to {}",
                        value,
                        any::type_name::<Self>(),
                    ))),
                }
            }
            fn extract(value: &mut &str) -> Result<Self> {
                for format in [$($formats,)+] {
                    let format = parse_borrowed::<2>(format).expect("The format was not valid");
                    let mut parsed = time::parsing::Parsed::new();
                    let remaining = parsed.parse_items(value.as_bytes(), &format);
                    if let Ok(remaining) = remaining {
                        let result = parsed.try_into()?;
                        *value = &value[(value.len() - remaining.len())..];
                        return Ok(result);
                    }
                }
                Err(Error::msg(format!(
                    "Cannot parse '{}' as {}",
                    value,
                    any::type_name::<Self>()
                )))
            }
        }
    };
}

impl_as_value!(time::Date, Value::Date, "[year]-[month]-[day]");
impl_as_value!(
    time::Time,
    Value::Time,
    "[hour]:[minute]:[second].[subsecond]",
    "[hour]:[minute]:[second]",
    "[hour]:[minute]",
);
impl_as_value!(
    time::PrimitiveDateTime,
    Value::Timestamp,
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]",
    "[year]-[month]-[day]T[hour]:[minute]:[second]",
    "[year]-[month]-[day]T[hour]:[minute]",
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]",
    "[year]-[month]-[day] [hour]:[minute]:[second]",
    "[year]-[month]-[day] [hour]:[minute]",
);
impl_as_value!(
    time::OffsetDateTime,
    Value::TimestampWithTimezone,
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]",
    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]",
    "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]",
    "[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]",
    "[year]-[month]-[day]T[hour]:[minute][offset_hour sign:mandatory]:[offset_minute]",
    "[year]-[month]-[day]T[hour]:[minute][offset_hour sign:mandatory]",
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
            Value::Unknown(Some(v)) => Self::parse(v),
            _ => Err(Error::msg(format!(
                "Cannot convert {:?} to array {}",
                value,
                any::type_name::<Self>()
            ))),
        }
    }
    fn extract(value: &mut &str) -> Result<Self> {
        *value = match (value.chars().next(), value.chars().last()) {
            (Some('{'), Some('}')) | (Some('['), Some(']')) => &value[1..value.len() - 1],
            _ => {
                return Err(Error::msg(format!(
                    "Cannot parse '{}' as array {}",
                    value,
                    any::type_name::<Self>()
                )));
            }
        };
        // TODO Replace with array::from_fn once stable
        let mut result = array::from_fn(|i| {
            let result = T::extract(value);
            match value.chars().next() {
                Some(',') => *value = &value[1..],
                _ if i != N - 1 => {
                    return Err(Error::msg(format!("Incorrect array format `{}`", value)));
                }
                _ => {}
            }
            result
        });
        // if !value.is_empty() {
        //     return Err(Error::msg(format!(
        //         "Some elements in the array could not be parsed: '{}'",
        //         value
        //     )));
        // }
        if let Some(error) = result.iter_mut().find_map(|v| {
            if let Err(e) = v {
                let mut r = Error::msg("");
                mem::swap(e, &mut r);
                Some(r)
            } else {
                None
            }
        }) {
            return Err(error);
        }
        Ok(result.map(Result::unwrap))
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
                        any::type_name::<Self>(),
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
                        any::type_name::<Self>(),
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
