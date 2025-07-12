use crate::Value;
use std::sync::Arc;

pub trait Prepared: Clone + Send + Sync {}

#[derive(Clone)]
pub enum Query<P: Prepared> {
    Raw(Arc<str>),
    Prepared(P),
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
