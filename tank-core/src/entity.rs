use crate::{Executor, Result};
use tank_metadata::ColumnDef;

pub trait Entity {
    type Column;

    fn table_name() -> &'static str;
    fn columns() -> &'static [ColumnDef];
    fn primary_key() -> &'static [ColumnDef];
    fn create_table<E: Executor>(
        executor: &mut E,
        if_not_exists: bool,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
    fn drop_table<E: Executor>(
        executor: &mut E,
        if_exists: bool,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}
