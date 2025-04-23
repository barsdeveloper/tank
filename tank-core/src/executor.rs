use crate::{Count, Driver, Query, QueryResult, Result, Row};
use futures::{future::BoxFuture, stream::BoxStream, FutureExt, StreamExt, TryStreamExt};
use std::fmt::Debug;

pub trait Executor: Send + Debug + Sized {
    type Driver: Driver;

    fn driver(&self) -> &Self::Driver;

    /// Execute the query and returns the results.
    fn run<'a>(
        &mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> BoxStream<'a, Result<QueryResult>>;

    /// Execute the query and returns the rows.
    fn fetch<'a>(
        &mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> BoxStream<'a, Result<Row>> {
        self.run(query)
            .filter_map(|v| async move {
                match v {
                    Ok(QueryResult::Row(v)) => Some(Ok(v)),
                    Err(e) => Some(Err(e)),
                    _ => None,
                }
            })
            .boxed()
    }

    /// Execute the query and return the total number of rows affected.
    fn execute<'a>(
        &mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> BoxFuture<'a, Result<Count>> {
        self.run(query)
            .filter_map(|v| async move {
                match v {
                    Ok(QueryResult::Count(v)) => Some(Ok(v)),
                    Err(e) => Some(Err(e)),
                    _ => None,
                }
            })
            .try_collect()
            .boxed()
    }
}
