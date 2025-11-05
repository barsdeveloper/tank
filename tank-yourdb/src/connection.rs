use crate::{YourDBDriver, YourDBPrepared, YourDBTransaction};
use tank_core::{
    Connection, Driver, Error, Executor, Query, QueryResult, Result, Transaction,
    stream::{self, Stream},
};

pub struct YourDBConnection {}

impl Executor for YourDBConnection {
    type Driver = YourDBDriver;

    fn driver(&self) -> &Self::Driver {
        &YourDBDriver {}
    }

    async fn prepare(&mut self, query: String) -> Result<Query<Self::Driver>> {
        // Return Err if not supported
        Ok(Query::Prepared(YourDBPrepared::new()))
    }

    fn run(
        &mut self,
        query: Query<Self::Driver>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        stream::iter([])
    }
}

impl Connection for YourDBConnection {
    async fn connect(
        url: std::borrow::Cow<'static, str>,
    ) -> Result<<Self::Driver as Driver>::Connection> {
        todo!()
    }

    #[allow(refining_impl_trait)]
    async fn begin(&mut self) -> Result<YourDBTransaction<'_>> {
        Err(Error::msg("Transations are not supported by YourDB"))
    }
}
