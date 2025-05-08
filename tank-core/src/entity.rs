use crate::{ColumnDef, Executor, Expression, Result, TableRef};

pub trait Entity: Send {
    type PrimaryKey;

    fn table_name() -> &'static str;
    fn schema_name() -> &'static str;
    fn table_ref() -> &'static TableRef;
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

    fn find_by_key<E: Executor>(
        executor: &mut E,
        primary_key: &Self::PrimaryKey,
    ) -> impl std::future::Future<Output = Result<Self>> + Send
    where
        Self: Sized;

    fn find_by_condition<E: Executor, Expr: Expression>(
        executor: &mut E,
        condition: Expr,
    ) -> impl std::future::Future<Output = Result<Self>> + Send
    where
        Self: Sized;
}
