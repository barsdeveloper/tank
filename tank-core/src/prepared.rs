use crate::{AsValue, Result};
use std::fmt::Display;

pub trait Prepared: Send + Sync + Display {
    fn bind<V: AsValue>(&mut self, v: V) -> Result<&mut Self>;
}
