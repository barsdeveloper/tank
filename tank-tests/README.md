<div align="center">
    <img width="300" height="300" src="../docs/public/logo.png" alt="Tank: Table Abstraction & Navigation Kit logo featuring a green tank with a gear background and stacked database cylinders" />
</div>

# tank-tests

Reusable integration test suite for the for [Tank](https://crates.io/crates/tank): the Rust data layer. Instead of duplicating many nearly identical tests inside every driver crate, `tank-tests` centralizes them with feature flags so each driver can opt in or gracefully skip unsupported domains.

## Why it exists
- Ensures consistency: all drivers pass the same baseline CRUD, query, join, multi‑statement and type coverage.
- Reduces maintenance: add a new scenario once, every driver benefits.
- Accelerates new driver development: run the suite early, disable what you have not implemented yet, iterate.

Each module focuses on a distinct aspect of Tank functionality. They are orchestrated by `execute_tests` in `src/lib.rs`.

## Feature flags
The crate exposes opt-out feature flags ("disable-*") that skip entire capability families when a driver cannot yet support them.

| Flag                     | Skips tests that use                       |
| ------------------------ | ------------------------------------------ |
| `disable-arrays`         | Fixed-size arrays                          |
| `disable-intervals`      | `Interval` an advanced duration handling   |
| `disable-large-integers` | `i128` and `u128` columns                  |
| `disable-lists`          | List/array-like dynamic collection types   |
| `disable-maps`           | Map containers                             |
| `disable-ordering`       | Explicit result ordering                   |
| `disable-references`     | Referential integrity                      |
| `disable-transactions`   | Transaction begin/commit/rollback coverage |

Use them from the driver crate's `Cargo.toml`:
```toml
[dev-dependencies]
tank-tests = { path = "../tank-tests", default-features = false, features = ["disable-arrays", "disable-intervals"] }
```

## How to use in a driver crate
Add as a dev-dependency and invoke `execute_tests` in a `tokio::test`. Call `init_logs()` to get structured output; wrap with a mutex if your database backend cannot handle concurrent schema drops/creates.

```rust
#[tokio::test]
async fn my_driver_full_suite() {
    use tank_tests::{execute_tests, init_logs};
    use my_driver::MyConnection; // Your driver connection type implementing `Connection`

    init_logs();
    let connection = MyConnection::connect("mydb://localhost:5555".into())
        .await
        .expect("Could not connect");
    execute_tests(connection).await; // Runs all enabled modules sequentially
}
```

### Example: DuckDB (from `tank-duckdb/tests/duckdb.rs`)
```rust
let driver = DuckDBDriver::new();
let connection = driver.connect("duckdb://../target/debug/tests.duckdb?mode=rw".into()).await?;
execute_tests(connection).await;
```

### Example: Postgres (from `tank-postgres/tests/postgres.rs`)
```rust
let connection = PostgresConnection::connect(url.into()).await?;
execute_tests(connection).await;
```

## Logging control
- `init_logs()` sets a test-friendly format and default WARN level (override via `RUST_LOG`).
- `silent_logs! { ... }` macro temporarily mutes logging for noisy negative-path assertions.

## Adding a new test scenario
1. Create a new module file under `src/` (e.g. `geo.rs`).
2. Implement an async `pub fn geo<E: Executor>(executor: &mut E)` or `pub fn geo<C: Connection>(connection: &mut C)` depending on needs.
3. Import and call it inside `execute_tests` (`src/lib.rs`). Keep ordering logical (lightweight → heavy).
4. Gate it with a feature flag if optional: add `disable-geo` to `[features]` and wrap the call with `#[cfg(not(feature = "disable-geo"))]`.
5. Prefer deterministic data; use `LazyLock<Mutex<()>>` if the test mutates shared tables.

## Guidance / best practices
- Keep each scenario independent: it must create/drop its own tables; never assume prior state.
- Use `drop_table(..., true, false)` followed by `create_table(..., <schema>, <if_not_exists>)` to ensure repeatability.
- Prefer small batch insert sizes unless testing bulk performance.
- When verifying multi-statement execution, assemble SQL via the driver's `SqlWriter` then stream results.
- For performance or stress tests (e.g. `insane`), maintain clear constants documenting expected totals.

## Skipping unsupported features
If your driver cannot implement a capability yet, enable the corresponding `disable-*` feature locally (as shown above) to keep the suite green while you iterate.

## Contributing
Pull requests adding coverage or tightening assertions are welcome:
- Include rationale & backend considerations.
- Avoid backend-specific syntax leaks in shared tests; rely on Tank abstractions & `SqlWriter`.
- Ensure new types are guarded by appropriate disable flags if not universally supported.

## License
This crate inherits the workspace license (see `../LICENSE`).

## FAQ
**Q: Why opt-out rather than opt-in features?**
A: New drivers should exercise everything available; disabling is a conscious acknowledgment of a gap.

**Q: Can I run a single scenario?**
A: Yes: directly call its function in a custom test (e.g. `tank_tests::simple(&mut connection).await`). The orchestrator is convenience only.

**Q: Does it benchmark drivers?**
A: No. All assertions are functional. You can wrap calls with your own timing if desired.

---
*Rustaceans don't hide behind ORMs, they drive Tanks — and they test their armor.*
