use quote::{quote, ToTokens};
use rust_decimal::Decimal;
use std::collections::HashMap;
use syn::{GenericArgument, PathArguments, Type, TypePath};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use uuid::Uuid;

use crate::interval::Interval;

#[derive(Debug, Clone)]
pub enum Value {
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
        Option<HashMap<Value, Value>>,
        /* key: */ Box<Value>,
        /* value: */ Box<Value>,
    ),
}

pub fn decode_type(type_path: &TypePath) -> (Value, bool) {
    let path = type_path.path.segments.last().unwrap();
    let ident = &path.ident;
    let mut nullable = ident == "Option";
    let data_type = if ident == "String" {
        Value::Varchar(None)
    } else if ident == "i8" {
        Value::Int8(None)
    } else if ident == "i16" {
        Value::Int16(None)
    } else if ident == "i32" {
        Value::Int32(None)
    } else if ident == "i64" {
        Value::Int64(None)
    } else if ident == "i128" {
        Value::Int128(None)
    } else if ident == "u8" {
        Value::UInt8(None)
    } else if ident == "u16" {
        Value::UInt16(None)
    } else if ident == "u32" {
        Value::UInt32(None)
    } else if ident == "u64" {
        Value::UInt64(None)
    } else if ident == "u128" {
        Value::UInt128(None)
    } else if ident == "f32" {
        Value::Float32(None)
    } else if ident == "f64" {
        Value::Float64(None)
    } else if ident == "Time" {
        Value::Time(None)
    } else if ident == "Date" {
        Value::Date(None)
    } else if ident == "Vec" {
        match &path.arguments {
            PathArguments::AngleBracketed(bracketed) => {
                if let GenericArgument::Type(Type::Path(type_path)) =
                    bracketed.args.first().unwrap()
                {
                    let nested_type = decode_type(&type_path);
                    Value::List(None, Box::new(nested_type.0))
                } else {
                    panic!("{} must have a type as the first generic argument", ident)
                }
            }
            _ => panic!("{} must have a generic argument", ident),
        }
    } else if ident == "Option" || ident == "Box" {
        match &path.arguments {
            PathArguments::AngleBracketed(bracketed) => {
                if let GenericArgument::Type(Type::Path(type_path)) =
                    bracketed.args.first().unwrap()
                {
                    let nested_type = decode_type(&type_path);
                    nullable = nullable || nested_type.1;
                    nested_type.0
                } else {
                    panic!("{} must have a type as the first generic argument", ident)
                }
            }
            _ => panic!("{} must have a generic argument", ident),
        }
    } else {
        panic!("Unknown type \"{}\"", ident)
    };
    (data_type, nullable)
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ts = match self {
            Value::Boolean(value) => quote! { ::tank::Value::Boolean(#value.into()) },
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
                quote! { ::std::collections::HashMap<#key, #value> }
            }
        };
        tokens.extend(ts);
    }
}
