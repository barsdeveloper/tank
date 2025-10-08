use tank_core::SqlWriter;

pub struct PostgresSqlWriter {}

impl SqlWriter for PostgresSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }
}
