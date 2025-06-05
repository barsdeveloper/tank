use crate::Value;
use std::sync::Arc;

pub trait Prepared: Clone {}

#[derive(Clone)]
pub enum Query<P: Prepared> {
    Raw(Arc<str>),
    Prepared(P),
}

#[derive(Default)]
pub struct Count {
    pub rows_affected: u64,
    pub last_insert_id: Option<u64>,
}

pub type RowNames = Arc<[String]>;
pub type Row = Box<[Value]>;

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

pub enum QueryResult {
    RowLabeled(RowLabeled),
    Count(Count),
}

impl Extend<Count> for Count {
    fn extend<T: IntoIterator<Item = Count>>(&mut self, iter: T) {
        for elem in iter {
            self.rows_affected += elem.rows_affected;
            if elem.last_insert_id.is_some() {
                self.last_insert_id = elem.last_insert_id;
            }
        }
    }
}
