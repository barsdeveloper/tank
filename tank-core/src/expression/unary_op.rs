use crate::{
    Expression, OpPrecedence,
    writer::{Context, SqlWriter},
};
use std::fmt::Write;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UnaryOpType {
    Negative,
    Not,
}
impl OpPrecedence for UnaryOpType {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32 {
        writer.expression_unary_op_precedence(self)
    }
}

#[derive(Debug)]
pub struct UnaryOp<V: Expression> {
    pub op: UnaryOpType,
    pub arg: V,
}

impl<E: Expression> OpPrecedence for UnaryOp<E> {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32 {
        writer.expression_unary_op_precedence(&self.op)
    }
}

impl<E: Expression> Expression for UnaryOp<E> {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut dyn Write) {
        writer.write_expression_unary_op(
            context,
            buff,
            &UnaryOp {
                op: self.op,
                arg: &self.arg,
            },
        )
    }
}
