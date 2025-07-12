use crate::{OpPrecedence, SqlWriter};
use std::fmt::Debug;

pub trait Expression: OpPrecedence + Send + Sync + Debug {
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String, qualify_columns: bool);
}

impl Expression for &dyn Expression {
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String, qualify_columns: bool) {
        (**self).write_query(writer, out, qualify_columns);
    }
}

impl Expression for () {
    fn write_query(&self, _writer: &dyn SqlWriter, _out: &mut String, _qualify_columns: bool) {}
}

impl Expression for bool {
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String, _qualify_columns: bool) {
        writer.write_value_bool(out, *self);
    }
}
