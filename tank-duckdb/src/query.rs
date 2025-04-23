use crate::cbox::CBox;
use libduckdb_sys::{duckdb_destroy_prepare, duckdb_prepared_statement};
use std::sync::Arc;
use tank_core::Prepared;

#[derive(Clone)]
pub struct DuckDBPrepared {
    pub(crate) prepared: Arc<CBox<duckdb_prepared_statement>>,
}
impl DuckDBPrepared {
    pub fn new(prepared: duckdb_prepared_statement) -> Self {
        let prepared = Arc::new(CBox::new(prepared, |mut ptr| unsafe {
            duckdb_destroy_prepare(&mut ptr)
        }));
        Self { prepared }
    }
}

impl Prepared for DuckDBPrepared {}
