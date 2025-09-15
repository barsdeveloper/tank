use crate::{Executor, Result};
use std::{borrow::Cow, future::Future};

pub trait Connection: Executor {
    /// Create a connection pool with at least one connection established to the given URL
    fn connect(url: Cow<'static, str>) -> impl Future<Output = Result<impl Connection>>;
}
