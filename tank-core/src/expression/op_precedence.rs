use crate::{Expression, SqlWriter, Value};

pub trait OpPrecedence {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32;
}

impl OpPrecedence for &dyn Expression {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32 {
        todo!()
    }
}

impl OpPrecedence for () {
    fn precedence(&self, _writer: &dyn SqlWriter) -> i32 {
        1_000_000_000
    }
}

impl OpPrecedence for Value {
    fn precedence(&self, _writer: &dyn SqlWriter) -> i32 {
        0
    }
}
