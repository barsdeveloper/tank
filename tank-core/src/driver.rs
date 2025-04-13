use crate::{Connection, SqlWriter};

pub trait Driver {
    type Connection: Connection;
    type SqlWriter: SqlWriter;

    fn get_instance() -> Self;
    fn sql_writer(&self) -> Self::SqlWriter;
}
