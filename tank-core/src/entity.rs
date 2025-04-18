use crate::{ColumnDef, Executor, Result};

pub trait Entity {
    type Column;
    type PrimaryKey;

    fn schema_name() -> &'static str;
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

    fn find_by_pk<E: Executor>(
        executor: &mut E,
        primary_key: &Self::PrimaryKey,
    ) -> impl std::future::Future<Output = Result<Self>> + Send
    where
        Self: Sized;
}
