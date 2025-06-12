use crate::{Expression, OpPrecedence, SqlWriter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpType {
    Negative,
    Not,
}
impl OpPrecedence for UnaryOpType {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32 {
        writer.expression_unary_op_precedence(self)
    }
}

pub struct UnaryOp<V: Expression> {
    pub op: UnaryOpType,
    pub v: V,
}

impl<E: Expression> OpPrecedence for UnaryOp<E> {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32 {
        writer.expression_unary_op_precedence(&self.op)
    }
}

impl<E: Expression> Expression for UnaryOp<E> {
    fn sql_write<'a>(
        &self,
        writer: &dyn SqlWriter,
        out: &'a mut String,
        qualify_columns: bool,
    ) -> &'a mut String {
        writer.sql_expression_unary_op(
            out,
            &UnaryOp {
                op: self.op,
                v: &self.v,
            },
            qualify_columns,
        )
    }
}
