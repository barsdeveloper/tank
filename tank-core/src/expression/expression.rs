use crate::{
    OpPrecedence, Value,
    writer::{Context, SqlWriter},
};
use std::fmt::Debug;

/// A renderable SQL expression node.
pub trait Expression: OpPrecedence + Send + Sync + Debug {
    /// Serialize the expression into the buffer using the dialect writer.
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, buff: &mut String);
    /// Whether this expression carries ordering (ASC/DESC) information.
    fn is_ordered(&self) -> bool {
        false
    }
}

impl<T: Expression> Expression for &T {
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, buff: &mut String) {
        (*self).write_query(writer, context, buff);
    }
    fn is_ordered(&self) -> bool {
        (*self).is_ordered()
    }
}

impl Expression for &dyn Expression {
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, buff: &mut String) {
        (*self).write_query(writer, context, buff);
    }
    fn is_ordered(&self) -> bool {
        (*self).is_ordered()
    }
}

impl Expression for () {
    fn write_query(&self, _writer: &dyn SqlWriter, _context: &mut Context, _buff: &mut String) {}
}

impl Expression for bool {
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, buff: &mut String) {
        writer.write_value_bool(context, buff, *self);
    }
}

impl<'a, T: Expression> From<&'a T> for &'a dyn Expression {
    fn from(value: &'a T) -> Self {
        value as &'a dyn Expression
    }
}

impl Expression for Value {
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, buff: &mut String) {
        writer.write_value(context, buff, self);
    }
}
