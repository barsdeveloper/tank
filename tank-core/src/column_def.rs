use crate::Value;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::borrow::Cow;

pub enum DefaultValue {
    Value(Value),
    Custom(String),
}

#[derive(Default, Debug, Clone)]
pub struct ColumnDef {
    pub name: Cow<'static, str>,
    pub table_name: Cow<'static, str>,
    pub schema_name: Cow<'static, str>,
    pub value: Value,
    pub nullable: bool,
    pub default: Option<String>,
    pub primary_key: bool,
    pub unique: bool,
    pub column_type: String,
}

impl ColumnDef {
    pub fn full_name(&self) -> String {
        let mut s = String::new();
        if !self.schema_name.is_empty() {
            s.push_str(&self.schema_name);
            s.push('.');
        }
        if !self.table_name.is_empty() {
            s.push_str(&self.table_name);
            s.push('.');
        }
        s.push_str(&self.name);
        s
    }
}

impl ToTokens for ColumnDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match &self.name {
            Cow::Borrowed(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
            Cow::Owned(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
        };
        let table_name = match &self.table_name {
            Cow::Borrowed(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
            Cow::Owned(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
        };
        let schema_name = match &self.schema_name {
            Cow::Borrowed(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
            Cow::Owned(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
        };
        let value = self.value.to_token_stream();
        let nullable = self.nullable;
        let default = match &self.default {
            Some(v) => quote! {Some(#v)},
            None => quote! {None},
        };
        let column_type = &self.column_type;
        let primary_key = &self.primary_key;
        let unique = &self.unique;
        tokens.append_all(quote! {
            ::tank::ColumnDef {
                name: #name,
                table_name: #table_name,
                schema_name: #schema_name,
                value: #value,
                nullable: #nullable,
                default: #default,
                column_type: #column_type.into(),
                primary_key: #primary_key,
                unique: #unique,
            }
        });
    }
}
