# Connection
###### *Field Manual Section 3* - Supply Lines

Welcome to the armored convoy, commander. Before you can unleash Tank's firepower, you have to secure your supply lines. Open a **Connection** to your database, and when the mission escalates, lock operations inside a **Transaction**. No connection, no combat. It's that simple.

## Connect
Every database connection abstraction implements the [`Connection`](https://docs.rs/tank/latest/tank/trait.Connection.html) trait. This is your communication link to the database server. Call [`connect("dbms://...")`](https://docs.rs/tank/latest/tank/trait.Connection.html#tymethod.connect) with a URL let Tank establish the line. Every driver is its own crate. Load only what you need for the operation. Check the [drivers](1-introduction.md#drivers) to see the available connections.

Once the line is open, the connection exposes both the [`Connection`](https://docs.rs/tank/latest/tank/trait.Connection.html) and [`Executor`](https://docs.rs/tank/latest/tank/trait.Executor.html) interfaces, enabling you to prepare statements, run multiple queries, execute commands, fetch rows and orchestrate transactions.

#### DuckDB
DuckDB is your embedded artillery piece: fast, local, and always ready. Perfect for rapid deployment scenarios and testing under fire.

```rust
use tank::Driver;
use tank_duckdb::{DuckDBConnection, DuckDBDriver};

async fn establish_duckdb_connection() -> Result<DuckDBConnection> {
    let driver = DuckDBDriver::new();
    let connection = driver
        .connect("duckdb://../target/debug/combat.duckdb?mode=rw".into())
        .await?;
    Ok(connection)
}
```

**URL Format**:
- File:`duckdb://path/to/database.duckdb?mode=rw`
- Memory: `duckdb://:memory:`

Modes:
- `mode=ro`: read-only access (fails if the database doesn’t exist)
- `mode=rw`: read-write access (creates the database if it doesn’t exist)
- `mode=rwc`: alias for `rw`
- `mode=memory`: in-memory access (creates a temporary database that lives only for the duration of the connection)

The `mode` parameter provides a common syntax for specifying connection access, similar to SQLite. The values map respectively to `access_mode=READ_ONLY`, `access_mode=READ_WRITE`, `access_mode=READ_WRITE` and the special `duckdb://:memory:` path. Additional URL parameters are passed directly to the DuckDB C API. See the full list of supported options on the [DuckDB website](https://duckdb.org/docs/stable/configuration/overview#global-configuration-options)

#### SQLite
SQLite is your trusty sidearm: lightweight, reliable, zero configuration. Deploy anywhere, anytime.

```rust
use tank::Driver;
use tank_sqlite::{SQLiteConnection, SQLiteDriver};

async fn establish_sqlite_connection() -> Result<SQLiteConnection> {
    let driver = SQLiteDriver::new();
    let connection = driver
        .connect("sqlite://../target/debug/operations.sqlite?mode=rwc".into())
        .await?;
    Ok(connection)
}
```
  
**URL Format**:
- File: `sqlite://path/to/database.sqlite?mode=rwc`
- Memory: 
- Same mode flags as DuckDB
- In-memory operations: `sqlite://:memory:`

#### Postgres
Postgres is your heavy artillery: powerful, networked, built for sustained campaigns with multiple units coordinating strikes.

```rust
use tank_postgres::PostgresConnection;
use tank::Connection;

async fn establish_postgres_connection() -> Result<PostgresConnection> {
    let connection = PostgresConnection::connect(
        "postgres://tank-user:armored@127.0.0.1:5432/military".into()
    )
    .await?;
    Ok(connection)
}
```

**URL Format**: `postgres://user:password@host:port/database`
- Standard PostgreSQL connection string
- Supports all libpq parameters
- Connection pooling handled automatically

## Operations Briefing
- [`prepare("SELECT * FROM ...*".into())`](https://docs.rs/tank/latest/tank/trait.Executor.html#tymethod.prepare):
  Compiles a raw SQL string into a reusable [`Query<Driver>`](https://docs.rs/tank/latest/tank/enum.Query.html) object without firing it. Use when the same statement will be dispatched multiple times.

- [`run(query.into())`](https://docs.rs/tank/latest/tank/trait.Executor.html#tymethod.run):
  Streams a mixed feed of [`QueryResult`](https://docs.rs/tank/latest/tank/enum.QueryResult.html) values (`Row` or `Affected`). Use when you want to run multiple statements (e.g. INSERT INTO followed by SELECT), or you are not sure what result type you might receive.

- [`fetch(query.into())`](https://docs.rs/tank/latest/tank/trait.Executor.html#method.fetch):
  Precise extraction. Wraps `run` and streams only row results (`QueryResult::Row`), executing all statements while filtering out counts.

- [`execute(query.into())`](https://docs.rs/tank/latest/tank/trait.Executor.html#method.execute):
  Complement to `fetch` for impact reports: awaits the stream and aggregates all `QueryResult::Affected` values into a single `RowsAffected` total (INSERT / UPDATE / DELETE). Row payloads are ignored.

- [`append(query.into())`](https://docs.rs/tank/latest/tank/trait.Executor.html#method.append):
  Convenience bulk insert for an iterator of entities. Builds an INSERT (or driver-optimized append if supported) and returns `RowsAffected`. Use when staging large batches into a table.

- [`begin()`](https://docs.rs/tank/latest/tank/trait.Connection.html#tymethod.begin):
  Launch a coordinated operation. Borrow the connection and yield a transactional executor. Issue any of the above ops against it, then `commit` (secure ground) or `rollback` (tactical retreat). Uncommitted drop triggers a rollback and gives back the connection.

## Transaction
Sometimes you need to execute multiple operations as a single atomic mission - all or nothing. That's where **Transactions** come in. You [`begin()`](https://docs.rs/tank/latest/tank/trait.Connection.html#tymethod.begin) a transaction, execute your operations, then either [`commit()`](https://docs.rs/tank/latest/tank/trait.Transaction.html#tymethod.commit) (mission success) or [`rollback()`](https://docs.rs/tank/latest/tank/trait.Transaction.html#tymethod.rollback) (abort and retreat). Uncommitted drop triggers a rollback and gives back the connection.

Transactions support depends on the specific driver and database capabilities. This is a thin layer over the database's native transaction concept. For databases without transaction support, `begin` should return an error.

## Connection Lifecycle
1. **Establish**: Call [`Connection::connect("dbms://...").await?`](https://docs.rs/tank/latest/tank/trait.Connection.html#tymethod.connect) with your database URL.
2. **Deploy**: Use the connection for queries, inserts, updates, and deletes.
3. **Lock (optional)**: Start a transaction with `connection.begin().await?`. This exclusively borrows the connection. Issue all statements through the transaction handle. On `commit()` (or `rollback()`) and get back the connection.
4. **Maintain**: Connection pooling is handled automatically by the driver.
5. **Terminate**: Connections close automatically when dropped.

*Lock, commit, advance. Dismissed*
