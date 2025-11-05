# Driver Creation
###### *Field Manual Section 8* - Armored Engineering

You want to open a new battlefront. That means forging a fresh **Driver**: the armored bridge between Tank's high‑level abstractions and a new database engine's trenches (type mapping, prepared semantics, transaction doctrine). This section walks you through building a crate from zero to live fire, then certifying it on the shared proving ground (`tank-tests`).

## Mission Objectives
- Stand up a new `tank-<backend>` crate
- Implement the core trait: `Driver`, that is the starting point, keep expanding from there
- Specialize dialect printing (specific query language syntax can be implemented by overriding the default methods from `SqlWriter`)
- Integrate with the shared test suite, gating unsupported munitions with feature flags
- Ship a lean, consistent crate matching existing armor plating

## Battlefield Topography
A driver is a thin composite of five moving parts:
| Trait                 | Purpose                                                                            |
| --------------------- | ---------------------------------------------------------------------------------- |
| `Driver`              | Public entry point for all the database abstractions                               |
| `Connection`          | Live session running queries and possibly starting a transaction                   |
| `Transaction`         | Abstraction over transactional database capabilities, borrows mutably a connection |
| `Prepared`            | Owns a compiled statement, binds positional parameters                             |
| `SqlWriter`           | Converts Tank's operations and semantic AST fragments into backend query language  |

All other machinery (entities, expressions, joins) already speak through these interfaces.

## Forge the Crate
Create `tank-yourdb` in your favorite source repository.

`Cargo.toml` template (adjust backend dependency + features):

<<< @/../tank-yourdb/Cargo.toml

## Assembly Steps
### 1. The Driver Shell
<<< @/../tank-yourdb/src/driver.rs

### 2. Connection + Executor
Responsibilities:
- Validate / parse URL (enforce `yourdb://` prefix)
- Open / pool backend session(s)
- Implement `prepare` (compile statement) & `run` (stream `QueryResult::{Row,Affected}`)
- Optionally implement fast-path bulk `append` (DuckDB style)

Skeleton:

<<< @/../tank-yourdb/src/connection.rs

### 3. Prepared Ordnance
Implement parameter binding according to backend type system. Convert each Rust value from `AsValue` into the native representation.

<<< @/../tank-yourdb/src/prepared.rs

### 4. Dialect Scribe (`SqlWriter`)
Override only differences from the generic fallback:
- Identifier quoting style
- Column type mapping
- Literal escaping quirks (BLOB, INTERVAL, UUID, arrays)
- Parameter placeholder (override `write_expression_operand_question_mark`) if not `?`
- Schema operations (skip if engine lacks schemas like SQLite)
- Upsert syntax via `write_insert_update_fragment` if divergence

Tip: Start from `tank-core`'s `GenericSqlWriter` implementation; copy then trim.

<<< @/../tank-yourdb/src/sql_writer.rs

### 5. Transactions
- Implement a `YourDBTransaction<'c>` type holding a mutable borrow of the connection.
- Provide `commit()` and `rollback()` on methods, ensure resource release.
- Expose via `Driver` associated `Transaction<'c>` type

If not supported, return relevant error messages in related functions and enable `disable-transactions` in `tank-tests`.

### 6. Test Range Certification
Add an integration test `tests/yourdb.rs`:

<<< @/../tank-yourdb/tests/yourdb.rs

Enable feature flags to disable specific functionality until green.

### Feature Flags Doctrine
`tank-tests` exposes opt-out switches:
- `disable-arrays`, `disable-lists`, `disable-maps`: collections not implemented
- `disable-intervals`: interval types absent
- `disable-large-integers`: `i128`, `u128` unsupported
- `disable-ordering`: yourdb does not support result ordering
- `disable-references`: foreign keys not enforced
- `disable-transactions`: no transactional support

### 7. Tactical Checklist
- URL prefix enforced (`yourdb://`)
- `Driver::NAME` correct and used consistently
- `prepare` handles multiple statements (or rejects cleanly)
- Streams drop promptly (no leaked locks / file handles)
- `SqlWriter` prints multi‑statement sequences with proper separators and terminal `;`
- Upsert path (`save()`) works if PK exists; documented fallback if not supported

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
