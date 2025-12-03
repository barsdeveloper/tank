use crate::{
    AsQuery, Driver, Entity, Query, QueryResult, Result, RowLabeled, RowsAffected,
    stream::{Stream, StreamExt, TryStreamExt},
    writer::SqlWriter,
};
use std::future::Future;

/// Async query executor bound to a concrete `Driver`.
///
/// Responsibilities:
/// - Translate high-level operations into driver queries
/// - Stream results without buffering the entire result set (if possible)
/// - Provide ergonomic helpers for common patterns
///
/// Implementors typically wrap a connection or pooled handle.
pub trait Executor: Send + Sized {
    /// Underlying driver type supplying SQL dialect + I/O.
    type Driver: Driver;

    /// Access the driver instance.
    fn driver(&self) -> &Self::Driver;

    /// Prepare a query (e.g. statement caching / parameter binding) returning a `Query`.
    ///
    /// Await/Consume:
    /// - Must be awaited; preparation may allocate resources on the driver.
    ///
    /// Errors:
    /// - Driver-specific preparation failures.
    fn prepare(
        &mut self,
        query: String,
    ) -> impl Future<Output = Result<Query<Self::Driver>>> + Send;

    /// Run an already prepared query, streaming heterogeneous `QueryResult` items.
    ///
    /// Await/Consume:
    /// - You must drive the returned stream to completion (or until you intentionally stop).
    ///
    /// Stream Items:
    /// - `QueryResult::Row` for each produced row.
    /// - `QueryResult::Affected` for write operations (may appear before/after rows depending on driver).
    ///
    /// Errors:
    /// - Emitted inline in the stream; consumers should use `TryStreamExt`.
    fn run<'s>(
        &'s mut self,
        query: impl AsQuery<Self::Driver> + 's,
    ) -> impl Stream<Item = Result<QueryResult>> + Send;

    /// Run a query and stream only labeled rows, filtering out non-row results.
    ///
    /// Await/Consume:
    /// - Consume the stream fully if you expect to release underlying resources cleanly.
    ///
    /// Each error from `run` is forwarded; affected-count results are discarded.
    fn fetch<'s>(
        &'s mut self,
        query: impl AsQuery<Self::Driver> + 's,
    ) -> impl Stream<Item = Result<RowLabeled>> + Send + 's {
        self.run(query).filter_map(|v| async move {
            match v {
                Ok(QueryResult::Row(v)) => Some(Ok(v)),
                Err(e) => Some(Err(e)),
                _ => None,
            }
        })
    }

    /// Execute a query and return a single aggregated `RowsAffected`.
    ///
    /// Await/Consume:
    /// - Must be awaited; no side-effects are guaranteed until completion.
    ///
    /// If a driver returns multiple `QueryResult::Affected` values, they are combined via `FromIterator`
    /// (driver/module must provide the appropriate implementation).
    fn execute<'s>(
        &'s mut self,
        query: impl AsQuery<Self::Driver> + 's,
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

    /// Append entities to a table.
    ///
    /// Await/Consume:
    /// - Must be awaited; insertion may be deferred until polled.
    ///
    /// Semantics:
    /// - Uses driver append/ingest feature when supported.
    /// - Falls back to plain INSERT statements via `sql_writer().write_insert(..., false)` otherwise.
    ///
    /// Returns:
    /// - Total number of inserted rows.
    fn append<'a, E, It>(
        &mut self,
        entities: It,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        E: Entity + 'a,
        It: IntoIterator<Item = &'a E> + Send,
    {
        let mut query = String::new();
        self.driver()
            .sql_writer()
            .write_insert(&mut query, entities, false);
        self.execute(query)
    }
}
