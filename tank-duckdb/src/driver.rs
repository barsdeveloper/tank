use crate::{
    DuckDBPrepared, DuckDBTransaction, connection::DuckDBConnection, sql_writer::DuckDBSqlWriter,
};
use tank_core::{Driver, DriverTransactional};

#[derive(Default, Clone, Copy)]
pub struct DuckDBDriver {}

impl DuckDBDriver {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Driver for DuckDBDriver {
    type Connection = DuckDBConnection;
    type SqlWriter = DuckDBSqlWriter;
    type Prepared = DuckDBPrepared;

    const NAME: &'static str = "duckdb";

    fn sql_writer(&self) -> Self::SqlWriter {
        DuckDBSqlWriter::default()
    }
}

impl DriverTransactional for DuckDBDriver {
    type Transaction<'c> = DuckDBTransaction<'c>;
}
