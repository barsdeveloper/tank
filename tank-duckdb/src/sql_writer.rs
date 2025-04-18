use tank_metadata::SqlWriter;

#[derive(Default)]
pub struct DuckDBSqlWriter {}

impl SqlWriter for DuckDBSqlWriter {}
