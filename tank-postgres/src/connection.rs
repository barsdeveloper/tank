use async_stream::try_stream;
use std::{borrow::Cow, future, pin::pin, sync::Arc};
use tank_core::{
    Connection, Driver, Error, ErrorContext, Executor, Query, QueryResult, Result, RowLabeled,
    future::Either,
    printable_query,
    stream::{self, Stream, StreamExt},
};
use tokio_postgres::{NoTls, Socket, tls::NoTlsStream};

use crate::{
    PostgresDriver, PostgresPrepared, PostgresTransaction, ValueHolder, util::row_to_tank_row,
};

pub struct PostgresConnection {
    pub(crate) connection: tokio_postgres::Connection<Socket, NoTlsStream>,
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
            Query::Raw(sql) => Either::Left(try_stream! {
                let stream = self
                    .client
                    .query_raw(&sql, Vec::<ValueHolder>::new())
                    .await
                    .context(context.clone())?;
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
            }),
            Query::Prepared(..) => Either::Right(stream::once(future::ready(Err(Error::msg(
                "Cannot run a prepares statement without a transaction",
            )
            .context(context.clone()))))),
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
        let url = url.trim_start_matches(&prefix);
        let (client, connection) = tokio_postgres::connect(url, NoTls)
            .await
            .with_context(|| format!("While trying to connect to {}", url))?;

        Ok(Self {
            connection,
            client,
            _transaction: false,
        })
    }

    #[allow(refining_impl_trait)]
    fn begin(&mut self) -> impl Future<Output = Result<PostgresTransaction>> {
        PostgresTransaction::new(self)
    }
}
