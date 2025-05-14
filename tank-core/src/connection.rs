use crate::{Executor, Result};
use std::future::Future;

pub trait Connection: Executor {
    /// Initial part of the connect url
    const PREFIX: &'static str;

    /// Create a connection pool with at least one connection established to the given URL
    fn connect(url: &str) -> impl Future<Output = Result<impl Connection>>;
}
