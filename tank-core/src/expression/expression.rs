use crate::{OpPrecedence, SqlWriter, Value};

pub trait Expression: OpPrecedence + Send {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
        qualify_columns: bool,
    ) -> &'a mut String;
}

impl Expression for () {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        _writer: &W,
        out: &'a mut String,
        _qualify_columns: bool,
    ) -> &'a mut String {
        out
    }
}

impl Expression for Value {
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        _writer: &W,
        out: &'a mut String,
        _qualify_columns: bool,
    ) -> &'a mut String {
        out
    }
}
