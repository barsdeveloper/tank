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
pub type RowValues = Box<[Value]>;

pub struct Row {
    names: RowNames,
    values: RowValues,
}

impl Row {
    pub fn new(names: RowNames, values: RowValues) -> Self {
        Self { names, values }
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }
}

pub enum QueryResult {
    Row(Row),
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
