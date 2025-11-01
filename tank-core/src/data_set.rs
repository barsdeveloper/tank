use crate::{
    Driver, Executor, Expression, Query, Result, RowLabeled,
    stream::Stream,
    writer::{Context, SqlWriter},
};

/// A selectable data source (table, join tree).
///
/// Implementors know how to render themselves inside a FROM clause and whether
/// column references should be qualified with schema/table.
///
/// Provided helpers construct SELECT statements for convenience.
pub trait DataSet {
    /// Whether columns should be qualified (`schema.table.column`).
    fn qualified_columns() -> bool
    where
        Self: Sized;
    /// Emit the textual representation into `out` using the given writer.
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, out: &mut String);
    /// Execute a SELECT, streaming labeled rows.
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
    /// Prepare (but do not yet run) a SQL statement.
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
    fn write_query(&self, writer: &dyn SqlWriter, context: &mut Context, out: &mut String) {
        (*self).write_query(writer, context, out)
    }
}
