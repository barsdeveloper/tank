use crate::{
    Driver, Executor, Prepared, Result, Value,
    future::FutureExt,
    printable_query,
    stream::{Stream, StreamExt},
};
use std::{
    fmt::{self, Display},
    pin::pin,
    sync::Arc,
};

#[derive(Clone)]
pub enum Query<D: Driver> {
    Raw(String),
    Prepared(D::Prepared),
}

impl<D: Driver> Query<D> {
    pub fn run<'e, Exec: Executor<Driver = D>>(
        self,
        executor: &mut Exec,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        executor.run(self)
    }
    pub fn fetch_one<Exec: Executor<Driver = D>>(
        self,
        executor: &mut Exec,
    ) -> impl Future<Output = Result<Option<RowLabeled>>> + Send {
        let stream = executor.fetch(self);
        async move { pin!(stream).into_future().map(|(v, _)| v).await.transpose() }
    }
    pub fn fetch_many<Exec: Executor<Driver = D>>(
        self,
        executor: &mut Exec,
    ) -> impl Stream<Item = Result<RowLabeled>> + Send {
        executor.fetch(self)
    }
}

impl<D: Driver> From<&str> for Query<D> {
    fn from(value: &str) -> Self {
        Query::Raw(value.into())
    }
}

impl<D: Driver> From<String> for Query<D> {
    fn from(value: String) -> Self {
        Query::Raw(value.into())
    }
}

impl<D, P> From<P> for Query<D>
where
    D: Driver<Prepared = P>,
    P: Prepared,
{
    fn from(value: P) -> Self {
        Query::Prepared(value)
    }
}

impl<D: Driver> Display for Query<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Query::Raw(query) => write!(f, "{}", printable_query!(query)),
            Query::Prepared(query) => query.fmt(f),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RowsAffected {
    pub rows_affected: u64,
    pub last_affected_id: Option<i64>,
}

pub type RowNames = Arc<[String]>;
pub type Row = Box<[Value]>;

#[derive(Debug, Clone)]
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
    pub fn get_column(&self, name: &str) -> Option<&Value> {
        self.labels
            .iter()
            .position(|v| v == name)
            .map(|i| &self.values()[i])
    }
}

#[derive(Debug)]
pub enum QueryResult {
    Row(RowLabeled),
    Affected(RowsAffected),
}

impl Extend<RowsAffected> for RowsAffected {
    fn extend<T: IntoIterator<Item = RowsAffected>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
            if elem.last_affected_id.is_some() {
                self.last_affected_id = elem.last_affected_id;
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

impl From<RowLabeled> for QueryResult {
    fn from(value: RowLabeled) -> Self {
        QueryResult::Row(value)
    }
}

impl From<RowsAffected> for QueryResult {
    fn from(value: RowsAffected) -> Self {
        QueryResult::Affected(value)
    }
}
