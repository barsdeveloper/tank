use crate::{ColumnRef, Expression, OpPrecedence, SqlWriter, Value};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

#[derive(Debug, Clone)]
pub enum Operand {
    // Function(String, Vec<String>),
    LitBool(bool),
    LitFloat(f64),
    LitIdent(&'static str),
    LitField(&'static [&'static str]),
    LitInt(i128),
    LitStr(&'static str),
    LitArray(&'static [Operand]),
    Null,
    Column(ColumnRef),
    Type(Value),
    Variable(Value),
}

impl OpPrecedence for Operand {
    fn precedence<W: SqlWriter + ?Sized>(&self, _writer: &W) -> i32 {
        1_000_000_000
    }
}

impl Expression for Operand {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
        qualify_columns: bool,
    ) -> &'a mut String {
        writer.sql_expression_operand(out, self, qualify_columns)
    }
}
impl PartialEq for Operand {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LitBool(l), Self::LitBool(r)) => l == r,
            (Self::LitFloat(l), Self::LitFloat(r)) => l == r,
            (Self::LitIdent(l), Self::LitIdent(r)) => l == r,
            (Self::LitInt(l), Self::LitInt(r)) => l == r,
            (Self::LitStr(l), Self::LitStr(r)) => l == r,
            (Self::LitArray(l), Self::LitArray(r)) => l == r,
            (Self::Column(l), Self::Column(r)) => l == r,
            (Self::Type(l), Self::Type(r)) => l.same_type(r),
            _ => false,
        }
    }
}

impl ToTokens for Operand {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Operand::*;
        tokens.append_all(match self {
            LitBool(v) => quote!(::tank::Operand::LitBool(#v)),
            LitFloat(v) => quote!(::tank::Operand::LitFloat(#v)),
            LitIdent(v) => quote!(::tank::Operand::LitIdent(#v)),
            LitField(v) => quote!(::tank::Operand::LitField(#(#v),*)),
            LitInt(v) => quote!(::tank::Operand::LitInt(#v)),
            LitStr(v) => quote!(::tank::Operand::LitStr(#v)),
            LitArray(v) => quote!(::tank::Operand::LitArray([#(#v),*])),
            Null => quote!(::tank::Operand::Null),
            Column(v) => quote!(::tank::Operand::Column(#v)),
            Type(v) => quote!(::tank::Operand::Type(#v)),
            Variable(v) => quote!(::tank::Operand::Variable(#v)),
        })
    }
}
