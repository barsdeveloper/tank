use crate::{Error, Passive, Result, interval::Interval};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use std::{
    array::{self},
    collections::{BTreeMap, HashMap, LinkedList, VecDeque},
    hash::Hash,
    mem::discriminant,
    rc::Rc,
    sync::Arc,
};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, macros::format_description};
use uuid::Uuid;

#[derive(Default, Debug, Clone)]
pub enum Value {
    #[default]
    Null,
    Boolean(Option<bool>),
    Int8(Option<i8>),
    Int16(Option<i16>),
    Int32(Option<i32>),
    Int64(Option<i64>),
    Int128(Option<i128>),
    UInt8(Option<u8>),
    UInt16(Option<u16>),
    UInt32(Option<u32>),
    UInt64(Option<u64>),
    UInt128(Option<u128>),
    Float32(Option<f32>),
    Float64(Option<f64>),
    Decimal(Option<Decimal>, /* width: */ u8, /* scale: */ u8),
    Varchar(Option<String>),
    Blob(Option<Box<[u8]>>),
    Date(Option<Date>),
    Time(Option<Time>),
    Timestamp(Option<PrimitiveDateTime>),
    TimestampWithTimezone(Option<OffsetDateTime>),
    Interval(Option<Interval>),
    Uuid(Option<Uuid>),
    Array(
        Option<Box<[Value]>>,
        /* type: */ Box<Value>,
        /* len: */ u32,
    ),
    List(Option<Vec<Value>>, /* type: */ Box<Value>),
    Map(
        Option<HashMap<Value, Value>>,
        /* key: */ Box<Value>,
        /* value: */ Box<Value>,
    ),
}

impl Value {
    pub fn same_type(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Decimal(.., l_width, l_scale), Self::Decimal(.., r_width, r_scale)) => {
                (*l_width == 0 || *r_width == 0 || l_width == r_width)
                    && (*l_scale == 0 || *r_scale == 0 || l_scale == r_scale)
            }
            (Self::Array(.., l_type, l_len), Self::Array(.., r_type, r_len)) => {
                l_len == r_len && l_type.same_type(&r_type)
            }
            (Self::List(.., l), Self::List(.., r)) => l.same_type(r),
            (Self::Map(.., l_key, l_value), Self::Map(.., r_key, r_value)) => {
                l_key.same_type(r_key) && l_value.same_type(&r_value)
            }
            _ => discriminant(self) == discriminant(other),
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Value::Null
            | Value::Boolean(None, ..)
            | Value::Int8(None, ..)
            | Value::Int16(None, ..)
            | Value::Int32(None, ..)
            | Value::Int64(None, ..)
            | Value::Int128(None, ..)
            | Value::UInt8(None, ..)
            | Value::UInt16(None, ..)
            | Value::UInt32(None, ..)
            | Value::UInt64(None, ..)
            | Value::UInt128(None, ..)
            | Value::Float32(None, ..)
            | Value::Float64(None, ..)
            | Value::Decimal(None, ..)
            | Value::Varchar(None, ..)
            | Value::Blob(None, ..)
            | Value::Date(None, ..)
            | Value::Time(None, ..)
            | Value::Timestamp(None, ..)
            | Value::TimestampWithTimezone(None, ..)
            | Value::Interval(None, ..)
            | Value::Uuid(None, ..)
            | Value::Array(None, ..)
            | Value::List(None, ..)
            | Value::Map(None, ..) => true,
            _ => false,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(l), Self::Boolean(r)) => l == r,
            (Self::Int8(l), Self::Int8(r)) => l == r,
            (Self::Int16(l), Self::Int16(r)) => l == r,
            (Self::Int32(l), Self::Int32(r)) => l == r,
            (Self::Int64(l), Self::Int64(r)) => l == r,
            (Self::Int128(l), Self::Int128(r)) => l == r,
            (Self::UInt8(l), Self::UInt8(r)) => l == r,
            (Self::UInt16(l), Self::UInt16(r)) => l == r,
            (Self::UInt32(l), Self::UInt32(r)) => l == r,
            (Self::UInt64(l), Self::UInt64(r)) => l == r,
            (Self::UInt128(l), Self::UInt128(r)) => l == r,
            (Self::Float32(l), Self::Float32(r)) => {
                l == r
                    || l.and_then(|l| r.and_then(|r| Some(l.is_nan() && r.is_nan())))
                        .unwrap_or_default()
            }
            (Self::Float64(l), Self::Float64(r)) => {
                l == r
                    || l.and_then(|l| r.and_then(|r| Some(l.is_nan() && r.is_nan())))
                        .unwrap_or_default()
            }
            (Self::Decimal(l, l_width, l_scale), Self::Decimal(r, r_width, r_scale)) => {
                l == r && l_width == r_width && l_scale == r_scale
            }
            (Self::Varchar(l), Self::Varchar(r)) => l == r,
            (Self::Blob(l), Self::Blob(r)) => l == r,
            (Self::Date(l), Self::Date(r)) => l == r,
            (Self::Time(l), Self::Time(r)) => l == r,
            (Self::Timestamp(l), Self::Timestamp(r)) => l == r,
            (Self::TimestampWithTimezone(l), Self::TimestampWithTimezone(r)) => l == r,
            (Self::Interval(l), Self::Interval(r)) => l == r,
            (Self::Uuid(l), Self::Uuid(r)) => l == r,
            (Self::Array(l, ..), Self::Array(r, ..)) => l == r && self.same_type(other),
            (Self::List(l, ..), Self::List(r, ..)) => l == r && self.same_type(other),
            (Self::Map(None, ..), Self::Map(None, ..)) => self.same_type(other),
            (Self::Map(Some(l), ..), Self::Map(Some(r), ..)) => {
                l.is_empty() && r.is_empty() && self.same_type(other)
            }
            (Self::Map(..), Self::Map(..)) => false,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Value::*;
        discriminant(self).hash(state);
        match self {
            Null => {}
            Boolean(v) => v.hash(state),
            Int8(v) => v.hash(state),
            Int16(v) => v.hash(state),
            Int32(v) => v.hash(state),
            Int64(v) => v.hash(state),
            Int128(v) => v.hash(state),

            UInt8(v) => v.hash(state),
            UInt16(v) => v.hash(state),
            UInt32(v) => v.hash(state),
            UInt64(v) => v.hash(state),
            UInt128(v) => v.hash(state),

            Float32(Some(v)) => {
                v.to_bits().hash(state);
            }
            Float32(None) => None::<u32>.hash(state),

            Float64(Some(v)) => {
                v.to_bits().hash(state);
            }
            Float64(None) => None::<u64>.hash(state),

            Decimal(v, width, scale) => {
                v.hash(state);
                width.hash(state);
                scale.hash(state);
            }

            Varchar(v) => v.hash(state),
            Blob(v) => v.hash(state),
            Date(v) => v.hash(state),
            Time(v) => v.hash(state),
            Timestamp(v) => v.hash(state),
            TimestampWithTimezone(v) => v.hash(state),
            Interval(v) => v.hash(state),
            Uuid(v) => v.hash(state),

            Array(v, typ, len) => {
                v.hash(state);
                typ.hash(state);
                len.hash(state);
            }

            List(v, typ) => {
                v.hash(state);
                typ.hash(state);
            }

            Map(opt_map, k, v) => {
                match opt_map {
                    Some(map) => {
                        for (key, val) in map {
                            key.hash(state);
                            val.hash(state);
                        }
                    }
                    None => {}
                }
                k.hash(state);
                v.hash(state);
            }
        }
    }
}

#[derive(Default)]
pub struct TypeDecoded {
    pub value: Value,
    pub nullable: bool,
    pub passive: bool,
}

pub type CheckPassive = Box<dyn Fn(TokenStream) -> TokenStream>;

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ts = match self {
            Value::Null => quote! { ::tank::Value::Null },
            Value::Boolean(..) => quote! { ::tank::Value::Boolean(None) },
            Value::Int8(..) => quote! { ::tank::Value::Int8(None) },
            Value::Int16(..) => quote! { ::tank::Value::Int16(None) },
            Value::Int32(..) => quote! { ::tank::Value::Int32(None) },
            Value::Int64(..) => quote! { ::tank::Value::Int64(None) },
            Value::Int128(..) => quote! { ::tank::Value::Int128(None) },
            Value::UInt8(..) => quote! { ::tank::Value::UInt8(None) },
            Value::UInt16(..) => quote! { ::tank::Value::UInt16(None) },
            Value::UInt32(..) => quote! { ::tank::Value::UInt32(None) },
            Value::UInt64(..) => quote! { ::tank::Value::UInt64(None) },
            Value::UInt128(..) => quote! { ::tank::Value::UInt128(None) },
            Value::Float32(..) => quote! { ::tank::Value::Float32(None) },
            Value::Float64(..) => quote! { ::tank::Value::Float64(None) },
            Value::Decimal(.., width, scale) => {
                quote! { ::tank::Value::Decimal(None, #width, #scale) }
            }
            Value::Varchar(..) => quote! { ::tank::Value::Varchar(None) },
            Value::Blob(..) => quote! { ::tank::Value::Blob(None) },
            Value::Date(..) => quote! { ::tank::Value::Date(None) },
            Value::Time(..) => quote! { ::tank::Value::Time(None) },
            Value::Timestamp(..) => quote! { ::tank::Value::Timestamp(None) },
            Value::TimestampWithTimezone(..) => {
                quote! { ::tank::Value::TimestampWithTimezone(None) }
            }
            Value::Interval(..) => quote! { ::tank::Value::Interval(None) },
            Value::Uuid(..) => quote! { ::tank::Value::Uuid(None) },
            Value::Array(.., inner, size) => {
                quote! { ::tank::Value::Array (None, Box::new(#inner), #size) }
            }
            Value::List(.., inner) => {
                let inner = inner.as_ref().to_token_stream();
                quote! { ::tank::Value::List(None, Box::new(#inner)) }
            }
            Value::Map(.., key, value) => {
                let key = key.as_ref().to_token_stream();
                let value = value.as_ref().to_token_stream();
                quote! { ::tank::Value::Map(None, Box::new(#key), Box::new(#value)) }
            }
        };
        tokens.extend(ts);
    }
}

pub trait AsValue {
    fn as_empty_value() -> Value;
    fn as_value(self) -> Value;
    fn try_from_value(value: Value) -> Result<Self>
    where
        Self: Sized;
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
                        "Cannot convert `{}` into `{}`",
                        value.to_token_stream().to_string(),
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
    i8,
    Value::Int8 => |v| Ok(v),
    Value::UInt8 => |v| Ok(v as i8),
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
);
impl_as_value!(u8, Value::UInt8 => |v| Ok(v));
impl_as_value!(
    u16,
    Value::UInt16 => |v| Ok(v),
    Value::UInt8 => |v| Ok(v as u16),
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
);
impl_as_value!(
    u128,
    Value::UInt128 => |v| Ok(v),
    Value::UInt64 => |v| Ok(v as u128),
    Value::UInt32 => |v| Ok(v as u128),
    Value::UInt16 => |v| Ok(v as u128),
    Value::UInt8 => |v| Ok(v as u128),
);
impl_as_value!(
    f32,
    Value::Float32 => |v| Ok(v),
    Value::Decimal => |v: Decimal| Ok(v.try_into()?),
);
impl_as_value!(
    f64,
    Value::Float64 => |v| Ok(v),
    Value::Float32 => |v| Ok(v as f64),
    Value::Decimal => |v: Decimal| Ok(v.try_into()?),
);
impl_as_value!(String, Value::Varchar => |v| Ok(v));
impl_as_value!(Box<[u8]>, Value::Blob => |v| Ok(v));
impl_as_value!(
    time::Date,
    Value::Date => |v| Ok(v),
    Value::Varchar => |v: String| Ok(time::Date::parse(
        &v,
        format_description!("[year]-[month]-[day]")
    )?),
);
impl_as_value!(
    time::Time,
    Value::Time => |v| Ok(v),
    Value::Varchar => |v: String| Ok(time::Time::parse(
        &v,
        format_description!("[hour]:[minute]:[second].[subsecond]")
    )?),
);
impl_as_value!(
    time::PrimitiveDateTime,
    Value::Timestamp => |v| Ok(v),
    Value::Varchar => |v: String| Ok(time::PrimitiveDateTime::parse(
        &v,
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]")
    )?),
);
impl_as_value!(
    time::OffsetDateTime,
    Value::TimestampWithTimezone => |v| Ok(v),
    Value::Varchar => |v: String| Ok(time::OffsetDateTime::parse(
        &v,
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond][offset_hour sign:mandatory]:[offset_minute]")
    )?),
);
impl_as_value!(Interval, Value::Interval => |v| Ok(v));
impl_as_value!(std::time::Duration, Value::Interval => |v: Interval| Ok(v.into()));
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
                .ok_or(Error::msg("Could not convert the Float32 into Decimal"))?),
            Value::Float64(Some(v), ..) => Ok(Decimal::from_f64(v)
                .ok_or(Error::msg("Could not convert the Float64 into Decimal"))?),
            _ => Err(Error::msg(format!(
                "Cannot convert `{}` into `{}`",
                value.to_token_stream().to_string(),
                stringify!(rust_decimal::Decimal),
            ))),
        }
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
        let err = Error::msg(format!(
            "Cannot convert `{}` into `{}`",
            value.to_token_stream().to_string(),
            stringify!(Vec<T>),
        ));
        if let Value::List(Some(v), ..) = value {
            if v.len() == N {
                let mut it = v.into_iter();
                let result =
                    array::try_from_fn(|_| T::try_from_value(it.next().ok_or(Error::msg(""))?))?;
                return Ok(result);
            }
        }
        Err(err)
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
                    Value::Array(Some(v), ..) => Ok(v
                        .into_iter()
                        .map(|v| Ok::<_, Error>(<T as AsValue>::try_from_value(v)?))
                        .collect::<Result<_>>()?),
                    _ => Err(Error::msg(format!(
                        "Cannot convert `{}` into `{}`",
                        value.to_token_stream().to_string(),
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
                        "Cannot convert `{}` into `{}`",
                        value.to_token_stream().to_string(),
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
                (*self).to_owned().as_value()
            }
            fn try_from_value(value: Value) -> Result<Self> {
                Ok($wrapper::new(<T as AsValue>::try_from_value(value)?))
            }
        }
    };
}

impl_as_value!(Arc);
impl_as_value!(Rc);

impl<T: AsValue> From<T> for Value {
    fn from(value: T) -> Self {
        value.as_value()
    }
}
