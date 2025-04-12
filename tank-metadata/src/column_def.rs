use crate::Value;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::borrow::Cow;

pub enum DefaultValue {
    Value(Value),
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: Cow<'static, str>,
    pub value: Value,
    pub nullable: bool,
    pub default: Option<String>,
    // /// `{ PRIMARY KEY | UNIQUE }`
    // pub unique: Option<ColumnUniqueOption>,
    // pub comment: Option<String>,
    pub column_type: String,
}

impl ToTokens for ColumnDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match &self.name {
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
        tokens.append_all(quote! {
            ::tank::ColumnDef {
                name: #name,
                value: #value,
                nullable: #nullable,
                default: #default,
                column_type: #column_type.into(),
            }
        });
    }
}
