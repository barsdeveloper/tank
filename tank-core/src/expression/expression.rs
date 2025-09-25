use crate::{
    OpPrecedence,
    writer::{Context, SqlWriter},
};
use std::fmt::{Debug, Write};

pub trait Expression: OpPrecedence + Send + Sync + Debug {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut dyn Write);
    fn is_ordered(&self) -> bool {
        false
    }
}

impl<T: Expression> Expression for &T {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut dyn Write) {
        (*self).write_query(writer, context, buff);
    }
    fn is_ordered(&self) -> bool {
        (*self).is_ordered()
    }
}

impl Expression for &dyn Expression {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut dyn Write) {
        (*self).write_query(writer, context, buff);
    }
    fn is_ordered(&self) -> bool {
        (*self).is_ordered()
    }
}

impl Expression for () {
    fn write_query(&self, _writer: &dyn SqlWriter, _context: Context, _out: &mut dyn Write) {}
}

impl Expression for bool {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut dyn Write) {
        writer.write_value_bool(context, buff, *self);
    }
}

impl<'a, T: Expression> From<&'a T> for &'a dyn Expression {
    fn from(value: &'a T) -> Self {
        value as &'a dyn Expression
    }
}
