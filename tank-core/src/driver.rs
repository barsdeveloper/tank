use crate::{Connection, Prepared, SqlWriter};

pub trait Driver {
    type Connection: Connection;
    type SqlWriter: SqlWriter;
    type Prepared: Prepared;

    fn get_instance() -> Self;
    fn sql_writer(&self) -> Self::SqlWriter;
}
