use crate::{Expression, OpPrecedence, Value};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};

pub trait ColumnTrait {
    fn column_def(&self) -> &ColumnDef;
    fn column_ref(&self) -> &ColumnRef;
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
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

#[derive(Default, Debug)]
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

impl OpPrecedence for ColumnRef {
    fn precedence(&self, _writer: &dyn crate::SqlWriter) -> i32 {
        1_000_000
    }
}

impl Expression for ColumnRef {
    fn write_query(&self, writer: &dyn crate::SqlWriter, out: &mut String, qualify_columns: bool) {
        writer.write_column_ref(out, self, qualify_columns);
    }
}

impl OpPrecedence for ColumnDef {
    fn precedence(&self, _writer: &dyn crate::SqlWriter) -> i32 {
        1_000_000
    }
}

impl Expression for ColumnDef {
    fn write_query(&self, writer: &dyn crate::SqlWriter, out: &mut String, qualify_columns: bool) {
        writer.write_column_ref(out, &self.reference, qualify_columns);
    }
}
