use crate::{MySQLDriver, MySQLPrepared, MySQLTransaction, RowWrap};
use async_stream::try_stream;
use mysql_async::{Conn, Opts, TxOpts, prelude::Queryable};
use std::{borrow::Cow, sync::Arc};
use tank_core::{
    AsQuery, Connection, Driver, Error, ErrorContext, Executor, Query, Result,
    stream::{Stream, StreamExt, TryStreamExt},
    truncate_long,
};
use url::Url;

pub struct MySQLConnection {
    pub(crate) conn: Conn,
}

impl Executor for MySQLConnection {
    type Driver = MySQLDriver;

    fn driver(&self) -> &Self::Driver {
        &MySQLDriver {}
    }

    async fn prepare(&mut self, query: String) -> Result<Query<Self::Driver>> {
        Ok(MySQLPrepared::new(self.conn.prep(query).await?).into())
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
                    let mut result = self.conn.query_iter(sql).await?;
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
                        .conn
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

impl Connection for MySQLConnection {
    async fn connect(url: Cow<'static, str>) -> Result<MySQLConnection> {
        let context = || format!("While trying to connect to `{}`", truncate_long!(url));
        let prefix = format!("{}://", <Self::Driver as Driver>::NAME);
        if !url.starts_with(&prefix) {
            let error = Error::msg(format!(
                "MySQL connection url must start with `{}`",
                &prefix
            ))
            .context(context());
            log::error!("{:#}", error);
            return Err(error);
        }
        let url = Url::parse(&url).with_context(context)?;
        let config = Opts::from_url(url.as_str()).with_context(context)?;
        let connection = Conn::new(config).await.with_context(context)?;
        Ok(MySQLConnection { conn: connection })
    }

    #[allow(refining_impl_trait)]
    fn begin(&mut self) -> impl Future<Output = Result<MySQLTransaction<'_>>> {
        MySQLTransaction::new(self)
    }
}
