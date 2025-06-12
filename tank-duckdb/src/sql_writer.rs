use tank_core::SqlWriter;

#[derive(Default)]
pub struct DuckDBSqlWriter {}

impl DuckDBSqlWriter {
    pub const fn new() -> Self {
        Self {}
    }
}

impl SqlWriter for DuckDBSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }
}
