use crate::{Connection, Prepared, Result, SqlWriter};
use std::future::Future;

pub trait Driver {
    type Connection: Connection;
    type SqlWriter: SqlWriter;
    type Prepared: Prepared;

    fn get_instance() -> Self;

    fn connect(&self, url: &str) -> impl Future<Output = Result<impl Connection>> {
        Self::Connection::connect(url)
    }

    fn sql_writer(&self) -> Self::SqlWriter;
}
