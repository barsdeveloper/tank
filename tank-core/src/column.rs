use crate::Value;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

pub trait ColumnTrait {
    fn column_def(&self) -> &ColumnDef;
    fn column_ref(&self) -> &ColumnRef;
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ColumnRef {
    pub name: &'static str,
    pub table: &'static str,
    pub schema: &'static str,
}

pub enum DefaultValue {
    Value(Value),
    Custom(String),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryKeyType {
    PrimaryKey,
    PartOfPrimaryKey,
    #[default]
    None,
}

impl ToTokens for PrimaryKeyType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use PrimaryKeyType::*;
        tokens.append_all(match self {
            PrimaryKey => quote!(::tank::PrimaryKeyType::PrimaryKey),
            PartOfPrimaryKey => quote!(::tank::PrimaryKeyType::PartOfPrimaryKey),
            None => quote!(::tank::PrimaryKeyType::None),
        });
    }
}

#[derive(Default, Debug, Clone)]
pub struct ColumnDef {
    pub reference: ColumnRef,
    pub column_type: &'static str,
    pub value: Value,
    pub nullable: bool,
    pub default: Option<String>,
    pub primary_key: PrimaryKeyType,
    pub unique: bool,
    pub auto_increment: bool,
    pub passive: bool,
}

impl ColumnDef {
    pub fn name(&self) -> &'static str {
        &self.reference.name
    }
    pub fn table(&self) -> &'static str {
        &self.reference.table
    }
    pub fn schema(&self) -> &'static str {
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
        let name = self.name;
        let table = self.table;
        let schema = self.schema;
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
        let column_type = self.column_type;
        let value = self.value.to_token_stream();
        let nullable = self.nullable;
        let default = match &self.default {
            Some(v) => quote! {Some(#v)},
            None => quote! {None},
        };
        let primary_key = &self.primary_key;
        let unique = &self.unique;
        let auto_increment = &self.auto_increment;
        let passive = &self.passive;
        tokens.append_all(quote! {
            ::tank::ColumnDef {
                reference: #reference,
                column_type: #column_type,
                value: #value,
                nullable: #nullable,
                default: #default,
                primary_key: #primary_key,
                unique: #unique,
                auto_increment: #auto_increment,
                passive: #passive,
            }
        });
    }
}
