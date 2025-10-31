use crate::{SQLiteConnection, SQLitePrepared, SQLiteTransaction, sql_writer::SQLiteSqlWriter};
use tank_core::{Driver, DriverTransactional};

pub struct SQLiteDriver {}

impl SQLiteDriver {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Driver for SQLiteDriver {
    type Connection = SQLiteConnection;
    type SqlWriter = SQLiteSqlWriter;
    type Prepared = SQLitePrepared;

    const NAME: &'static str = "sqlite";

    fn sql_writer(&self) -> SQLiteSqlWriter {
        SQLiteSqlWriter {}
    }
}

impl DriverTransactional for SQLiteDriver {
    type Transaction<'c> = SQLiteTransaction<'c>;
}
