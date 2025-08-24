use crate::{AsValue, Driver, Executor, Result, Value, stream::Stream};
use futures::{FutureExt, StreamExt};
use std::{
    fmt::{self, Display},
    mem,
    pin::pin,
    sync::Arc,
};

pub trait Prepared: Default + Clone + Send + Sync {
    // fn bind<V: AsValue>(&mut self, v: V) -> Result<Self>;
    // fn run<'e, Exec: Executor<Driver = impl Driver<Prepared = Self>>>(
    //     &mut self,
    //     executor: &'e mut Exec,
    // ) -> impl Stream<Item = Result<QueryResult>> + Send + 'e
    // where
    //     Self: 'e + AsMut<Query<<Exec::Driver as Driver>::Prepared>>,
    // {
    //     let mut query = Query::Prepared(mem::take(self));
    //     let stream = executor.run(query.as_mut());
    //     let Query::Prepared(query) = query else {
    //         unreachable!("It must be a prepared query as it was initially");
    //     };
    //     *self = query;
    //     stream
    // }
    // fn fetch_one<Exec: Executor>(
    //     &mut self,
    //     executor: &mut Exec,
    // ) -> impl Future<Output = Result<Option<RowLabeled>>> + Send {
    //     let stream = executor.fetch(self);
    //     async move { pin!(stream).into_future().map(|(v, _)| v).await.transpose() }
    // }
    // fn fetch_many<Exec: Executor>(
    //     &mut self,
    //     executor: &mut Exec,
    // ) -> impl Stream<Item = Result<RowLabeled>> + Send {
    //     executor.fetch(self)
    // }
}

pub enum Query<P: Prepared> {
    Raw(Arc<str>),
    Prepared(P),
}

impl<P: Prepared> Default for Query<P> {
    fn default() -> Self {
        Query::Raw("".into())
    }
}

impl<P: Prepared> AsMut<Query<P>> for Query<P> {
    fn as_mut(&mut self) -> &mut Query<P> {
        self
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
            Query::Raw(query) => f.write_str(query),
            Query::Prepared(_) => todo!(),
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
