use crate::{Driver, Executor, Result, Transaction};
use std::{borrow::Cow, future::Future};

pub trait Connection: Executor {
    /// Create a connection pool with at least one connection established to the given URL
    fn connect(
        url: Cow<'static, str>,
    ) -> impl Future<Output = Result<<Self::Driver as Driver>::Connection>>;

    fn begin(&mut self) -> impl Future<Output = Result<impl Transaction<'_>>>;
}
