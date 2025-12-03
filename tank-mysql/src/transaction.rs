use crate::{MySQLConnection, MySQLDriver, MySQLPrepared, RowWrap};
use async_stream::try_stream;
use mysql_async::{TxOpts, prelude::Queryable};
use std::sync::Arc;
use tank_core::{
    AsQuery, Error, Executor, Query, Result, Transaction,
    stream::{Stream, StreamExt, TryStreamExt},
};

pub struct MySQLTransaction<'c>(pub(crate) mysql_async::Transaction<'c>);

impl<'c> MySQLTransaction<'c> {
    pub async fn new(connection: &'c mut MySQLConnection) -> Result<Self> {
        Ok(Self(
            connection
                .conn
                .start_transaction(TxOpts::new())
                .await
                .map_err(|e| {
                    log::error!("{:#}", e);
                    e
                })?,
        ))
    }
}

impl<'c> Executor for MySQLTransaction<'c> {
    type Driver = MySQLDriver;

    fn driver(&self) -> &Self::Driver {
        &MySQLDriver {}
    }

    async fn prepare(&mut self, query: String) -> Result<Query<Self::Driver>> {
        Ok(MySQLPrepared::new(self.0.prep(query).await?).into())
    }

    fn run<'s>(
        &'s mut self,
        query: impl AsQuery<Self::Driver> + 's,
    ) -> impl Stream<Item = Result<tank_core::QueryResult>> + Send {
        let mut query = query.as_query();
        let context = Arc::new(format!("While running the query:\n{}", query.as_mut()));
        try_stream! {
            match query.as_mut() {
                Query::Raw(sql) => {
                    let sql = sql.as_str();
                    let mut result = self.0.query_iter(sql).await?;
                    let mut rows = 0;
                    while let Some(mut stream) = result.stream::<RowWrap>().await? {
                        while let Some(row) = stream.next().await.transpose()? {
                            rows += 1;
                            yield tank_core::QueryResult::Row(row.0)
                        }
                    }
                    let affected = result.affected_rows();
                    if rows == 0 && affected > 0 {
                        yield tank_core::QueryResult::Affected(tank_core::RowsAffected {
                            rows_affected: affected,
                            last_affected_id: result.last_insert_id().map(|v| v as _),
                        });
                    }
                }
                Query::Prepared(prepared) => {
                    let params = prepared.take_params()?;
                    let mut stream = self
                        .0
                        .exec_stream::<RowWrap, _, _>(&prepared.statement, params)
                        .await?;
                    while let Some(row) = stream.next().await.transpose()? {
                        yield row.0.into()
                    }
                }
            }
        }
        .map_err(move |e: Error| {
            let error = e.context(context.clone());
            log::error!("{:#}", error);
            error
        })
    }
}

impl<'c> Transaction<'c> for MySQLTransaction<'c> {
    async fn commit(self) -> Result<()> {
        self.0.commit().await.map(|_| ()).map_err(Into::into)
    }
    async fn rollback(self) -> Result<()> {
        self.0.rollback().await.map(|_| ()).map_err(Into::into)
    }
}
