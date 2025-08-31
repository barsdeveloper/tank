use crate::{
    AsValue, Driver, Executor, Result, Value,
    future::FutureExt,
    printable_query,
    stream::{Stream, StreamExt},
};
use std::{
    fmt::{self, Display},
    pin::pin,
    sync::Arc,
};

pub trait Prepared: Clone + Send + Sync + Display {
    fn bind<V: AsValue>(&mut self, v: V) -> Result<&mut Self>;
}

#[derive(Clone)]
pub enum Query<P: Prepared> {
    Raw(Arc<str>),
    Prepared(P),
}

impl<P: Prepared> Query<P> {
    pub fn run<'e, Exec: Executor<Driver = impl Driver<Prepared = P>>>(
        self,
        executor: &mut Exec,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        executor.run(self)
    }
    pub fn fetch_one<Exec: Executor<Driver = impl Driver<Prepared = P>>>(
        self,
        executor: &mut Exec,
    ) -> impl Future<Output = Result<Option<RowLabeled>>> + Send {
        let stream = executor.fetch(self);
        async move { pin!(stream).into_future().map(|(v, _)| v).await.transpose() }
    }
    pub fn fetch_many<Exec: Executor<Driver = impl Driver<Prepared = P>>>(
        self,
        executor: &mut Exec,
    ) -> impl Stream<Item = Result<RowLabeled>> + Send {
        executor.fetch(self)
    }
}

impl<P: Prepared> From<&str> for Query<P> {
    fn from(value: &str) -> Self {
        Query::Raw(value.into())
    }
}

impl<P: Prepared> From<String> for Query<P> {
    fn from(value: String) -> Self {
        Query::Raw(value.into())
    }
}

impl<P: Prepared> From<Arc<str>> for Query<P> {
    fn from(value: Arc<str>) -> Self {
        Query::Raw(value)
    }
}

impl<P: Prepared> From<P> for Query<P> {
    fn from(value: P) -> Self {
        Query::Prepared(value)
    }
}

impl<P: Prepared> Display for Query<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Query::Raw(query) => write!(f, "{}", printable_query!(query)),
            Query::Prepared(query) => query.fmt(f),
        }
    }
}

#[derive(Default, Debug)]
pub struct RowsAffected {
    pub rows_affected: u64,
    pub last_insert_id: Option<u64>,
}

pub type RowNames = Arc<[String]>;
pub type Row = Box<[Value]>;

#[derive(Debug)]
pub struct RowLabeled {
    pub labels: RowNames,
    pub values: Row,
}

impl RowLabeled {
    pub fn new(names: RowNames, values: Row) -> Self {
        Self {
            labels: names,
            values,
        }
    }

    pub fn names(&self) -> &[String] {
        &self.labels
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }
}

#[derive(Debug)]
pub enum QueryResult {
    RowLabeled(RowLabeled),
    Affected(RowsAffected),
}

impl Extend<RowsAffected> for RowsAffected {
    fn extend<T: IntoIterator<Item = RowsAffected>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
            if elem.last_insert_id.is_some() {
                self.last_insert_id = elem.last_insert_id;
            }
        }
    }
}

impl From<RowLabeled> for Row {
    fn from(value: RowLabeled) -> Self {
        value.values
    }
}

impl<'a> From<&'a RowLabeled> for &'a Row {
    fn from(value: &'a RowLabeled) -> Self {
        &value.values
    }
}
