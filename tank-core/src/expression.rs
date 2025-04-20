use crate::{ColumnDef, SqlWriter, Value};

pub trait OpPrecedence {
    fn precedence<W: SqlWriter + ?Sized>(&self, writer: &W) -> i32;
}

pub trait Expression: OpPrecedence {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
    ) -> &'a mut String;
}

#[derive(Debug)]
pub enum Operand {
    // Function(String, Vec<String>),
    LitBool(bool),
    LitFloat(f64),
    LitIdent(&'static str),
    LitInt(i128),
    LitStr(&'static str),
    LitArray(&'static [Operand]),
    Column(ColumnDef),
    Type(Value),
}
impl OpPrecedence for Operand {
    fn precedence<W: SqlWriter + ?Sized>(&self, _writer: &W) -> i32 {
        0
    }
}
impl Expression for Operand {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
    ) -> &'a mut String {
        writer.sql_expression_operand(out, self)
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
            (Self::Column(l), Self::Column(r)) => l.name == r.name && l.value.same_type(&r.value),
            (Self::Type(l), Self::Type(r)) => l.same_type(r),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UnaryOpType {
    Negative,
    Not,
}
impl OpPrecedence for UnaryOpType {
    fn precedence<W: SqlWriter + ?Sized>(&self, writer: &W) -> i32 {
        writer.expression_unary_op_precedence(self)
    }
}

#[derive(Debug, PartialEq)]
pub enum BinaryOpType {
    ArrayIndexing,
    Cast,
    Multiplication,
    Division,
    Remainder,
    Addition,
    Subtraction,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseOr,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}
impl OpPrecedence for BinaryOpType {
    fn precedence<W: SqlWriter + ?Sized>(&self, writer: &W) -> i32 {
        writer.expression_binary_op_precedence(self)
    }
}

pub struct UnaryOp<V: Expression> {
    pub op: UnaryOpType,
    pub v: V,
}
impl<E: Expression> OpPrecedence for UnaryOp<E> {
    fn precedence<W: SqlWriter + ?Sized>(&self, writer: &W) -> i32 {
        writer.expression_unary_op_precedence(&self.op)
    }
}
impl<E: Expression> Expression for UnaryOp<E> {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
    ) -> &'a mut String {
        writer.sql_expression_unary_op(out, self)
    }
}

pub struct BinaryOp<L: Expression, R: Expression> {
    pub op: BinaryOpType,
    pub lhs: L,
    pub rhs: R,
}
impl<L: Expression, R: Expression> OpPrecedence for BinaryOp<L, R> {
    fn precedence<W: SqlWriter + ?Sized>(&self, writer: &W) -> i32 {
        writer.expression_binary_op_precedence(&self.op)
    }
}
impl<L: Expression, R: Expression> Expression for BinaryOp<L, R> {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
    ) -> &'a mut String {
        writer.sql_expression_binary_op(out, self)
    }
}

impl OpPrecedence for Value {
    fn precedence<W: SqlWriter + ?Sized>(&self, _writer: &W) -> i32 {
        0
    }
}
impl Expression for Value {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        _writer: &W,
        out: &'a mut String,
    ) -> &'a mut String {
        out
    }
}
