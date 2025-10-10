use crate::{
    PostgresDriver, PostgresPrepared, PostgresTransaction, ValueHolder,
    util::stream_postgres_row_as_tank_row,
};
use std::{borrow::Cow, future, sync::Arc};
use tank_core::{
    Connection, Driver, Error, ErrorContext, Executor, Query, QueryResult, Result,
    future::Either,
    printable_query,
    stream::{self, Stream},
};
use tokio::spawn;
use tokio_postgres::NoTls;

pub struct PostgresConnection {
    pub(crate) client: tokio_postgres::Client,
    pub(crate) _transaction: bool,
}

impl Executor for PostgresConnection {
    type Driver = PostgresDriver;

    fn driver(&self) -> &Self::Driver {
        &PostgresDriver {}
    }

    async fn prepare(&mut self, sql: String) -> Result<Query<Self::Driver>> {
        Ok(
            PostgresPrepared::new(self.client.prepare(&sql).await.with_context(|| {
                format!(
                    "While preparing the query:\n{}",
                    printable_query!(sql.as_str())
                )
            })?)
            .into(),
        )
    }

    fn run(
        &mut self,
        query: Query<Self::Driver>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        let context = Arc::new(format!("While executing the query:\n{}", query));
        match query {
            Query::Raw(sql) => Either::Left(stream_postgres_row_as_tank_row(async move || {
                self.client
                    .query_raw(&sql, Vec::<ValueHolder>::new())
                    .await
                    .context(context)
            })),
            Query::Prepared(..) => Either::Right(stream::once(future::ready(Err(Error::msg(
                "Cannot run a prepares statement without a transaction",
            )
            .context(context))))),
        }
    }
}

impl Connection for PostgresConnection {
    #[allow(refining_impl_trait)]
    async fn connect(url: Cow<'static, str>) -> Result<PostgresConnection> {
        let prefix = format!("{}://", <Self::Driver as Driver>::NAME);
        if !url.starts_with(&prefix) {
            let error = Error::msg(format!(
                "Postgres connection url must start with `{}`",
                &prefix
            ));
            log::error!("{:#}", error);
            return Err(error);
        }
        let (client, connection) = tokio_postgres::connect(&url, NoTls)
            .await
            .with_context(|| format!("While trying to connect to `{}`", url))?;
        spawn(async move {
            if let Err(e) = connection.await {
                log::error!("Postgres connection error: {:#}", e);
            }
        });

        Ok(Self {
            client,
            _transaction: false,
        })
    }

    #[allow(refining_impl_trait)]
    fn begin(&mut self) -> impl Future<Output = Result<PostgresTransaction<'_>>> {
        PostgresTransaction::new(self)
    }
}
