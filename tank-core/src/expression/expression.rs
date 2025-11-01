use crate::{
    OpPrecedence, Value,
    writer::{Context, SqlWriter},
};
use std::fmt::Debug;

/// A renderable SQL expression node.
pub trait Expression: OpPrecedence + Send + Sync + Debug {
    /// Serialize the expression into the output string using the dialect writer.
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, out: &mut String);
    /// Whether this expression carries ordering (ASC/DESC) information.
    fn is_ordered(&self) -> bool {
        false
    }
}

impl<T: Expression> Expression for &T {
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, out: &mut String) {
        (*self).write_query(writer, context, out);
    }
    fn is_ordered(&self) -> bool {
        (*self).is_ordered()
    }
}

impl Expression for &dyn Expression {
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, out: &mut String) {
        (*self).write_query(writer, context, out);
    }
    fn is_ordered(&self) -> bool {
        (*self).is_ordered()
    }
}

impl Expression for () {
    fn write_query(&self, _writer: &dyn SqlWriter, _context: &mut Context, _out: &mut String) {}
}

impl Expression for bool {
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, out: &mut String) {
        writer.write_value_bool(context, out, *self);
    }
}

impl<'a, T: Expression> From<&'a T> for &'a dyn Expression {
    fn from(value: &'a T) -> Self {
        value as &'a dyn Expression
    }
}

impl Expression for Value {
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, out: &mut String) {
        writer.write_value(context, out, self);
    }
}
