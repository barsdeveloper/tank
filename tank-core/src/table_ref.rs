use crate::{
    DataSet, quote_cow,
    writer::{Context, SqlWriter},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};
use std::{borrow::Cow, fmt::Write};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
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

impl DataSet for TableRef {
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        false
    }
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, out: &mut dyn Write) {
        writer.write_table_ref(context, out, self)
    }
}

impl DataSet for &TableRef {
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        false
    }
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, out: &mut dyn Write) {
        (*writer).write_table_ref(context, out, self)
    }
}

impl ToTokens for TableRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
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

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct DeclareTableRef(pub TableRef);

impl DataSet for DeclareTableRef {
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        false
    }
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, out: &mut dyn Write) {
        writer.write_table_ref(context, out, &self.0)
    }
}

impl ToTokens for DeclareTableRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table_ref = &self.0;
        tokens.append_all(quote!(::tank::DeclareTableRef(#table_ref)));
    }
}
