use crate::{Expression, OpPrecedence, SqlWriter};

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
