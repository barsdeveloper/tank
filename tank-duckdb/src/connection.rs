use crate::driver::DuckDBDriver;
use anyhow::Result;
use futures::stream::BoxStream;
use tank_core::{Connection, Executor};

#[derive(Debug)]
pub struct DuckDBConnection {}

impl Executor for DuckDBConnection {
    type Driver = DuckDBDriver;

    fn run<'a>(&self, query: tank_core::Query) -> BoxStream<'a, Result<tank_core::QueryResult>> {
        todo!()
    }
}

impl Connection for DuckDBConnection {}
