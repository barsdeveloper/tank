use crate::cbox::CBox;
use libduckdb_sys::duckdb_prepared_statement;
use std::sync::Arc;
use tank_core::Prepared;

#[derive(Clone)]
pub struct DuckDBPrepared {
    pub(crate) prepared: Arc<CBox<duckdb_prepared_statement>>,
}
impl DuckDBPrepared {
    pub(crate) fn new(prepared: CBox<duckdb_prepared_statement>) -> Self {
        Self {
            prepared: Arc::new(prepared),
        }
    }
}

impl Prepared for DuckDBPrepared {}
