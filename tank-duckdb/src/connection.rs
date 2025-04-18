use crate::driver::DuckDBDriver;
use anyhow::Result;
use futures::stream::BoxStream;
use tank_core::{Connection, Executor, Query, QueryResult};

#[derive(Debug)]
pub struct DuckDBConnection {}

impl Executor for DuckDBConnection {
    type Driver = DuckDBDriver;

    fn driver(&self) -> &Self::Driver {
        &DuckDBDriver {}
    }

    fn run<'a>(&mut self, query: Query) -> BoxStream<'a, Result<QueryResult>> {
        todo!()
    }
}

impl Connection for DuckDBConnection {}
