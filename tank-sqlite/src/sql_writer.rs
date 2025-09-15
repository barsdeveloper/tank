use tank_core::{Entity, SqlWriter};

pub struct SqliteSqlWriter {}

impl SqlWriter for SqliteSqlWriter {
    fn as_dyn(&self) -> &dyn SqlWriter {
        self
    }

    fn write_create_schema<E>(&self, _out: &mut String, _if_not_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        // Sqlite does not support schema
    }

    fn write_drop_schema<E>(&self, _out: &mut String, _if_exists: bool)
    where
        Self: Sized,
        E: Entity,
    {
        // Sqlite does not support schema
    }
}
