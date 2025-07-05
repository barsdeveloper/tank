use crate::{Driver, Query, QueryResult, Result, RowLabeled, RowsAffected};
use futures::{Stream, StreamExt, TryStreamExt};
use std::future::Future;

pub trait Executor: Send + Sized {
    type Driver: Driver;

    fn driver(&self) -> &Self::Driver;

    fn prepare(
        &mut self,
        query: String,
    ) -> impl Future<Output = Result<Query<<Self::Driver as Driver>::Prepared>>> + Send;

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
    ) -> impl Future<Output = Result<RowsAffected>> + Send {
        self.run(query)
            .filter_map(|v| async move {
                match v {
                    Ok(QueryResult::Affected(v)) => Some(Ok(v)),
                    Err(e) => Some(Err(e)),
                    _ => None,
                }
            })
            .try_collect()
    }

    // fn append<'a, It: Iterator<Item = &'a Value>>(
    //     &mut self,
    //     rows: It,
    // ) -> impl Future<Output = Result<RowsAffected>> + Send {

    // }
}
