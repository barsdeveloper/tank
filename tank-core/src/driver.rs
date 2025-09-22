use crate::{Connection, Prepared, Result, writer::SqlWriter};
use std::{borrow::Cow, future::Future};

pub trait Driver {
    type Connection: Connection;
    type SqlWriter: SqlWriter;
    type Prepared: Prepared;

    const NAME: &'static str;

    fn connect(&self, url: Cow<'static, str>) -> impl Future<Output = Result<impl Connection>> {
        Self::Connection::connect(url)
    }

    fn sql_writer(&self) -> Self::SqlWriter;
}
