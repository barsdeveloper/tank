//! Execution abstraction wrapping a database `Driver`.
//! NOTE: All returned Futures and Streams MUST be awaited / fully consumed.
//! Some drivers may side-effect early, but for portability and correctness
//! always await the future or exhaust the stream.
//!
//! The `Executor` trait provides a uniform, async/stream-based interface for:
//! - Preparing parameterized queries (`prepare`)
//! - Running arbitrary queries yielding heterogeneous results (`run`)
//! - Convenience adapters to obtain only rows (`fetch`) or only affected counts (`execute`)
//! - Bulk data ingestion (`append`), with graceful fallback for drivers lacking append semantics.
//!
//! Streams:
//! `run` yields `QueryResult` items. Higher-level helpers (`fetch`, `execute`) filter & map only the
//! variants they care about, propagating errors while discarding unrelated items.
//!
//! Lifetimes:
//! `fetch` ties the stream lifetime `'s` to `&'s mut self`, ensuring the executor outlives row decoding.
//!
//! Fallbacks:
//! `append` always emits an INSERT when the driver has no dedicated append/merge support.
//!
//! Awaiting:
//! Every method returning a Future or Stream must be awaited / polled to completion. Some drivers
//! might perform effects before awaiting, but relying on that is undefined behavior across drivers.
#![doc = r#"
The `Executor` trait provides a uniform, async/stream-based interface for:
- Preparing parameterized queries (`prepare`)
- Running arbitrary queries yielding heterogeneous results (`run`)
- Convenience adapters to obtain only rows (`fetch`) or only affected counts (`execute`)
- Bulk data ingestion (`append`), with graceful fallback for drivers lacking append semantics.

Streams:
`run` yields `QueryResult` items. Higher-level helpers (`fetch`, `execute`) filter & map only the
variants they care about, propagating errors while discarding unrelated items.

Lifetimes:
`fetch` ties the stream lifetime `'s` to `&'s mut self`, ensuring the executor outlives row decoding.

Fallbacks:
`append` always emits an INSERT when the driver has no dedicated append/merge support.

Awaiting:
Every method returning a Future or Stream must be awaited / polled to completion. Some drivers
might perform effects before awaiting, but relying on that is undefined behavior across drivers.
"#]

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

    /// Must prepare the query in order to get typed columns
    fn types_need_prepare(&self) -> bool {
        false
    }

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
