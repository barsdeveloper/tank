use crate::{Expression, OpPrecedence, writer::SqlWriter};

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
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String, qualify_columns: bool) {
        self.expression.write_query(writer, out, qualify_columns)
    }
}
