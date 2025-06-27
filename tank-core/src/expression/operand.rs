use crate::{ColumnRef, Expression, OpPrecedence, SqlWriter, Value};

pub enum Operand {
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
    Call(&'static [Operand]),
}

impl OpPrecedence for Operand {
    fn precedence(&self, _writer: &dyn SqlWriter) -> i32 {
        1_000_000_000
    }
}

impl Expression for Operand {
    fn sql_write<'a>(
        &self,
        writer: &dyn SqlWriter,
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
