//! Core primitives for the `tank` ORM / query toolkit.
//!
//! This crate exposes a thin, async-first abstraction layer over SQL
//! backends.  The goal is portability – the same entity and query
//! building code should compile and run against multiple SQL
//! engines with minimal (ideally zero) changes.
//!
//! # Design Highlights
//! * "Bring your own runtime" – traits return `impl Future` / `impl Stream`.
//!   You can use Tokio, async-std, or any executor capable of driving
//!   `futures` streams. This varies from driver to driver.
//! * Streaming results (`Executor::run` / `fetch`) to avoid buffering entire
//!   result sets.
//! * A rich `Value` enum representing typed SQL values (including arrays,
//!   maps, structs, intervals, decimals) with conversion helpers via `AsValue`.
//! * Declarative entity definition through the `Entity` trait + derive macro.
//! * Dialect pluggability: the `Driver` supplies a `SqlWriter` that renders
//!   the appropriate SQL for its backend.
//! * Composability: expressions / datasets build larger statements (SELECT,
//!   INSERT, DELETE) without stringly-typed concatenation.
//!
//! # Quick Start
//! ```rust,no_run
//! use tank::{Entity, Executor};
//! use uuid::Uuid;
//!
//! #[derive(tank::Entity)]
//! #[tank(schema = "operations")]
//! struct Operator {
//!     #[tank(primary_key)]
//!     id: Uuid,
//!     callsign: String,
//!     #[tank(name = "rank")]
//!     service_rank: String,
//! }
//!
//! async fn demo<E: Executor>(exec: &mut E, id: Uuid) -> anyhow::Result<Option<Operator>> {
//!     // Ensure table exists (idempotent when supported by driver)
//!     Operator::create_table(exec, true, true).await?;
//!
//!     // Upsert an operator
//!     Operator { id, callsign: "Red-One".into(), service_rank: "LT".into() }
//!         .save(exec).await?;
//!
//!     // Fetch it back
//!     let op = Operator::find_one(exec, &true).await?; // bool implements Expression
//!     Ok(op)
//! }
//! ```
//!
//! # Error Handling
//! All fallible operations use crate-level `Result<T>` (`anyhow::Result<T>`).
//! For detailed context attachers prefer `ErrorContext`.
//!
//! # Futures & Streams MUST be driven
//! Any method returning a future or stream performs no guaranteed side-effect
//! until it is awaited / fully consumed. Never drop them silently.
//!
//! # Feature Flags
//! Drivers may expose feature flags (e.g. disabling 128-bit integers). This
//! crate itself is mostly feature-free; consult driver crates for details.
//!
//! # Modules
//! The public surface re-exports most modules. Common entry points:
//! * `Entity` – declare and persist typed rows.
//! * `Executor` – run queries, fetch streams.
//! * `Query`, `QueryResult` – represent prepared/raw statements & results.
//! * `Value`, `AsValue` – value type system & conversions.
//! * `SqlWriter` – backend SQL generation.
//! * `Interval` – time span type usable as SQL INTERVAL.
//! * `Join`, `Expression` – build complex predicates / SELECT trees.
//!
//! For more elaborate examples inspect the `tank-tests` crate in the
//! repository (`src/simple.rs`, `src/transaction1.rs`, etc.).
//!
//! # Macros
//! Supporting macros like `truncate_long!`, `possibly_parenthesized!` and
//! `take_until!` assist with SQL / token generation. They are re-exported at
//! crate root for macro expansion within derives.
//!
//! # Safety & Portability Notes
//! * Do not rely on side-effects before awaiting returned futures.
//! * Always exhaust streams if the driver might hold transactional or cursor
//!   resources.
//! * When mapping `Value` conversions prefer `AsValue::try_from_value` for
//!   graceful errors.
//!
//! # Contributing Docs
//! When adding new traits or types ensure they carry `///` rustdoc including:
//! * Purpose & invariants
//! * Example snippet (consider pulling from tests)
//! * Error semantics
//!
mod as_value;
mod column;
mod connection;
mod data_set;
mod decode_type;
mod driver;
mod entity;
mod executor;
mod expression;
mod interval;
mod join;
mod prepared;
mod query;
mod relations;
mod table_ref;
mod transaction;
mod util;
mod value;
mod writer;

pub use ::anyhow::Context as ErrorContext;
pub use as_value::*;
pub use column::*;
pub use connection::*;
pub use data_set::*;
pub use decode_type::*;
pub use driver::*;
pub use entity::*;
pub use executor::*;
pub use expression::*;
pub use interval::*;
pub use join::*;
pub use prepared::*;
pub use query::*;
pub use relations::*;
pub use table_ref::*;
pub use transaction::*;
pub use util::*;
pub use value::*;
pub use writer::*;
pub mod stream {
    pub use ::futures::stream::*;
}
pub use ::futures::future;
pub use ::futures::sink;

/// Crate-wide result alias using `anyhow` for flexible error context.
pub type Result<T> = anyhow::Result<T>;
/// Crate-wide error alias using `anyhow`.
pub type Error = anyhow::Error;
