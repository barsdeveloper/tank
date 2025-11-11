use crate::{MySQLDriver, MySQLPrepared, MySQLTransaction, RowWrap};
use async_stream::try_stream;
use mysql_async::{Conn, Opts, prelude::Queryable};
use std::{borrow::Cow, sync::Arc};
use tank_core::{
    Connection, Driver, Error, ErrorContext, Executor, Query, QueryResult, Result,
    stream::{Stream, StreamExt, TryStreamExt},
    truncate_long,
};
use url::Url;

pub struct MySQLConnection {
    pub(crate) connection: Conn,
}

impl Executor for MySQLConnection {
    type Driver = MySQLDriver;

    fn driver(&self) -> &Self::Driver {
        &MySQLDriver {}
    }

    async fn prepare(&mut self, query: String) -> Result<Query<Self::Driver>> {
        Ok(Query::Prepared(MySQLPrepared::new(
            self.connection.prep(query).await?,
        )))
    }

    fn run(
        &mut self,
        query: Query<Self::Driver>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        let context = Arc::new(format!("While running the query:\n{}", query));
        try_stream! {
            match query {
                Query::Raw(sql) => {
                    let mut result = self.connection.query_iter(sql).await?;
                    while let Some(mut stream) = result.stream::<RowWrap>().await? {
                        while let Some(row) = stream.next().await.transpose()? {
                            yield row.0.into()
                        }
                    }
                }
                Query::Prepared(mut prepared) => {
                    let params = prepared.take_params()?;
                    let mut stream = self
                        .connection
                        .exec_stream::<RowWrap, _, _>(prepared.statement, params)
                        .await?;
                    while let Some(row) = stream.next().await.transpose()? {
                        yield row.0.into()
                    }
                }
            }
        }
        .map_err(move |e: Error| {
            let e = e.context(context.clone());
            log::error!("{:#}", e);
            e
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
        Ok(MySQLConnection { connection })
    }

    #[allow(refining_impl_trait)]
    async fn begin(&mut self) -> Result<MySQLTransaction<'_>> {
        Err(Error::msg("Transactions are not supported by MySQL"))
    }
}
