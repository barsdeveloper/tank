use crate::{OpPrecedence, SqlWriter};

pub trait Expression: OpPrecedence + Send + Sync {
    fn sql_write<'a>(
        &self,
        writer: &dyn SqlWriter,
        out: &'a mut String,
        qualify_columns: bool,
    ) -> &'a mut String;
}

impl Expression for &dyn Expression {
    fn sql_write<'a>(
        &self,
        writer: &dyn SqlWriter,
        out: &'a mut String,
        qualify_columns: bool,
    ) -> &'a mut String {
        (**self).sql_write(writer, out, qualify_columns)
    }
}

impl Expression for () {
    fn sql_write<'a>(
        &self,
        _writer: &dyn SqlWriter,
        out: &'a mut String,
        _qualify_columns: bool,
    ) -> &'a mut String {
        out
    }
}
