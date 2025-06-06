use crate::{ColumnRef, SqlWriter, Value};

pub trait OpPrecedence {
    fn precedence<W: SqlWriter + ?Sized>(&self, writer: &W) -> i32;
}

impl OpPrecedence for () {
    fn precedence<W: SqlWriter + ?Sized>(&self, _writer: &W) -> i32 {
        1_000_000_000
    }
}

pub trait Expression: OpPrecedence + Send {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
        qualify_columns: bool,
    ) -> &'a mut String;
}

impl Expression for () {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        _writer: &W,
        out: &'a mut String,
        _qualify_columns: bool,
    ) -> &'a mut String {
        out
    }
}

#[derive(Debug)]
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
    Indexing,
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
    Is,
    IsNot,
    Like,
    NotLike,
    Regexp,
    NotRegexpr,
    Glob,
    NotGlob,
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
        qualify_columns: bool,
    ) -> &'a mut String {
        writer.sql_expression_unary_op(out, self, qualify_columns)
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
        qualify_columns: bool,
    ) -> &'a mut String {
        writer.sql_expression_binary_op(out, self, qualify_columns)
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
        _qualify_columns: bool,
    ) -> &'a mut String {
        out
    }
}
