use crate::{Count, Driver, Query, QueryResult, Result, RowLabeled};
use futures::{Stream, StreamExt, TryStreamExt};
use std::{fmt::Debug, future::Future};

pub trait Executor: Send + Debug + Sized {
    type Driver: Driver;

    fn driver(&self) -> &Self::Driver;

    fn prepare(
        &mut self,
        query: String,
    ) -> impl Future<Output = Result<<Self::Driver as Driver>::Prepared>> + Send;

    fn run(
        &mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send;

    /// Execute the query and returns the rows.
    fn fetch(
        &mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> impl Stream<Item = Result<RowLabeled>> + Send {
        self.run(query).filter_map(|v| async move {
            match v {
                Ok(QueryResult::RowLabeled(v)) => Some(Ok(v)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        })
    }

    /// Execute the query and return the total number of rows affected.
    fn execute(
        &mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> impl Future<Output = Result<Count>> + Send {
        self.run(query)
            .filter_map(|v| async move {
                match v {
                    Ok(QueryResult::Count(v)) => Some(Ok(v)),
                    Err(e) => Some(Err(e)),
                    _ => None,
                }
            })
            .try_collect()
    }
}
