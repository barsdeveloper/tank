use crate::{Connection, Prepared, Result, Transaction, writer::SqlWriter};
use std::{borrow::Cow, future::Future};

/// A backend implementation providing connection + SQL dialect services.
///
/// The `Driver` forms the bridge between high-level Tank abstractions and
/// engine-specific details (quoting rules, type names, upsert syntax, etc.).
///
/// # Associated Types
/// * `Connection` – concrete type implementing [`Connection`].
/// * `SqlWriter` – dialect printer translating Tank's semantic AST into SQL.
/// * `Prepared` – owned prepared statement handle used by `Query::Prepared`.
///
/// # Notes
/// * `connect` delegates to the associated `Connection::connect` – drivers may
///   wrap pooling or additional validation around it.
/// * `NAME` is a human readable identifier (e.g. "postgres", "sqlite").
pub trait Driver {
    /// Concrete connection type.
    type Connection: Connection;
    /// Dialect aware SQL writer.
    type SqlWriter: SqlWriter;
    /// Prepared statement wrapper binding values.
    type Prepared: Prepared;

    /// Human-readable backend name.
    const NAME: &'static str;

    /// Establish a connection given a URL.
    fn connect(&self, url: Cow<'static, str>) -> impl Future<Output = Result<impl Connection>> {
        Self::Connection::connect(url)
    }

    /// Obtain a SQL writer object (cheap to construct).
    fn sql_writer(&self) -> Self::SqlWriter;
}

/// Extension trait for drivers supporting transactions.
///
/// Separates transactional capabilities so drivers can avoid the complexity
/// when transactions are not supported.
pub trait DriverTransactional: Driver {
    /// Concrete transaction type, parameterized by connection borrow lifetime.
    type Transaction<'c>: Transaction<'c>;
}
