use crate::{YourDBConnection, YourDBPrepared, YourDBSqlWriter};
use tank_core::Driver;

#[derive(Clone, Copy, Default)]
pub struct YourDBDriver;
impl YourDBDriver {
    pub const fn new() -> Self {
        Self
    }
}

impl Driver for YourDBDriver {
    type Connection = YourDBConnection;
    type SqlWriter = YourDBSqlWriter;
    type Prepared = YourDBPrepared;
    const NAME: &'static str = "yourdb";
    fn sql_writer(&self) -> Self::SqlWriter {
        YourDBSqlWriter::default()
    }
}

// If transactions are supported
// impl DriverTransactional for YourDBDriver {
//     type Transaction<'c> = YourDBTransaction<'c>;
// }
