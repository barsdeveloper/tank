use crate::{quote_cow, DataSet, SqlWriter};
use quote::{quote, ToTokens, TokenStreamExt};
use std::borrow::Cow;

#[derive(Clone)]
pub struct TableRef {
    pub name: &'static str,
    pub schema: &'static str,
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
    pub fn with_alias(&self, alias: Cow<'static, str>) -> Self {
        let mut result = self.clone();
        result.alias = alias.into();
        result
    }
}

impl ToTokens for TableRef {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let schema = &self.schema;
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
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        false
    }
    fn sql_write<'a>(&self, writer: &dyn SqlWriter, out: &'a mut String) -> &'a mut String {
        writer.sql_table_ref(out, self)
    }
}
