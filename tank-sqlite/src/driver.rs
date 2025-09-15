use tank_core::Driver;

use crate::{SqliteConnection, SqlitePrepared, sql_writer::SqliteSqlWriter};

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
