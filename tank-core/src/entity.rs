use crate::{ColumnDef, Executor, Expression, Result, Row, RowLabeled, TableRef};
use std::future::Future;

pub trait Entity: Send {
    type PrimaryKey;

    fn table_name() -> &'static str;
    fn schema_name() -> &'static str;
    fn table_ref() -> &'static TableRef;
    fn columns() -> &'static [ColumnDef];
    fn primary_key() -> &'static [ColumnDef];

    fn create_table<Exec: Executor>(
        executor: &mut Exec,
        if_not_exists: bool,
    ) -> impl Future<Output = Result<()>> + Send;

    fn drop_table<Exec: Executor>(
        executor: &mut Exec,
        if_exists: bool,
    ) -> impl Future<Output = Result<()>> + Send;

    fn find_by_key<Exec: Executor>(
        executor: &mut Exec,
        primary_key: &Self::PrimaryKey,
    ) -> impl Future<Output = Result<Self>> + Send
    where
        Self: Sized;

    fn find_by_condition<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: Expr,
    ) -> impl Future<Output = Result<Self>> + Send
    where
        Self: Sized;

    fn row(&self) -> Row;

    fn row_labeled(&self) -> RowLabeled;

    fn save<Exec: Executor>(&self, executor: &mut Exec) -> impl Future<Output = Result<()>> + Send;
}
