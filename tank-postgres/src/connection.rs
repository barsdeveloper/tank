use crate::{
    PostgresDriver, PostgresPrepared, PostgresTransaction, ValueWrap,
    util::{
        stream_postgres_row_to_tank_row, stream_postgres_simple_query_message_to_tank_query_result,
    },
};
use async_stream::try_stream;
use openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use std::{borrow::Cow, env, path::Path, pin::pin, sync::Arc};
use tank_core::{
    Connection, Driver, Error, ErrorContext, Executor, Query, QueryResult, Result, Transaction,
    future::Either,
    stream::{Stream, StreamExt, TryStreamExt},
    truncate_long,
};
use tokio::spawn;
use tokio_postgres::NoTls;
use url::Url;
use urlencoding::decode;

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
        let sql = sql.trim_end().trim_end_matches(';');
        Ok(
            PostgresPrepared::new(self.client.prepare(&sql).await.map_err(|e| {
                let e = Error::new(e).context(format!(
                    "While preparing the query:\n{}",
                    truncate_long!(sql)
                ));
                log::error!("{:#}", e);
                e
            })?)
            .into(),
        )
    }

    fn run(
        &mut self,
        query: Query<Self::Driver>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        let context = Arc::new(format!("While running the query:\n{}", query));
        match query {
            Query::Raw(sql) => {
                Either::Left(stream_postgres_simple_query_message_to_tank_query_result(
                    async move || self.client.simple_query_raw(&sql).await.map_err(Into::into),
                ))
            }
            Query::Prepared(..) => Either::Right(try_stream! {
                let mut transaction = self.begin().await?;
                {
                    let mut stream = pin!(transaction.run(query));
                    while let Some(value) = stream.next().await.transpose()? {
                        yield value;
                    }
                }
                transaction.commit().await?;
            }),
        }
        .map_err(move |e: Error| {
            let e = e.context(context.clone());
            log::error!("{:#}", e);
            e
        })
    }

    fn fetch<'s>(
        &'s mut self,
        query: Query<Self::Driver>,
    ) -> impl Stream<Item = Result<tank_core::RowLabeled>> + Send + 's {
        let context = Arc::new(format!("While fetching the query:\n{}", query));
        match query {
            Query::Raw(sql) => Either::Left(stream_postgres_row_to_tank_row(async move || {
                self.client
                    .query_raw(&sql, Vec::<ValueWrap>::new())
                    .await
                    .map_err(|e| {
                        let e = Error::new(e).context(context.clone());
                        log::error!("{:#}", e);
                        e
                    })
            })),
            Query::Prepared(..) => Either::Right(
                try_stream! {
                    let mut transaction = self.begin().await?;
                    {
                        let mut stream = pin!(transaction.fetch(query));
                        while let Some(value) = stream.next().await.transpose()? {
                            yield value;
                        }
                    }
                    transaction.commit().await?;
                }
                .map_err(move |e: Error| {
                    let e = e.context(context.clone());
                    log::error!("{:#}", e);
                    e
                }),
            ),
        }
    }
}

impl Connection for PostgresConnection {
    #[allow(refining_impl_trait)]
    async fn connect(url: Cow<'static, str>) -> Result<PostgresConnection> {
        let context = || format!("While trying to connect to `{}`", url);
        let url = decode(&url).with_context(context)?;
        let prefix = format!("{}://", <Self::Driver as Driver>::NAME);
        if !url.starts_with(&prefix) {
            let error = Error::msg(format!(
                "Postgres connection url must start with `{}`",
                &prefix
            ))
            .context(context());
            log::error!("{:#}", error);
            return Err(error);
        }
        let mut url = Url::parse(&url).with_context(context)?;
        let mut take_url_param = |key: &str, env_var: &str| {
            let mut value = None;
            let mut pairs: Vec<(String, String)> = url
                .query_pairs()
                .map(|(k, v)| (k.into(), v.into()))
                .collect();
            if let Some(pos) = pairs.iter().position(|(k, _)| k == key) {
                let (_, v) = pairs.remove(pos);
                value = Some(v);
            }
            url.query_pairs_mut()
                .clear()
                .extend_pairs(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())));

            value.or_else(|| env::var(env_var).ok())
        };
        let sslmode = take_url_param("sslmode", "PGSSLMODE").unwrap_or("disable".into());
        let client = if sslmode == "disable" {
            let (client, connection) = tokio_postgres::connect(url.as_str(), NoTls).await?;
            spawn(async move {
                if let Err(e) = connection.await
                    && !e.is_closed()
                {
                    log::error!("Postgres connection error: {:#}", e);
                }
            });
            client
        } else {
            let mut builder = SslConnector::builder(SslMethod::tls())?;
            if let Some(path) = take_url_param("sslrootcert", "PGSSLROOTCERT")
                .as_deref()
                .map(Path::new)
                && path.exists()
            {
                builder.set_ca_file(path)?;
            }
            if let Some(path) = take_url_param("sslcert", "PGSSLCERT")
                .as_deref()
                .map(Path::new)
                && path.exists()
            {
                builder.set_certificate_chain_file(path)?;
            }
            if let Some(path) = take_url_param("sslkey", "PGSSLKEY")
                .as_deref()
                .map(Path::new)
                && path.exists()
            {
                builder.set_private_key_file(path, SslFiletype::PEM)?;
            }
            match &*sslmode {
                "require" => {
                    builder.set_verify(SslVerifyMode::NONE);
                }
                "verify-ca" => {
                    builder.set_verify(SslVerifyMode::PEER);
                }
                "verify-full" => {
                    builder.set_verify(SslVerifyMode::PEER);
                }
                _ => {
                    builder.set_verify(SslVerifyMode::PEER);
                }
            }
            let connector = MakeTlsConnector::new(builder.build());
            let (client, connection) = tokio_postgres::connect(url.as_str(), connector).await?;
            spawn(async move {
                if let Err(e) = connection.await
                    && !e.is_closed()
                {
                    log::error!("Postgres connection error: {:#}", e);
                }
            });
            client
        };
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
