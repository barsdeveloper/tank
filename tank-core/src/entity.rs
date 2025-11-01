use crate::{
    ColumnDef, Context, DataSet, Driver, Error, Executor, Expression, Result, Row, RowLabeled,
    RowsAffected, TableRef, Value, future::Either, stream::Stream, writer::SqlWriter,
};
use futures::{FutureExt, StreamExt, TryFutureExt};
use log::Level;
use std::{
    future::{self, Future},
    pin::pin,
};

/// Represents a database-backed record with schema and persistence behavior.
///
/// An `Entity` defines:
/// - Static table/column metadata
/// - Conversion to/from raw `Row` / `RowLabeled`
/// - Helper CRUD operations using an `Executor`
///
/// Lifetimes:
/// - Associated `PrimaryKey<'a>` may borrow from `&self` when composed.
///
/// Error Handling:
/// - Methods return `Result<...>` using crate-level `Error`.
/// - `save` / `delete` early-return an error when a primary key is not defined.
///
/// Streaming:
/// - `find_many` returns a `Stream` of row conversions.
/// - `find_one` and `find_pk` internally consume a single element from that stream.
///
/// Idempotency:
/// - `save` performs an UPSERT-style insert when supported (based on
///   `sql_writer().write_insert(..., true)`), falling back to insert-only on
///   unsupported drivers.
pub trait Entity {
    /// Associated primary key type. May be a single value or a tuple.
    type PrimaryKey<'a>;

    /// Returns the table reference backing this entity.
    fn table() -> &'static TableRef;

    /// Returns all declared column definitions in declaration order.
    fn columns() -> &'static [ColumnDef];

    /// Iterator over columns forming the primary key. Empty iterator means no PK.
    fn primary_key_def() -> impl ExactSizeIterator<Item = &'static ColumnDef>;

    /// Extracts the primary key value(s) from `self`.
    ///
    /// Should mirror the order and shape returned by `primary_key_def()`.
    fn primary_key(&self) -> Self::PrimaryKey<'_>;

    /// Returns an iterator over unique constraint definitions.
    ///
    /// Each inner iterator represents one unique composite constraint.
    fn unique_defs()
        -> impl ExactSizeIterator<Item = impl ExactSizeIterator<Item = &'static ColumnDef>>;

    /// Returns a filtered mapping of column name to value, typically excluding
    /// auto-generated or default-only columns.
    fn row_filtered(&self) -> Box<[(&'static str, Value)]>;

    /// Returns a full `Row` representation including all persisted columns.
    fn row_full(&self) -> Row;

    /// Constructs `Self` from a labeled database row.
    ///
    /// Errors if mandatory columns are missing or type conversion fails.
    fn from_row(row: RowLabeled) -> Result<Self>
    where
        Self: Sized;

    /// Creates the underlying table (and optionally schema) if requested.
    ///
    /// Parameters:
    /// - `if_not_exists`: guards against existing table (if drivers support it, otherwise just create table).
    /// - `create_schema`: attempt to create schema prior to table creation (if drivers support it).
    fn create_table<Exec: Executor>(
        executor: &mut Exec,
        if_not_exists: bool,
        create_schema: bool,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Drops the underlying table (and optionally schema) if requested.
    ///
    /// Parameters:
    /// - `if_exists`: guards against missing table (if drivers support it, otherwise just drop table).
    /// - `drop_schema`: attempt to drop schema after table removal (if drivers support it).
    fn drop_table<Exec: Executor>(
        executor: &mut Exec,
        if_exists: bool,
        drop_schema: bool,
    ) -> impl Future<Output = Result<()>> + Send;

    /// Inserts a single entity row.
    ///
    /// Returns rows affected (expected: 1 on success).
    fn insert_one<Exec: Executor, E: Entity>(
        executor: &mut Exec,
        entity: &E,
    ) -> impl Future<Output = Result<RowsAffected>> + Send;

    /// Multiple insert for a homogeneous iterator of entities.
    ///
    /// Returns the number of rows inserted.
    fn insert_many<'a, Exec, It>(
        executor: &mut Exec,
        items: It,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        Self: 'a,
        Exec: Executor,
        It: IntoIterator<Item = &'a Self> + Send;

    /// Finds an entity by primary key.
    ///
    /// Returns `Ok(None)` if no row matches.
    fn find_pk<Exec: Executor>(
        executor: &mut Exec,
        primary_key: &Self::PrimaryKey<'_>,
    ) -> impl Future<Output = Result<Option<Self>>> + Send
    where
        Self: Sized;

    /// Finds the first entity matching a condition expression.
    ///
    /// Returns `Ok(None)` if no row matches.
    fn find_one<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
    ) -> impl Future<Output = Result<Option<Self>>> + Send
    where
        Self: Sized,
    {
        let stream = Self::find_many(executor, condition, Some(1));
        async move { pin!(stream).into_future().map(|(v, _)| v).await.transpose() }
    }

    /// Streams entities matching a condition.
    ///
    /// `limit` restricts the maximum number of rows returned at a database level if `Some` (if supported, otherwise unlimited).
    fn find_many<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
        limit: Option<u32>,
    ) -> impl Stream<Item = Result<Self>> + Send
    where
        Self: Sized;

    /// Deletes exactly one entity by primary key.
    ///
    /// Returns rows affected (0 if not found).
    fn delete_one<Exec: Executor>(
        executor: &mut Exec,
        primary_key: Self::PrimaryKey<'_>,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        Self: Sized;

    /// Deletes all entities matching a condition.
    ///
    /// Returns the number of rows deleted.
    fn delete_many<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        Self: Sized;

    /// Saves the entity (insert or update) based on primary key presence.
    ///
    /// Behavior:
    /// - Errors if no primary key is defined in the table.
    /// - Uses driver-specific UPSERT semantics when available (`write_insert(..., true)`).
    ///
    /// Errors:
    /// - Missing PK in the table.
    /// - Execution failures from underlying driver.
    fn save<Exec: Executor>(&self, executor: &mut Exec) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        if Self::primary_key_def().len() == 0 {
            let error = Error::msg(
                "Cannot save a entity without a primary key, it would always result in a insert",
            );
            log::error!("{}", error);
            return Either::Left(future::ready(Err(error)));
        }
        let mut query = String::with_capacity(512);
        executor
            .driver()
            .sql_writer()
            .write_insert(&mut query, [self], true);
        Either::Right(executor.execute(query.into()).map_ok(|_| ()))
    }

    /// Deletes this entity instance via its primary key.
    ///
    /// Errors:
    /// - Missing PK in the table.
    /// - If not exactly one row was deleted.
    fn delete<Exec: Executor>(&self, executor: &mut Exec) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        if Self::primary_key_def().len() == 0 {
            let error =
                Error::msg("Cannot delete a entity without a primary key, it would delete nothing");
            log::error!("{}", error);
            return Either::Left(future::ready(Err(error)));
        }
        Either::Right(Self::delete_one(executor, self.primary_key()).map(|v| {
            v.and_then(|v| {
                if v.rows_affected == 1 {
                    Ok(())
                } else {
                    let error = Error::msg(format!(
                        "The query deleted {} rows instead of the expected 1",
                        v.rows_affected
                    ));
                    log::log!(
                        if v.rows_affected == 0 {
                            Level::Info
                        } else {
                            Level::Error
                        },
                        "{}",
                        error
                    );
                    Err(error)
                }
            })
        }))
    }
}

impl<E: Entity> DataSet for E {
    /// Indicates whether column names should be fully qualified with schema and table name.
    ///
    /// For entities this returns `false` to keep queries concise.
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        false
    }

    /// Writes the table reference into the query string.
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, out: &mut String) {
        Self::table().write_query(writer, context, out);
    }
}
