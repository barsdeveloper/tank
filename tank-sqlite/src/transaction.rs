use crate::{SqliteConnection, SqliteDriver};
use tank_core::{
    Driver, Executor, Query, QueryResult, Result, RowLabeled, SqlWriter, Transaction,
    future::TryFutureExt, stream::Stream,
};

pub struct SqliteTransaction<'c> {
    connection: &'c mut SqliteConnection,
}

impl<'c> SqliteTransaction<'c> {
    pub async fn new(connection: &'c mut SqliteConnection) -> Result<Self> {
        let result = Self { connection };
        let mut sql = String::new();
        result
            .connection
            .driver()
            .sql_writer()
            .write_transaction_begin(&mut sql);
        result.connection.execute(sql.into()).await?;
        Ok(result)
    }
}

impl<'c> Executor for SqliteTransaction<'c> {
    type Driver = SqliteDriver;

    fn driver(&self) -> &SqliteDriver {
        self.connection.driver()
    }

    fn prepare(
        &mut self,
        query: String,
    ) -> impl Future<Output = Result<Query<SqliteDriver>>> + Send {
        self.connection.prepare(query)
    }

    fn run(
        &mut self,
        query: tank_core::Query<Self::Driver>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        self.connection.run(query)
    }

    fn fetch<'s>(
        &'s mut self,
        query: Query<SqliteDriver>,
    ) -> impl Stream<Item = Result<RowLabeled>> + Send + 's {
        self.connection.fetch(query)
    }

    fn execute(
        &mut self,
        query: tank_core::Query<Self::Driver>,
    ) -> impl Future<Output = tank_core::Result<tank_core::RowsAffected>> + Send {
        self.connection.execute(query)
    }

    fn append<'a, E, It>(
        &mut self,
        entities: It,
    ) -> impl Future<Output = tank_core::Result<tank_core::RowsAffected>> + Send
    where
        E: tank_core::Entity + 'a,
        It: IntoIterator<Item = &'a E> + Send,
    {
        self.connection.append(entities)
    }
}

impl<'c> Transaction<'c> for SqliteTransaction<'c> {
    fn commit(self) -> impl Future<Output = Result<()>> {
        let mut sql = String::new();
        self.driver()
            .sql_writer()
            .write_transaction_commit(&mut sql);
        self.connection.execute(sql.into()).map_ok(|_| ())
    }

    fn rollback(self) -> impl Future<Output = Result<()>> {
        let mut sql = String::new();
        self.driver()
            .sql_writer()
            .write_transaction_rollback(&mut sql);
        self.connection.execute(sql.into()).map_ok(|_| ())
    }
}
