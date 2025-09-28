use crate::{Executor, Result};

pub trait Transaction<'c>: Executor {
    fn commit(self) -> impl Future<Output = Result<()>>;
    fn rollback(self) -> impl Future<Output = Result<()>>;
}
