use tank_core::SqlWriter;

pub struct SqliteSqlWriter {}

impl SqlWriter for SqliteSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }
}
