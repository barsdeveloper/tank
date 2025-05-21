use core::panic;
use quote::{quote, ToTokens};
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use syn::{GenericArgument, Path, PathArguments, Type};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use uuid::Uuid;

use crate::interval::Interval;

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
    Decimal(Option<Decimal>, /* prec: */ u8, /* scale: */ u8),
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
        /* len: */ u8,
    ),
    List(Option<Vec<Value>>, /* type: */ Box<Value>),
    Map(
        Option<BTreeMap<Value, Value>>,
        /* key: */ Box<Value>,
        /* value: */ Box<Value>,
    ),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
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
            (Self::Float32(l), Self::Float32(r)) => l == r,
            (Self::Float64(l), Self::Float64(r)) => l == r,
            (Self::Decimal(l, l_prec, l_scale), Self::Decimal(r, r_prec, r_scale)) => {
                l == r && l_prec == r_prec && l_scale == r_scale
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
            (Self::Map(l, ..), Self::Map(r, ..)) => l == r && self.same_type(other),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Value {
    pub fn same_type(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(..), Self::Boolean(..)) => true,
            (Self::Int8(..), Self::Int8(..)) => true,
            (Self::Int16(..), Self::Int16(..)) => true,
            (Self::Int32(..), Self::Int32(..)) => true,
            (Self::Int64(..), Self::Int64(..)) => true,
            (Self::Int128(..), Self::Int128(..)) => true,
            (Self::UInt8(..), Self::UInt8(..)) => true,
            (Self::UInt16(..), Self::UInt16(..)) => true,
            (Self::UInt32(..), Self::UInt32(..)) => true,
            (Self::UInt64(..), Self::UInt64(..)) => true,
            (Self::UInt128(..), Self::UInt128(..)) => true,
            (Self::Float32(..), Self::Float32(..)) => true,
            (Self::Float64(..), Self::Float64(..)) => true,
            (Self::Decimal(.., l_prec, l_scale), Self::Decimal(.., r_prec, r_scale)) => {
                l_prec == r_prec && l_scale == r_scale
            }
            (Self::Varchar(..), Self::Varchar(..)) => true,
            (Self::Blob(..), Self::Blob(..)) => true,
            (Self::Date(..), Self::Date(..)) => true,
            (Self::Time(..), Self::Time(..)) => true,
            (Self::Timestamp(..), Self::Timestamp(..)) => true,
            (Self::TimestampWithTimezone(..), Self::TimestampWithTimezone(..)) => true,
            (Self::Interval(..), Self::Interval(..)) => true,
            (Self::Uuid(..), Self::Uuid(..)) => true,
            (Self::Array(.., l_type, l_len), Self::Array(.., r_type, r_len)) => {
                l_len == r_len && l_type.same_type(&r_type)
            }
            (Self::List(.., l), Self::List(.., r)) => l.same_type(r),
            (Self::Map(.., l_key, l_value), Self::Map(.., r_key, r_value)) => {
                l_key.same_type(r_key) && l_value.same_type(&r_value)
            }
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

pub fn decode_type(path: &Path) -> (Value, bool) {
    let mut nullable = false;
    let data_type = 'data_type: {
        let ident = path.get_ident();
        if let Some(ident) = ident {
            if ident == "bool" {
                break 'data_type Value::Boolean(None);
            } else if ident == "i8" {
                break 'data_type Value::Int8(None);
            } else if ident == "i16" {
                break 'data_type Value::Int16(None);
            } else if ident == "i32" {
                break 'data_type Value::Int32(None);
            } else if ident == "i64" {
                break 'data_type Value::Int64(None);
            } else if ident == "i128" {
                break 'data_type Value::Int128(None);
            } else if ident == "u8" {
                break 'data_type Value::UInt8(None);
            } else if ident == "u16" {
                break 'data_type Value::UInt16(None);
            } else if ident == "u32" {
                break 'data_type Value::UInt32(None);
            } else if ident == "u64" {
                break 'data_type Value::UInt64(None);
            } else if ident == "u128" {
                break 'data_type Value::UInt128(None);
            } else if ident == "f32" {
                break 'data_type Value::Float32(None);
            } else if ident == "f64" {
                break 'data_type Value::Float64(None);
            }
        }
        macro_rules! matches_path {
            ($vec:ident, $array:expr) => {
                $vec.iter().eq($array.iter().rev().take($vec.len()))
            };
        }
        let segments = path
            .segments
            .iter()
            .rev()
            .map(|v| v.ident.to_string())
            .collect::<Vec<_>>();
        if matches_path!(segments, ["std", "string", "String"]) {
            break 'data_type Value::Varchar(None);
        } else if matches_path!(segments, ["rust_decimal", "Decimal"]) {
            break 'data_type Value::Decimal(None, 0, 0);
        } else if matches_path!(segments, ["time", "Time"]) {
            break 'data_type Value::Time(None);
        } else if matches_path!(segments, ["time", "Date"]) {
            break 'data_type Value::Date(None);
        } else if matches_path!(segments, ["time", "PrimitiveDateTime"]) {
            break 'data_type Value::Date(None);
        } else if matches_path!(segments, ["std", "time", "Duration"]) {
            break 'data_type Value::Interval(None);
        } else if matches_path!(segments, ["uuid", "Uuid"]) {
            break 'data_type Value::Uuid(None);
        } else if matches_path!(segments, ["uuid", "Uuid"]) {
            break 'data_type Value::Uuid(None);
        } else {
            let is_option = matches_path!(segments, ["std", "option", "Option"]);
            let is_list = matches_path!(segments, ["std", "vec", "Vec"]);
            let is_wrapper = is_option
                || matches_path!(segments, ["std", "boxed", "Box"])
                || matches_path!(segments, ["std", "sync", "Arc"]);
            if is_list || is_wrapper {
                match &path.segments.last().unwrap().arguments {
                    PathArguments::AngleBracketed(bracketed) => {
                        if let GenericArgument::Type(Type::Path(type_path)) =
                            bracketed.args.first().unwrap()
                        {
                            let nested_type = decode_type(&type_path.path);
                            if is_wrapper {
                                nullable = if is_option {
                                    true
                                } else {
                                    nullable || nested_type.1
                                };
                                break 'data_type nested_type.0;
                            } else if is_list {
                                break 'data_type Value::List(None, Box::new(nested_type.0));
                            }
                        } else {
                            panic!(
                                "{} must have a type as the first generic argument",
                                path.to_token_stream()
                            )
                        }
                    }
                    _ => panic!("{} must have a generic argument", path.to_token_stream()),
                }
            }
        }
        panic!("Unknown type `{}`", path.to_token_stream());
    };
    (data_type, nullable)
}

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
            Value::Decimal(.., precision, scale) => {
                quote! { ::tank::Value::Decimal(None, #precision, #scale) }
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
                quote! { ::tank::Value::Array (None, #inner, #size) }
            }
            Value::List(.., inner) => {
                let inner = inner.as_ref().to_token_stream();
                quote! { ::tank::Value::List(None, Box::new(#inner)) }
            }
            Value::Map(.., key, value) => {
                let key = key.as_ref().to_token_stream();
                let value = value.as_ref().to_token_stream();
                quote! { ::tank::Value::Map<#key, #value>(None, Box::new(#key), Box::new(#value)) }
            }
        };
        tokens.extend(ts);
    }
}

trait AsValue {
    fn as_empty_value() -> Value;
    fn as_value(self) -> Value;
}

macro_rules! impl_as_value {
    ($source:ty, $into:path $(, $args:tt)* $(,)?) => {
        impl AsValue for $source {
            fn as_empty_value() -> Value {
                $into(None)
            }
            fn as_value(self) -> Value {
                $into(Some(self, $($args),*))
            }
        }
    };
}

impl_as_value!(bool, Value::Boolean);
impl_as_value!(i8, Value::Int8);
impl_as_value!(i16, Value::Int16);
impl_as_value!(i32, Value::Int32);
impl_as_value!(i64, Value::Int64);
impl_as_value!(i128, Value::Int128);
impl_as_value!(u8, Value::UInt8);
impl_as_value!(u16, Value::UInt16);
impl_as_value!(u32, Value::UInt32);
impl_as_value!(u64, Value::UInt64);
impl_as_value!(u128, Value::UInt128);
impl_as_value!(f32, Value::Float32);
impl_as_value!(f64, Value::Float64);
impl_as_value!(String, Value::Varchar);
impl_as_value!(Box<[u8]>, Value::Blob);
impl_as_value!(time::Date, Value::Date);
impl_as_value!(time::Time, Value::Time);
impl_as_value!(time::PrimitiveDateTime, Value::Timestamp);
impl_as_value!(time::OffsetDateTime, Value::TimestampWithTimezone);
impl_as_value!(Interval, Value::Interval);
impl_as_value!(uuid::Uuid, Value::Uuid);

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
}

macro_rules! impl_as_value {
    ($wrapper:ident) => {
        impl<T: AsValue> AsValue for $wrapper<T> {
            fn as_empty_value() -> Value {
                T::as_empty_value()
            }

            fn as_value(self) -> Value {
                (*self).as_value()
            }
        }
    };
}
impl_as_value!(Box);

impl<T: AsValue> AsValue for Vec<T> {
    fn as_value(self) -> Value {
        Value::List(
            Some(self.into_iter().map(AsValue::as_value).collect()),
            Box::new(T::as_empty_value()),
        )
    }

    fn as_empty_value() -> Value {
        Value::List(None, Box::new(T::as_empty_value()))
    }
}

impl<T: AsValue> From<T> for Value {
    fn from(value: T) -> Self {
        value.as_value()
    }
}
