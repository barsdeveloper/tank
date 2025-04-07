use crate::Value;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: Cow<'static, str>,
    pub value: Value,
    pub nullable: bool,
    // /// `DEFAULT <restricted-expr>`
    // pub default: Option<Expr>,
    // /// `{ PRIMARY KEY | UNIQUE }`
    // pub unique: Option<ColumnUniqueOption>,
    // pub comment: Option<String>,
}

impl ToString for ColumnDef {
    fn to_string(&self) -> String {
        let mut result = format!("{} {}", self.name, "self.data_type");
        if !self.nullable {
            result.push_str(" NOT NULL");
        }
        result
    }
}

impl ToTokens for ColumnDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match &self.name {
            Cow::Borrowed(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
            Cow::Owned(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
        };
        let value = self.value.to_token_stream();
        let nullable = self.nullable;
        tokens.append_all(quote! {
            ::tank::ColumnDef {
                name: #name,
                value: #value,
                nullable: #nullable,
            }
        });
    }
}
