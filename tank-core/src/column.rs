use crate::{Expression, OpPrecedence, TableRef, Value, writer::Context};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};

pub trait ColumnTrait {
    fn column_def(&self) -> &ColumnDef;
    fn column_ref(&self) -> &ColumnRef;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnRef {
    pub name: &'static str,
    pub table: &'static str,
    pub schema: &'static str,
}

impl ColumnRef {
    pub fn table_ref(&self) -> TableRef {
        TableRef {
            name: self.table,
            schema: self.schema,
            ..Default::default()
        }
    }
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

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Action {
    #[default]
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}

impl ToTokens for Action {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(match self {
            Action::NoAction => quote! { ::tank::Action::NoAction },
            Action::Restrict => quote! { ::tank::Action::Restrict },
            Action::Cascade => quote! { ::tank::Action::Cascade },
            Action::SetNull => quote! { ::tank::Action::SetNull },
            Action::SetDefault => quote! { ::tank::Action::SetDefault },
        });
    }
}

#[derive(Default, Debug)]
pub struct ColumnDef {
    pub column_ref: ColumnRef,
    pub column_type: &'static str,
    pub value: Value,
    pub nullable: bool,
    pub default: Option<Box<dyn Expression>>,
    pub primary_key: PrimaryKeyType,
    pub unique: bool,
    pub references: Option<ColumnRef>,
    pub on_delete: Option<Action>,
    pub on_update: Option<Action>,
    pub passive: bool,
    pub comment: &'static str,
}

impl ColumnDef {
    pub fn name(&self) -> &'static str {
        &self.column_ref.name
    }
    pub fn table(&self) -> &'static str {
        &self.column_ref.table
    }
    pub fn schema(&self) -> &'static str {
        &self.column_ref.schema
    }
}

impl<'a> From<&'a ColumnDef> for &'a ColumnRef {
    fn from(value: &'a ColumnDef) -> Self {
        &value.column_ref
    }
}

impl OpPrecedence for ColumnRef {
    fn precedence(&self, _writer: &dyn crate::SqlWriter) -> i32 {
        1_000_000
    }
}

impl Expression for ColumnRef {
    fn write_query(&self, writer: &dyn crate::SqlWriter, context: &mut Context, buff: &mut String) {
        writer.write_column_ref(context, buff, self);
    }
}

impl OpPrecedence for ColumnDef {
    fn precedence(&self, _writer: &dyn crate::SqlWriter) -> i32 {
        1_000_000
    }
}

impl Expression for ColumnDef {
    fn write_query(&self, writer: &dyn crate::SqlWriter, context: &mut Context, buff: &mut String) {
        writer.write_column_ref(context, buff, &self.column_ref);
    }
}
