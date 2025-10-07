# Introduction
###### *Field Manual Section 1* – Mission Briefing
Welcome to the Tank field manual. This is the quick-and-mean guide for developers who want to drive, fight and survive with Tank (Table Abstraction & Navigation Kit): the Rust data layer.

## Mission objectives
- **Async operations** - fire and forget.
- **Simple workflow** - every query is visible on your tactical map.
- **Extensible driver system** - swap databases like changing magazines mid-battle.
- **SQL and NoSQL support**: one Tank, all terrains.
- **Transactional strikes**: commit on success or rollback the mission if the plan goes sideways.
- **Rich type arsenal** with automatic conversions between Rust and database types.
- **Optional appender API** for high caliber bulk inserts.

## Equipment
- [**tank**](https://crates.io/crates/tank): The command vehicle, rallying the core arsenal and procedural macro firepower for seamless battlefield operations.
- [**tank-duckdb**](https://crates.io/crates/tank-duckdb): DuckDB driver.
- [**tank-sqlite**](https://crates.io/crates/tank-sqlite): Sqlite driver.
- [**tank-postgres**](https://crates.io/crates/tank-postgres): Postgres driver.
- [**tank-mysql**](https://crates.io/crates/tank-mysql): Mysql driver.
- [**tank-core**](https://crates.io/crates/tank-core): All the heavy machinery that makes the Tank move.
- [**tank-macros**](https://crates.io/crates/tank-macros): Because Rust requires procedural macros to live in a separate silo, this crate houses the derive magic.
- [**tank-tests**](https://crates.io/crates/tank-tests): The proving ground. Shared integration tests used by every driver to ensure that when the shooting starts, nothing jams.

## Why Tank?
Tank is a thin, battle-ready layer over your database workflow.
It keeps the learning curve low, just a handful of clear concepts. It stays deliberately lean so you can maneuver fast and stay flexible.

Because its scope is tight, Tank can deploy on many fronts, from classic SQL databases to non-SQL theaters. At the same time, it doesn’t limit your capabilities: Tank never tries to hide SQL behind a heavy query builder: you can write plain SQL whenever you need and still benefit from its rich type-conversion features. In this way, it’s similar in spirit to [SQLx](https://crates.io/crates/sqlx) but unlike SQLx, Tank does not perform compile-time SQL validation. It prioritizes runtime flexibility and multi-database support over static checking.

Tank also provides convenient methods to set up tables and get database communication running in just a few lines of code, all through a unified API that works the same regardless of the backend. Perfect to setup testing very quickly.
