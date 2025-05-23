use crate::{connection::DuckDBConnection, sql_writer::DuckDBSqlWriter, DuckDBPrepared};
use tank_core::Driver;

#[derive(Default, Clone, Copy)]
pub struct DuckDBDriver {}

impl DuckDBDriver {
    pub const fn new() -> Self {
        DuckDBDriver {}
    }
}

impl Driver for DuckDBDriver {
    type Connection = DuckDBConnection;
    type SqlWriter = DuckDBSqlWriter;
    type Prepared = DuckDBPrepared;

    fn get_instance() -> Self {
        static INSTANCE: DuckDBDriver = DuckDBDriver {};
        INSTANCE
    }

    fn sql_writer(&self) -> Self::SqlWriter {
        DuckDBSqlWriter::default()
    }
}
