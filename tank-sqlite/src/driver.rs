use crate::{SqliteConnection, SqlitePrepared, SqliteTransaction, sql_writer::SqliteSqlWriter};
use tank_core::{Driver, DriverTransactional};

pub struct SqliteDriver {}

impl SqliteDriver {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Driver for SqliteDriver {
    type Connection = SqliteConnection;
    type SqlWriter = SqliteSqlWriter;
    type Prepared = SqlitePrepared;

    const NAME: &'static str = "sqlite";

    fn sql_writer(&self) -> SqliteSqlWriter {
        SqliteSqlWriter {}
    }
}

impl DriverTransactional for SqliteDriver {
    type Transaction<'c> = SqliteTransaction<'c>;
}
