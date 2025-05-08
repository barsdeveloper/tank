use quote::{quote, ToTokens, TokenStreamExt};
use std::borrow::Cow;

use crate::DataSet;

pub struct TableRef {
    pub name: Cow<'static, str>,
    pub schema: Cow<'static, str>,
}

impl TableRef {
    pub fn full_name(&self) -> String {
        let mut result = String::new();
        if !self.schema.is_empty() {
            result.push_str(&self.schema);
            result.push('.');
        }
        result.push_str(&self.name);
        result
    }
}

impl ToTokens for TableRef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = match &self.name {
            Cow::Borrowed(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
            Cow::Owned(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
        };
        let schema = match &self.schema {
            Cow::Borrowed(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
            Cow::Owned(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
        };
        tokens.append_all(quote! {
            ::tank::ColumnRef {
                name: #name,
                schema: #schema,
            }
        });
    }
}

impl DataSet for TableRef {}
impl<T: DataSet> DataSet for &T {}
