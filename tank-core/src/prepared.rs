use crate::{AsValue, Result};
use std::fmt::Display;

/// A parameterized, backend-prepared query handle.
///
/// `Prepared` enables drivers to pre-parse / optimize SQL statements and later
/// bind positional parameters. Values are converted via the `AsValue` trait.
///
/// # Binding Semantics
/// * `bind` appends a value (driver chooses actual placeholder numbering).
/// * `bind_index` sets the parameter at `index` (from 0).
///
/// Methods return `&mut Self` for fluent chaining:
/// ```rust,ignore
/// prepared.bind(42)?.bind("hello")?;
/// ```
pub trait Prepared: Send + Sync + Display {
    /// Append a parameter value.
    fn bind<V: AsValue>(&mut self, value: V) -> Result<&mut Self>;
    /// Bind a value at a specific index.
    fn bind_index<V: AsValue>(&mut self, value: V, index: u64) -> Result<&mut Self>;
}
