use crate::{Driver, Executor, Expression, Query, Result, RowLabeled, SqlWriter, stream::Stream};

pub trait DataSet {
    /// Must qualify the column names with the table name
    fn qualified_columns() -> bool
    where
        Self: Sized;
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String);
    fn select<Item, Cols, Exec, Expr>(
        &self,
        columns: Cols,
        executor: &mut Exec,
        condition: &Expr,
        limit: Option<u32>,
    ) -> impl Stream<Item = Result<RowLabeled>>
    where
        Self: Sized,
        Item: Expression,
        Cols: IntoIterator<Item = Item>,
        Exec: Executor,
        Expr: Expression,
    {
        let mut query = String::with_capacity(1024);
        executor
            .driver()
            .sql_writer()
            .write_select(&mut query, columns, self, condition, limit);
        executor.fetch(Query::Raw(query.into()))
    }
}

impl DataSet for &dyn DataSet {
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        unreachable!("Cannot call static qualified_columns on a dyn object directly");
    }
    fn write_query(&self, _writer: &dyn SqlWriter, _out: &mut String) {
        todo!()
    }
}

impl<T: DataSet> DataSet for &T {
    fn qualified_columns() -> bool {
        <T as DataSet>::qualified_columns()
    }
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String) {
        <T as DataSet>::write_query(self, writer, out);
    }
    fn select<Item, Cols, Exec: Executor, Expr: Expression>(
        &self,
        columns: Cols,
        executor: &mut Exec,
        condition: &Expr,
        limit: Option<u32>,
    ) -> impl Stream<Item = Result<RowLabeled>>
    where
        Self: Sized,
        Item: Expression,
        Cols: IntoIterator<Item = Item>,
        Exec: Executor,
        Expr: Expression,
    {
        (*self).select(columns, executor, condition, limit)
    }
}
