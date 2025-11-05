# Driver Creation
###### *Field Manual Section 8* - Armored Engineering

You want to open a new battlefront. That means forging a fresh **Driver**: the armored bridge between Tank's high‑level abstractions and a new database engine's trenches (type mapping, prepared semantics, transaction doctrine). This section walks you through building a crate from zero to live fire, then certifying it on the shared proving ground (`tank-tests`).

## Mission Objectives
- Stand up a new `tank-<backend>` crate
- Implement the core traits: `Driver`, `Connection`, `Executor`, `Prepared`, `SqlWriter`
- (Optional) Provide transactional support via `DriverTransactional` + `Transaction`
- Specialize dialect printing (DDL / DML / literals / parameters)
- Integrate with the shared test suite, gating unsupported munitions with feature flags
- Ship a lean, consistent `Cargo.toml` matching existing armor plating

## Battlefield Topography
A driver is a thin composite of five moving parts:
| Trait                 | Purpose                                                                             |
| --------------------- | ----------------------------------------------------------------------------------- |
| `Driver`              | Public entry point: builds connections and hands out a dialect writer               |
| `DriverTransactional` | optional `Driver` variant supporting scoped atomic operations: commit and rollback  |
| `Executor`            | Core query interface: prepares statements, runs queries, streams results, ...       |
| `Connection`          | Live session implementing `Executor`, may start transactional scopes with `begin()` |
| `Prepared`            | Owns a compiled statement, binds positional parameters                              |
| `SqlWriter`           | Converts Tank's operations and semantic AST fragments into backend query  language  |

All other machinery (entities, expressions, joins) already speak through these interfaces.

## Forge the Crate
Create `tank-yourdb` in your favorite source repository.

`Cargo.toml` template (adjust backend dependency + features):
```toml
[package]
name = "tank-yourdb"
description = "YourDB driver implementation for Tank: the Rust data layer"
version = "0.1.0"
authors = ["your_name <email@address.com>"]
license = "Apache-2.0"
edition = "2024"
repository = "https://github.com/your_name/tank-yourdb"
categories = ["database"]

[features]
# Example: default = ["bundled"]
# Add backend-specific toggles; keep minimal.

[dependencies]
log = "0"
rust_decimal = "1"
tank-core = "1"
time = "0" # if supported
url = "2"
uuid = "1"
yourdb-sys = "<version>"

[dev-dependencies]
tank-tests = "1"
```

## Assembly Steps
### 1. The Driver Shell
```rust
use tank_core::{Driver, DriverTransactional};

#[derive(Clone, Copy, Default)]
pub struct YourDBDriver;
impl YourDBDriver { pub const fn new() -> Self { Self } }

impl Driver for YourDBDriver {
    type Connection = YourDBConnection;
    type SqlWriter = YourDBSqlWriter;
    type Prepared = YourDBPrepared;
    const NAME: &'static str = "yourdb";
    fn sql_writer(&self) -> Self::SqlWriter { YourDBSqlWriter::default() }
}

// If transactions are supported
impl DriverTransactional for YourDBDriver { type Transaction<'c> = YourDBTransaction<'c>; }
```

### 2. Connection + Executor
Responsibilities:
- Validate / parse URL (enforce `yourdb://` prefix)
- Open / pool backend session(s)
- Implement `prepare` (compile statement) & `run` (stream `QueryResult::{Row,Affected}`)
- Optionally implement fast-path bulk `append` (DuckDB style)

Skeleton:
```rust
use tank_core::{Connection, Executor, Driver, Query, QueryResult, Result, RowsAffected, stream::Stream};
use std::{borrow::Cow, future::Future};

pub struct YourDBConnection { /* raw handle(s), flags */ }

impl Executor for YourDBConnection {
    type Driver = YourDBDriver;
    fn driver(&self) -> &Self::Driver { &YourDBDriver }
    async fn prepare(&mut self, sql: String) -> Result<Query<Self::Driver>> {
        // Compile backend statement; wrap into YourDBPrepared
        let stmt = backend_prepare(&sql)?; // pseudo
        Ok(YourDBPrepared::new(stmt).into())
    }
    fn run(&mut self, query: Query<Self::Driver>) -> impl Stream<Item = Result<QueryResult>> + Send {
        // Dispatch raw vs prepared; translate messages into Row / Affected
        your_stream_adapter(query)
    }
    async fn append<'a, E, It>(&mut self, rows: It) -> Result<RowsAffected>
    where E: tank_core::Entity + 'a, It: IntoIterator<Item = &'a E> + Send {
        // Optional optimization; else omit and inherit default
        unimplemented!()
    }
}

impl Connection for YourDBConnection {
    async fn connect(url: Cow<'static, str>) -> Result<Self> {
        // Parse URL, open session, return Self
        ensure_prefix(&url)?; // yourdb://
        Ok(Self { /* ... */ })
    }
    fn begin(&mut self) -> impl Future<Output = Result<YourDBTransaction<'_>>> {
        YourDBTransaction::new(self) // if supported
    }
}
```

### 3. Prepared Ordnance
Implement parameter binding according to backend type system. Convert each Rust value from `AsValue` into the native representation.
```rust
use tank_core::{Prepared, AsValue, Result};
use std::fmt::{self, Display};

pub struct YourDBPrepared { raw: BackendStmt, next: u64 }
impl YourDBPrepared { pub fn new(raw: BackendStmt) -> Self { Self { raw, next: 1 } } }
impl Display for YourDBPrepared { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{:p}", &self.raw) } }
impl Prepared for YourDBPrepared {
    fn bind<V: AsValue>(&mut self, v: V) -> Result<&mut Self> { self.bind_index(v, self.next) }
    fn bind_index<V: AsValue>(&mut self, v: V, index: u64) -> Result<&mut Self> {
        let value = v.as_value(); // match and write into backend
        backend_bind(&self.raw, index, &value)?; // pseudo
        self.next = index + 1;
        Ok(self)
    }
}
```

### 4. Dialect Scribe (`SqlWriter`)
Override only differences from the generic fallback:
- Identifier quoting style
- Column type mapping
- Literal escaping quirks (BLOB, INTERVAL, UUID, arrays)
- Parameter placeholder (override `write_expression_operand_question_mark`) if not `?`
- Schema operations (skip if engine lacks schemas like SQLite)
- Upsert syntax via `write_insert_update_fragment` if divergence

Tip: Start from `tank-core`'s `GenericSqlWriter` implementation; copy then trim.
```rust
#[derive(Default)]
pub struct YourDBSqlWriter;
impl SqlWriter for YourDBSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter { self }
    fn write_column_type(&self, ctx: &mut tank_core::Context, out: &mut String, value: &tank_core::Value) {
        match value { /* map to engine native types */ _ => out.push_str("TEXT") }
    }
    fn write_expression_operand_question_mark(&self, _ctx: &mut tank_core::Context, out: &mut String) {
        out.push_str("$"); // Example: use positional tokens like $1 (then adjust binder)
    }
}
```

### 5. Transactions (Optional Theater)
If supported:
- Implement a `YourDBTransaction<'c>` type holding a mutable borrow of the connection + backend handle
- Provide `commit()` / `rollback()` on `Transaction` impl; ensure resource release
- Expose via `DriverTransactional` associated `Transaction<'c>` type

If not: expose `disable-transactions` in dev dependency features and let tests skip.

### 6. Test Range Certification
Add an integration test `tests/yourdb.rs`:
```rust
#[cfg(test)]
mod tests {
    use tank_core::Driver; // OR Connection if connecting directly
    use tank_yourdb::YourDBDriver; // or YourDBConnection
    use tank_tests::{execute_tests, init_logs};

    #[tokio::test]
    async fn yourdb() {
        init_logs();
        let driver = YourDBDriver::new();
        let connection = driver
            .connect("yourdb://localhost:9000?param=value".into())
            .await
            .expect("Cannot connect");
        execute_tests(connection).await;
    }
}
```
Tune feature flags until green. Each failing test is a missing capability, not an annoyance—treat it like a misfiring barrel.

### Feature Flags Doctrine
`tank-tests` exposes opt-out switches:
- `disable-arrays`, `disable-lists`, `disable-maps` – collections not implemented
- `disable-intervals` – interval types absent
- `disable-large-integers` – `i128` / `u128` unsupported
- `disable-ordering` – cannot order columns in SELECT
- `disable-references` – foreign keys not enforced
- `disable-transactions` – no transactional support

### 7. Tactical Checklist
- URL prefix enforced? (`yourdb://`)
- `Driver::NAME` correct and used consistently
- `prepare` handles multiple statements (or rejects cleanly)
- Streams drop promptly (no leaked locks / file handles)
- `SqlWriter` prints multi‑statement sequences with proper separators & terminal `;`
- Upsert path (`save()`) works if PK exists; documented fallback if not supported
- Feature flags trimmed to actual unsupported payloads
- Transaction begin -> commit/rollback validated (if available)

## Minimal End‑to‑End Skeleton
```rust
// lib.rs (of tank-yourdb)
pub mod driver; pub mod connection; pub mod prepared; pub mod sql_writer; pub mod transaction; // optional
pub use driver::YourDBDriver;
```

Remove a flag the moment your driver truly supports the capability. Each removed flag unlocks corresponding test sorties.

## Performance Brief
- Prefer streaming APIs over buffering entire result sets.
- Implement backend bulk ingestion if native (like DuckDB's appender) for `append()`.
- Reuse prepared statements internally if engine offers server‑side caching.

## Failure Signals
Return early with rich context:
- Wrong URL prefix: immediate `Error::msg("YourDB connection url must start with `yourdb://`")`
- Prepare failure: attach truncated query text (`truncate_long!` style) to context
- Bind failure: specify parameter index and offending value type

## Example Dev Dependency Evolution
Start
```toml
tank-tests = { path = "../tank-tests", features = ["disable-arrays", "disable-intervals", "disable-large-integers", "disable-lists", "disable-maps", "disable-transactions"] }
```
After adding arrays + transactions
```toml
tank-tests = { path = "../tank-tests", features = ["disable-intervals", "disable-large-integers", "disable-maps"] }
```
Final (fully armed)
```toml
tank-tests = { path = "../tank-tests" }
```

*Fabricate the engine. Fuel the advance. Tank out.*
