use crate::{Executor, PreparedCache, Query, Result, stream::StreamExt};
use async_stream::try_stream;
use std::{future::Future, pin::pin};

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
        todo!()
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
        mut query: Query<<Self::Driver as crate::Driver>::Prepared>,
    ) -> impl futures::Stream<Item = Result<crate::QueryResult>> + Send {
        let cache = &mut self.prepared_cache;
        let executor = &mut self.executor;
        try_stream! {
            cache.as_prepared(executor, &mut query).await?;
            let mut stream = pin!(executor.run(query));
            while let Some(item) = stream.as_mut().next().await {
                yield item?;
            }
        }
    }
}
