use crate::{Executor, Result};

/// A mutable transactional context implementing [`Executor`].
///
/// Consuming methods (`commit`, `rollback`) finalize the transaction. Dropping
/// without explicit finalization should implicitly rollback, prefer an explicit
/// choice for clarity.
pub trait Transaction<'c>: Executor {
    /// Commit the outstanding changes.
    fn commit(self) -> impl Future<Output = Result<()>>;
    /// Rollback any uncommitted changes.
    fn rollback(self) -> impl Future<Output = Result<()>>;
}
