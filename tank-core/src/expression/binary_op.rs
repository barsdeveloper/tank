use crate::{Expression, OpPrecedence, SqlWriter};

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
