use std::sync::Arc;
use tank_metadata::Value;

pub trait PreparedQuery {}

pub enum Query {
    Raw(String),
    Prepared(Box<dyn PreparedQuery>),
}

#[derive(Default)]
pub struct Count {
    rows_affected: u64,
    last_insert_id: Option<u64>,
}

pub struct Row {
    names: Arc<[String]>,
    fields: Box<[Value]>,
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
