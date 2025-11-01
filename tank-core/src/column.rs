use crate::{Expression, OpPrecedence, TableRef, Value, writer::Context};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};

/// Helper trait for types that expose an underlying column definition and reference.
pub trait ColumnTrait {
    /// Logical definition (column metadata).
    fn column_def(&self) -> &ColumnDef;
    /// Reference used in expressions.
    fn column_ref(&self) -> &ColumnRef;
}

/// Fully-ì-qualified reference to a table column.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnRef {
    /// Column name.
    pub name: &'static str,
    /// Table name.
    pub table: &'static str,
    /// Schema name (may be empty).
    pub schema: &'static str,
}

impl ColumnRef {
    pub fn table(&self) -> TableRef {
        TableRef {
            name: self.table,
            schema: self.schema,
            ..Default::default()
        }
    }
}

/// Indicates how (or if) a column participates in the primary key.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryKeyType {
    /// Single-column primary key.
    PrimaryKey,
    /// Member of a composite primary key.
    PartOfPrimaryKey,
    /// Not part of the primary key.
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

/// Referential action for foreign key updates / deletes.
#[derive(Default, Debug, PartialEq, Eq)]
pub enum Action {
    /// No special action.
    #[default]
    NoAction,
    /// Reject the operation.
    Restrict,
    /// Propagate delete/update.
    Cascade,
    /// Set referencing columns to NULL.
    SetNull,
    /// Apply column DEFAULT.
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

/// Declarative specification of a table column.
#[derive(Default, Debug)]
pub struct ColumnDef {
    /// Column identity.
    pub column_ref: ColumnRef,
    /// Explicit SQL type override (empty => infer from `value`).
    pub column_type: &'static str,
    /// `Value` describing column type and shape (arrays/maps/decimal precision).
    pub value: Value,
    /// Nullability flag.
    pub nullable: bool,
    /// Default value (expression rendered by `SqlWriter`).
    pub default: Option<Box<dyn Expression>>,
    /// Primary key participation.
    pub primary_key: PrimaryKeyType,
    /// Unique constraint (single column only, composite handled in the `TableDef`).
    pub unique: bool,
    /// Foreign key target column.
    pub references: Option<ColumnRef>,
    /// Action for deletes.
    pub on_delete: Option<Action>,
    /// Action for updates.
    pub on_update: Option<Action>,
    /// Passive columns are skipped when generating `INSERT` value lists (DEFAULT used).
    pub passive: bool,
    /// Optional human-readable comment.
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
