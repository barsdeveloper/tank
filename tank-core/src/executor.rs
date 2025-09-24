use futures::FutureExt;

use crate::{
    Driver, Entity, Query, QueryResult, Result, RowLabeled, RowsAffected,
    stream::{Stream, StreamExt, TryStreamExt},
    writer::SqlWriter,
};
use std::future::Future;

pub trait Executor: Send + Sized {
    type Driver: Driver;

    fn driver(&self) -> &Self::Driver;

    fn prepare(
        &mut self,
        query: String,
    ) -> impl Future<Output = Result<Query<<Self::Driver as Driver>::Prepared>>> + Send;

    /// General method to send any query and return any result type (either row or count)
    fn run(
        &mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send;

    /// Execute the query and returns the rows.
    fn fetch<'s>(
        &'s mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> impl Stream<Item = Result<RowLabeled>> + Send + 's {
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

    /// Append rows to a table. Defaults to insert query for drivers that do not support this feature.
    fn append<'a, E, It>(&mut self, rows: It) -> impl Future<Output = Result<()>>
    where
        E: Entity + 'a,
        It: IntoIterator<Item = &'a E>,
    {
        let mut query = String::new();
        self.driver()
            .sql_writer()
            .write_insert(&mut query, rows, false);
        self.execute(query.into()).map(|_| Ok(()))
    }
}
