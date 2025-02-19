use quote::{quote, ToTokens};
use syn::{GenericArgument, PathArguments, Type, TypePath};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DataType {
    Varchar,
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Float32,
    Float64,
    Double,
    Decimal(u8, u8),
    Byte,
    Date,
    Time,
    Timestamp,
    TimestampWithTimezone,
    Interval,
    Uuid,
    Json,
    Array(Box<DataType>, u8),
    List(Box<DataType>),
    Map(Box<DataType>, Box<DataType>),
}

pub fn decode_type(type_path: &TypePath) -> (DataType, bool) {
    let path = type_path.path.segments.last().unwrap();
    let ident = &path.ident;
    let mut nullable = ident == "Option";
    let data_type = if ident == "String" {
        DataType::Varchar
    } else if ident == "i8" {
        DataType::Int8
    } else if ident == "i16" {
        DataType::Int16
    } else if ident == "i32" {
        DataType::Int32
    } else if ident == "i64" {
        DataType::Int64
    } else if ident == "i128" {
        DataType::Int128
    } else if ident == "u8" {
        DataType::UInt8
    } else if ident == "u16" {
        DataType::UInt16
    } else if ident == "u32" {
        DataType::UInt32
    } else if ident == "u64" {
        DataType::UInt64
    } else if ident == "u128" {
        DataType::UInt128
    } else if ident == "f32" {
        DataType::Float32
    } else if ident == "f64" {
        DataType::Float64
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
    } else if ident == "Vec" {
        match &path.arguments {
            PathArguments::AngleBracketed(bracketed) => {
                if let GenericArgument::Type(Type::Path(type_path)) =
                    bracketed.args.first().unwrap()
                {
                    let nested_type = decode_type(&type_path);
                    DataType::List(Box::new(nested_type.0))
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

impl ToTokens for DataType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ts = match self {
            DataType::Varchar => quote! { ::tank::DataType::Varchar },
            DataType::Boolean => quote! { ::tank::DataType::Boolean },
            DataType::Int8 => quote! { ::tank::DataType::Int8 },
            DataType::Int16 => quote! { ::tank::DataType::Int16 },
            DataType::Int32 => quote! { ::tank::DataType::Int32 },
            DataType::Int64 => quote! { ::tank::DataType::Int64 },
            DataType::Int128 => quote! { ::tank::DataType::Int128 },
            DataType::UInt8 => quote! { ::tank::DataType::UInt8 },
            DataType::UInt16 => quote! { ::tank::DataType::UInt16 },
            DataType::UInt32 => quote! { ::tank::DataType::UInt32 },
            DataType::UInt64 => quote! { ::tank::DataType::UInt64 },
            DataType::UInt128 => quote! { ::tank::DataType::UInt128 },
            DataType::Float32 => quote! { ::tank::DataType::Float32 },
            DataType::Float64 => quote! { ::tank::DataType::Float64 },
            DataType::Double => quote! { ::tank::DataType::Double },
            DataType::Decimal(precision, scale) => {
                quote! { ::tank::DataType::Decimal(#precision, #scale) }
            }
            DataType::Byte => quote! { ::tank::DataType::Byte },
            DataType::Date => quote! { ::tank::DataType::Date },
            DataType::Time => quote! { ::tank::DataType::Time },
            DataType::Timestamp => quote! { ::tank::DataType::Timestamp },
            DataType::TimestampWithTimezone => {
                quote! { ::tank::DataType::TimestampWithTimezone }
            }
            DataType::Interval => quote! { ::tank::DataType::Interval },
            DataType::Uuid => quote! { ::tank::DataType::Uuid },
            DataType::Json => quote! { ::tank::DataType::Json },
            DataType::Array(inner, size) => quote! { ::tank::DataType::Array (#inner, #size) },
            DataType::List(inner) => {
                let inner = inner.as_ref().to_token_stream();
                quote! { ::tank::DataType::List(Box::new(#inner)) }
            }
            DataType::Map(key, value) => {
                let key = key.as_ref().to_token_stream();
                let value = value.as_ref().to_token_stream();
                quote! { ::std::collections::HashMap<#key, #value> }
            }
        };
        tokens.extend(ts);
    }
}
