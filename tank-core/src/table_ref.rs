use crate::{quote_cow, DataSet};
use quote::{quote, ToTokens, TokenStreamExt};
use std::borrow::Cow;

pub struct TableRef {
    pub name: Cow<'static, str>,
    pub schema: Cow<'static, str>,
    pub alias: Cow<'static, str>,
}

impl TableRef {
    pub fn full_name(&self) -> String {
        let mut result = String::new();
        if !self.alias.is_empty() {
            result.push_str(&self.alias);
        } else {
            if !self.schema.is_empty() {
                result.push_str(&self.schema);
                result.push('.');
            }
            result.push_str(&self.name);
        }
        result
    }
}

impl ToTokens for TableRef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = quote_cow(&self.name);
        let schema = quote_cow(&self.schema);
        let alias = quote_cow(&self.alias);
        tokens.append_all(quote! {
            ::tank::ColumnRef {
                name: #name,
                schema: #schema,
                alias: #alias,
            }
        });
    }
}

impl DataSet for TableRef {
    const QUALIFIED_COLUMNS: bool = false;
    fn sql_write<'a, W: crate::SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
    ) -> &'a mut String {
        writer.sql_table_ref(out, self)
    }
}
