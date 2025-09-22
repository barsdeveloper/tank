use crate::{AsValue, Result};
use std::fmt::Display;

pub trait Prepared: Send + Sync + Display {
    fn bind<V: AsValue>(&mut self, value: V) -> Result<&mut Self>;
    fn bind_index<V: AsValue>(&mut self, value: V, index: u64) -> Result<&mut Self>;
}
