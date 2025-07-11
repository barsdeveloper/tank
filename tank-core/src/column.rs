use crate::{Expression, Value};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};

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

#[derive(Default)]
pub struct ColumnDef {
    pub reference: ColumnRef,
    pub column_type: &'static str,
    pub value: Value,
    pub nullable: bool,
    pub default: Option<Box<dyn Expression>>,
    pub primary_key: PrimaryKeyType,
    pub unique: bool,
    pub passive: bool,
    pub comment: &'static str,
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

impl<'a> From<&'a ColumnDef> for &'a ColumnRef {
    fn from(value: &'a ColumnDef) -> Self {
        &value.reference
    }
}
