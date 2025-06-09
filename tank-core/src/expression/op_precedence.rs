use crate::{SqlWriter, Value};

pub trait OpPrecedence {
    fn precedence<W: SqlWriter + ?Sized>(&self, writer: &W) -> i32;
}

impl OpPrecedence for () {
    fn precedence<W: SqlWriter + ?Sized>(&self, _writer: &W) -> i32 {
        1_000_000_000
    }
}

impl OpPrecedence for Value {
    fn precedence<W: SqlWriter + ?Sized>(&self, _writer: &W) -> i32 {
        0
    }
}
