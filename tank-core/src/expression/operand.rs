use crate::{
    Expression, OpPrecedence, Value,
    writer::{Context, SqlWriter},
};
use std::fmt::Write;

#[derive(Debug)]
pub enum Operand<'a> {
    LitBool(bool),
    LitFloat(f64),
    LitIdent(&'a str),
    LitField(&'a [&'a str]),
    LitInt(i128),
    LitStr(&'a str),
    LitArray(&'a [Operand<'a>]),
    Null,
    Type(Value),
    Variable(Value),
    Call(&'static str, &'a [&'a dyn Expression]),
    Asterisk,
    QuestionMark,
}

impl OpPrecedence for Operand<'_> {
    fn precedence(&self, _writer: &dyn SqlWriter) -> i32 {
        1_000_000
    }
}

impl Expression for Operand<'_> {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut dyn Write) {
        writer.write_expression_operand(context, buff, self)
    }
}

impl PartialEq for Operand<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LitBool(l), Self::LitBool(r)) => l == r,
            (Self::LitFloat(l), Self::LitFloat(r)) => l == r,
            (Self::LitIdent(l), Self::LitIdent(r)) => l == r,
            (Self::LitInt(l), Self::LitInt(r)) => l == r,
            (Self::LitStr(l), Self::LitStr(r)) => l == r,
            (Self::LitArray(l), Self::LitArray(r)) => l == r,
            (Self::Type(l), Self::Type(r)) => l.same_type(r),
            _ => false,
        }
    }
}
