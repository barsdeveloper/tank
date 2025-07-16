use crate::{
    ColumnDef, Executor, Expression, Result, Row, RowLabeled, RowsAffected, TableRef, Value,
    stream::Stream,
};
use std::future::Future;

pub trait Entity: Send {
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
        It: Iterator<Item = &'a Self>;

    fn find_one<Exec: Executor>(
        executor: &mut Exec,
        primary_key: &Self::PrimaryKey<'_>,
    ) -> impl Future<Output = Result<Option<Self>>> + Send
    where
        Self: Sized;

    fn find_many<Exec: Executor, Expr: Expression>(
        executor: &mut Exec,
        condition: &Expr,
        limit: Option<u32>,
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

    fn row_filtered(&self) -> Box<[(&'static str, Value)]>;
    fn row_full(&self) -> Row;
    fn primary_key(&self) -> Self::PrimaryKey<'_>;
    fn save<Exec: Executor>(&self, executor: &mut Exec) -> impl Future<Output = Result<()>> + Send;
}

// impl<E: Entity> From<E> for RowLabeled {
//     fn from(value: E) -> Self {
//         let cols = E::columns();
//         RowLabeled { labels: cols.iter().map(|c| c.name()), values: () }
//     }
// }
