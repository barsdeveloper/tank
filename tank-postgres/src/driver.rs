use crate::{PostgresConnection, PostgresPrepared, PostgresSqlWriter, PostgresTransaction};
use tank_core::{Driver, DriverTransactional};

pub struct PostgresDriver {}

impl PostgresDriver {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Driver for PostgresDriver {
    type Connection = PostgresConnection;
    type SqlWriter = PostgresSqlWriter;
    type Prepared = PostgresPrepared;

    const NAME: &'static str = "postgres";

    fn sql_writer(&self) -> PostgresSqlWriter {
        PostgresSqlWriter {}
    }
}

impl DriverTransactional for PostgresDriver {
    type Transaction<'c> = PostgresTransaction<'c>;
}
