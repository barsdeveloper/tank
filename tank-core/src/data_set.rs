use crate::{Driver, Executor, Expression, Result, RowLabeled, SqlWriter, stream::Stream};

pub trait DataSet {
    /// Must qualify the column names with the table name
    fn qualified_columns() -> bool
    where
        Self: Sized;
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String);
    fn select<'a, C, Exec, Expr>(
        &self,
        columns: C,
        executor: &mut Exec,
        condition: &Expr,
        limit: Option<u32>,
    ) -> impl Stream<Item = Result<RowLabeled>>
    where
        Self: Sized,
        C: Iterator<Item = &'a dyn Expression>,
        Exec: Executor,
        Expr: Expression,
    {
        let mut query = String::with_capacity(1024);
        executor
            .driver()
            .sql_writer()
            .write_select(&mut query, columns, self, condition, limit);
        executor.fetch(query.into())
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
    fn select<'a, C, Exec: Executor, Expr: Expression>(
        &self,
        columns: C,
        executor: &mut Exec,
        condition: &Expr,
        limit: Option<u32>,
    ) -> impl Stream<Item = Result<RowLabeled>>
    where
        Self: Sized,
        C: Iterator<Item = &'a dyn Expression>,
    {
        (*self).select(columns, executor, condition, limit)
    }
}
