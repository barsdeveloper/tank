use tank_core::SqlWriter;

#[derive(Default)]
pub struct YourDBSqlWriter {}

impl SqlWriter for YourDBSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }
}
