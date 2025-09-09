use crate::{
    ColumnDef, Driver, Error, Executor, Expression, Result, Row, RowLabeled, RowsAffected,
    SqlWriter, TableRef, Value, stream::Stream,
};
use futures::{FutureExt, StreamExt, TryFutureExt};
use log::Level;
use std::{future::Future, pin::pin};

pub trait Entity {
    type PrimaryKey<'a>;

    fn table_ref() -> &'static TableRef;
    fn columns() -> &'static [ColumnDef];
    fn primary_key_def() -> impl ExactSizeIterator<Item = &'static ColumnDef>;
    fn unique_defs()
    -> impl ExactSizeIterator<Item = impl ExactSizeIterator<Item = &'static ColumnDef>>;

    fn from_row(row: RowLabeled) -> Result<Self>
    where
        Self: Sized;

    fn create_table<Exec: Executor>(
        executor: &mut Exec,
        if_not_exists: bool,
        create_schema: bool,
    ) -> impl Future<Output = Result<()>> + Send;

    fn drop_table<Exec: Executor>(
        executor: &mut Exec,
        if_exists: bool,
        drop_schema: bool,
    ) -> impl Future<Output = Result<()>> + Send;

    fn insert_one<Exec: Executor, E: Entity>(
        executor: &mut Exec,
        entity: &E,
    ) -> impl Future<Output = Result<RowsAffected>> + Send;

    fn insert_many<'a, Exec, It>(
        executor: &mut Exec,
        items: It,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        Self: 'a,
        Exec: Executor,
        It: IntoIterator<Item = &'a Self>;

    fn find_pk<Exec: Executor>(
        executor: &mut Exec,
        primary_key: &Self::PrimaryKey<'_>,
    ) -> impl Future<Output = Result<Option<Self>>> + Send
    where
        Self: Sized;

    fn find_one<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
    ) -> impl Future<Output = Result<Option<Self>>> + Send
    where
        Self: Sized,
    {
        let stream = Self::find_many(executor, condition, Some(1));
        async move { pin!(stream).into_future().map(|(v, _)| v).await.transpose() }
    }

    fn find_many<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
        limit: Option<u32>,
    ) -> impl Stream<Item = Result<Self>> + Send
    where
        Self: Sized;

    fn delete_one<Exec: Executor>(
        executor: &mut Exec,
        primary_key: Self::PrimaryKey<'_>,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        Self: Sized;

    fn delete_many<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
    ) -> impl Future<Output = Result<RowsAffected>> + Send
    where
        Self: Sized;

    fn row_filtered(&self) -> Box<[(&'static str, Value)]>;
    fn row_full(&self) -> Row;
    fn primary_key(&self) -> Self::PrimaryKey<'_>;
    fn save<Exec: Executor>(&self, executor: &mut Exec) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        let mut query = String::with_capacity(512);
        executor
            .driver()
            .sql_writer()
            .write_insert(&mut query, [self], true);
        executor.execute(query.into()).map_ok(|_| ())
    }
    fn delete<Exec: Executor>(&self, executor: &mut Exec) -> impl Future<Output = Result<()>> + Send
    where
        Self: Sized,
    {
        Self::delete_one(executor, self.primary_key()).map(|v| {
            v.and_then(|v| {
                if v.rows_affected == 1 {
                    Ok(())
                } else {
                    let error = Error::msg(format!(
                        "The query deleted {} rows instead of the expected 1",
                        v.rows_affected
                    ));
                    log::log!(
                        if v.rows_affected == 0 {
                            Level::Info
                        } else {
                            Level::Error
                        },
                        "{}",
                        error
                    );
                    Err(error)
                }
            })
        })
    }
}
