use crate::{ColumnDef, Executor, Expression, Result, Row, RowLabeled, RowsAffected, TableRef};
use futures::Stream;
use std::future::Future;

pub trait Entity: Send {
    type PrimaryKey<'a>;

    fn table_name() -> &'static str;
    fn schema_name() -> &'static str;
    fn table_ref() -> &'static TableRef;
    fn columns_def() -> &'static [ColumnDef];
    fn primary_key_def() -> &'static [&'static ColumnDef];

    fn from_row(row: RowLabeled) -> Result<Self>
    where
        Self: Sized;

    fn create_table<Exec: Executor>(
        executor: &mut Exec,
        if_not_exists: bool,
    ) -> impl Future<Output = Result<()>> + Send;

    fn drop_table<Exec: Executor>(
        executor: &mut Exec,
        if_exists: bool,
    ) -> impl Future<Output = Result<()>> + Send;

    fn find_one<Exec: Executor>(
        executor: &mut Exec,
        primary_key: &Self::PrimaryKey<'_>,
    ) -> impl Future<Output = Result<Option<Self>>> + Send
    where
        Self: Sized;

    fn find_many<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
    ) -> impl Stream<Item = Result<Self>> + Send
    where
        Self: Sized;

    fn delete_one<Exec: Executor>(
        executor: &mut Exec,
        primary_key: &Self::PrimaryKey<'_>,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        Self: Sized;

    fn delete_many<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        Self: Sized;

    fn row(&self) -> Row;
    fn row_labeled(&self) -> RowLabeled;
    fn primary_key(&self) -> Self::PrimaryKey<'_>;
    fn save<Exec: Executor>(&self, executor: &mut Exec) -> impl Future<Output = Result<()>> + Send;
}
