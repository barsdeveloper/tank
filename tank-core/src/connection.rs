use crate::{Executor, PreparedCache, Query, Result};
use futures::TryFutureExt;
use std::future::Future;

pub trait Connection: Executor {
    /// Create a connection pool with at least one connection established to the given URL
    fn connect(url: &str) -> impl Future<Output = Result<impl Connection>>;

    fn as_cached_connection(self) -> impl Connection {
        CachedConnection::<Self>::new(self)
    }
}

pub struct CachedConnection<C: Connection> {
    pub connection: C,
    pub cache: PreparedCache<C::Driver>,
}

impl<C: Connection> CachedConnection<C> {
    pub fn new(connection: C) -> Self {
        Self {
            connection,
            cache: PreparedCache::new(),
        }
    }
}

impl<C: Connection> Executor for CachedConnection<C> {
    type Driver = C::Driver;

    fn driver(&self) -> &Self::Driver {
        self.connection.driver()
    }

    fn prepare(
        &mut self,
        query: String,
    ) -> impl Future<Output = Result<Query<<Self::Driver as crate::Driver>::Prepared>>> + Send {
        async {
            let mut query = Query::Raw(query.into());
            Ok(self
                .cache
                .as_prepared(&mut self.connection, &mut query)
                .await?
                .clone()
                .into())
        }
    }

    fn run(
        &mut self,
        query: Query<<Self::Driver as crate::Driver>::Prepared>,
    ) -> impl futures::Stream<Item = Result<crate::QueryResult>> + Send {
        let executor = &mut self.connection;
        let cache = &mut self.cache;
        async move {
            // For run, only try to get the prepared statement, otherwise use the original query
            let query = if let Query::Raw(raw) = &query {
                cache.get(raw).await.unwrap_or(query)
            } else {
                query
            };
            Ok(executor.run(query))
        }
        .try_flatten_stream()
    }

    fn fetch(
        &mut self,
        mut query: Query<<Self::Driver as crate::Driver>::Prepared>,
    ) -> impl futures::Stream<Item = Result<crate::RowLabeled>> + Send {
        let executor = &mut self.connection;
        let cache = &mut self.cache;
        async move {
            cache.as_prepared(executor, &mut query).await?;
            Ok(executor.fetch(query))
        }
        .try_flatten_stream()
    }

    // fn execute(
    //     &mut self,
    //     query: Query<<Self::Driver as crate::Driver>::Prepared>,
    // ) -> impl Future<Output = Result<crate::RowsAffected>> + Send {
    //     self.run(query)
    //         .filter_map(|v| async move {
    //             match v {
    //                 Ok(crate::QueryResult::Affected(v)) => Some(Ok(v)),
    //                 Err(e) => Some(Err(e)),
    //                 _ => None,
    //             }
    //         })
    //         .try_collect()

    //}
}

impl<C: Connection> Connection for CachedConnection<C> {
    fn connect(url: &str) -> impl Future<Output = Result<impl Connection>> {
        C::connect(url)
    }

    fn as_cached_connection(self) -> impl Connection {
        self
    }
}
