use crate::{Executor, PreparedCache, Query, Result};
use futures::TryFutureExt;
use std::future::Future;

pub trait Connection: Executor {
    /// Initial part of the connect url
    const PREFIX: &'static str;

    /// Create a connection pool with at least one connection established to the given URL
    fn connect(url: &str) -> impl Future<Output = Result<impl Connection>>;
}

pub struct CachedConnection<E: Executor> {
    pub executor: E,
    pub prepared_cache: PreparedCache<E::Driver>,
}

impl<E: Executor> CachedConnection<E> {
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            prepared_cache: PreparedCache::new(),
        }
    }
}

impl<E: Executor> Executor for CachedConnection<E> {
    type Driver = E::Driver;

    fn driver(&self) -> &Self::Driver {
        self.executor.driver()
    }

    fn prepare(
        &mut self,
        query: String,
    ) -> impl Future<Output = Result<Query<<Self::Driver as crate::Driver>::Prepared>>> + Send {
        async {
            let mut query = Query::Raw(query.into());
            Ok(self
                .prepared_cache
                .as_prepared(&mut self.executor, &mut query)
                .await?
                .clone()
                .into())
        }
    }

    fn run(
        &mut self,
        query: Query<<Self::Driver as crate::Driver>::Prepared>,
    ) -> impl futures::Stream<Item = Result<crate::QueryResult>> + Send {
        let executor = &mut self.executor;
        let cache = &mut self.prepared_cache;
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
        let executor = &mut self.executor;
        let cache = &mut self.prepared_cache;
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
