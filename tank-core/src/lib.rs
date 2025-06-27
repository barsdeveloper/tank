#![feature(array_try_from_fn)]
mod column;
mod connection;
mod data_set;
mod driver;
mod entity;
mod executor;
mod expression;
mod interval;
mod join;
mod prepared;
mod query;
mod relations;
mod sql_writer;
mod table_ref;
mod util;
mod value;

pub use column::*;
pub use connection::*;
pub use data_set::*;
pub use driver::*;
pub use entity::*;
pub use executor::*;
pub use expression::*;
pub use interval::*;
pub use join::*;
pub use prepared::*;
pub use query::*;
pub use relations::*;
pub use sql_writer::*;
pub use table_ref::*;
pub use util::*;
pub use value::*;

pub use ::anyhow::Context;
pub mod stream {
    pub use ::futures::stream::*;
}
pub use ::futures::future;

pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;
