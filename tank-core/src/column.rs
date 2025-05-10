use crate::{quote_cow, Value};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use std::borrow::Cow;

pub trait ColumnTrait {
    fn column_def(&self) -> &ColumnDef;
    fn column_ref(&self) -> &ColumnRef;
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ColumnRef {
    pub name: Cow<'static, str>,
    pub table: Cow<'static, str>,
    pub schema: Cow<'static, str>,
}

pub enum DefaultValue {
    Value(Value),
    Custom(String),
}

#[derive(Default, Debug, Clone)]
pub struct ColumnDef {
    pub reference: ColumnRef,
    pub value: Value,
    pub nullable: bool,
    pub default: Option<String>,
    pub primary_key: bool,
    pub unique: bool,
    pub column_type: Cow<'static, str>,
}
impl ColumnDef {
    pub fn name(&self) -> &Cow<'static, str> {
        &self.reference.name
    }
    pub fn table(&self) -> &Cow<'static, str> {
        &self.reference.table
    }
    pub fn schema(&self) -> &Cow<'static, str> {
        &self.reference.schema
    }
}

impl From<&ColumnRef> for ColumnRef {
    fn from(value: &ColumnRef) -> Self {
        value.clone()
    }
}

impl From<ColumnDef> for ColumnRef {
    fn from(value: ColumnDef) -> Self {
        value.reference
    }
}

impl<'a> From<&'a ColumnDef> for &'a ColumnRef {
    fn from(value: &'a ColumnDef) -> Self {
        &value.reference
    }
}

impl ToTokens for ColumnRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = quote_cow(&self.name);
        let table = quote_cow(&self.table);
        let schema = quote_cow(&self.schema);
        tokens.append_all(quote! {
            ::tank::ColumnRef {
                name: #name,
                table: #table,
                schema: #schema,
            }
        });
    }
}

impl ToTokens for ColumnDef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let reference = &self.reference;
        let value = self.value.to_token_stream();
        let nullable = self.nullable;
        let default = match &self.default {
            Some(v) => quote! {Some(#v)},
            None => quote! {None},
        };
        let column_type = quote_cow(&self.column_type);
        let primary_key = &self.primary_key;
        let unique = &self.unique;
        tokens.append_all(quote! {
            ::tank::ColumnDef {
                reference: #reference,
                value: #value,
                nullable: #nullable,
                default: #default,
                column_type: #column_type,
                primary_key: #primary_key,
                unique: #unique,
            }
        });
    }
}
