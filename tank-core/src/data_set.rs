use crate::{
    Driver, Executor, Expression, Query, Result, RowLabeled,
    stream::Stream,
    writer::{Context, SqlWriter},
};

pub trait DataSet {
    /// Must qualify the column names with the table name
    fn qualified_columns() -> bool
    where
        Self: Sized;
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, buff: &mut String);
    fn select<'s, Exec, Item, Cols, Expr>(
        &'s self,
        executor: &'s mut Exec,
        columns: Cols,
        condition: &Expr,
        limit: Option<u32>,
    ) -> impl Stream<Item = Result<RowLabeled>> + 's
    where
        Self: Sized,
        Exec: Executor,
        Item: Expression,
        Cols: IntoIterator<Item = Item> + Clone,
        Expr: Expression,
    {
        let mut query = String::with_capacity(1024);
        executor
            .driver()
            .sql_writer()
            .write_select(&mut query, columns, self, condition, limit);
        executor.fetch(query.into())
    }
    fn prepare<Item, Cols, Exec, Expr>(
        &self,
        columns: Cols,
        executor: &mut Exec,
        condition: &Expr,
        limit: Option<u32>,
    ) -> impl Future<Output = Result<Query<Exec::Driver>>>
    where
        Self: Sized,
        Item: Expression,
        Cols: IntoIterator<Item = Item> + Clone,
        Exec: Executor,
        Expr: Expression,
    {
        let mut query = String::with_capacity(1024);
        executor
            .driver()
            .sql_writer()
            .write_select(&mut query, columns, self, condition, limit);
        executor.prepare(query.into())
    }
}

impl DataSet for &dyn DataSet {
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        unreachable!("Cannot call static qualified_columns on a dyn object directly");
    }
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, buff: &mut String) {
        (*self).write_query(writer, context, buff)
    }
}
