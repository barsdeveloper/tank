use crate::{MySQLConnection, MySQLDriver};
use tank_core::{Error, Result, Transaction, impl_executor_transaction};

pub struct MySQLTransaction<'c> {
    connection: &'c mut MySQLConnection,
}

impl_executor_transaction!(MySQLDriver, MySQLTransaction, connection);

impl<'c> Transaction<'c> for MySQLTransaction<'c> {
    async fn commit(self) -> Result<()> {
        Err(Error::msg("Transactions are not supported by MySQL"))
    }

    async fn rollback(self) -> Result<()> {
        Err(Error::msg("Transactions are not supported by MySQL"))
    }
}
