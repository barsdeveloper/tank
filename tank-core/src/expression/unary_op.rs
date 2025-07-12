use crate::{Expression, OpPrecedence, SqlWriter};

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
    pub v: V,
}

impl<E: Expression> OpPrecedence for UnaryOp<E> {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32 {
        writer.expression_unary_op_precedence(&self.op)
    }
}

impl<E: Expression> Expression for UnaryOp<E> {
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String, qualify_columns: bool) {
        writer.write_expression_unary_op(
            out,
            &UnaryOp {
                op: self.op,
                v: &self.v,
            },
            qualify_columns,
        )
    }
}
