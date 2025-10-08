use crate::{
    PostgresConnection, PostgresDriver, PostgresPrepared, ValueHolder, util::row_to_tank_row,
};
use async_stream::try_stream;
use std::pin::pin;
use tank_core::{
    Error, Executor, Query, QueryResult, Result, RowLabeled, Transaction,
    future::{Either, TryFutureExt},
    stream::{Stream, StreamExt},
};

pub struct PostgresTransaction<'c>(pub(crate) tokio_postgres::Transaction<'c>);

impl<'c> PostgresTransaction<'c> {
    pub async fn new(client: &'c mut PostgresConnection) -> Result<Self> {
        Ok(Self(client.client.transaction().await?))
    }
}

impl<'c> Executor for PostgresTransaction<'c> {
    type Driver = PostgresDriver;
    fn driver(&self) -> &Self::Driver {
        &PostgresDriver {}
    }
    async fn prepare(&mut self, query: String) -> Result<Query<Self::Driver>> {
        Ok(PostgresPrepared::new(self.0.prepare(&query).await?).into())
    }
    fn run(
        &mut self,
        query: Query<Self::Driver>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        try_stream! {
            let stream = match query {
                Query::Raw(sql) => {
                    Either::Left(self.0.query_raw(&sql, Vec::<ValueHolder>::new()).await?)
                }
                Query::Prepared(mut prepared) => {
                    let portal = if !prepared.is_complete() {
                        prepared.complete(self).await?
                    } else {
                        prepared.get_portal().ok_or(Error::msg(format!(
                            "The prepared statement `{}` is not complete",
                            prepared
                        )))?
                    };
                    Either::Right(self.0.query_portal_raw(&portal, 0).await?)
                }
            };
            let mut stream = pin!(stream);
            if let Some(first) = stream.next().await {
                let labels = first?
                    .columns()
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect::<tank_core::RowNames>();
                while let Some(value) = stream.next().await {
                    yield RowLabeled {
                        labels: labels.clone(),
                        values: row_to_tank_row(value?).into(),
                    }
                    .into()
                }
            }
        }
    }
}

impl<'c> Transaction<'c> for PostgresTransaction<'c> {
    fn commit(self) -> impl Future<Output = Result<()>> {
        self.0.commit().map_err(Into::into)
    }
    fn rollback(self) -> impl Future<Output = Result<()>> {
        self.0.rollback().map_err(Into::into)
    }
}
