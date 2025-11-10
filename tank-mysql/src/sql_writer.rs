use tank_core::SqlWriter;

#[derive(Default)]
pub struct MySQLSqlWriter {}

impl SqlWriter for MySQLSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }
}
