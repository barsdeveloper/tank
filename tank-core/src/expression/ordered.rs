use crate::{
    Expression, OpPrecedence,
    writer::{Context, SqlWriter},
};
use std::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub enum Order {
    ASC,
    DESC,
}

#[derive(Debug)]
pub struct Ordered<E: Expression> {
    pub order: Order,
    pub expression: E,
}

impl<E: Expression> OpPrecedence for Ordered<E> {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32 {
        self.expression.precedence(writer)
    }
}

impl<E: Expression> Expression for Ordered<E> {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, out: &mut dyn Write) {
        self.expression.write_query(writer, context, out)
    }
    fn is_ordered(&self) -> bool {
        true
    }
}
