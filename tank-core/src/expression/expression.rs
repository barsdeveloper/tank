use crate::{
    OpPrecedence,
    writer::{Context, SqlWriter},
};
use std::fmt::Debug;

pub trait Expression: OpPrecedence + Send + Sync + Debug {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut String);
    fn is_ordered(&self) -> bool {
        false
    }
}

impl<T: Expression> Expression for &T {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut String) {
        (*self).write_query(writer, context, buff);
    }
    fn is_ordered(&self) -> bool {
        (*self).is_ordered()
    }
}

impl Expression for &dyn Expression {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut String) {
        (*self).write_query(writer, context, buff);
    }
    fn is_ordered(&self) -> bool {
        (*self).is_ordered()
    }
}

impl Expression for () {
    fn write_query(&self, _writer: &dyn SqlWriter, _context: Context, _buff: &mut String) {}
}

impl Expression for bool {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut String) {
        writer.write_value_bool(context, buff, *self);
    }
}

impl<'a, T: Expression> From<&'a T> for &'a dyn Expression {
    fn from(value: &'a T) -> Self {
        value as &'a dyn Expression
    }
}
