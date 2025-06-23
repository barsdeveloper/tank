use crate::SqlWriter;

pub trait DataSet {
    /// Must qualify the column names with the table name
    fn qualified_columns() -> bool
    where
        Self: Sized;
    fn sql_write<'a>(&self, writer: &dyn SqlWriter, out: &'a mut String) -> &'a mut String;
}

impl DataSet for &dyn DataSet {
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        todo!()
    }

    fn sql_write<'a>(&self, _writer: &dyn SqlWriter, _out: &'a mut String) -> &'a mut String {
        todo!()
    }
}

impl<T: DataSet> DataSet for &T {
    fn qualified_columns() -> bool {
        <T as DataSet>::qualified_columns()
    }
    fn sql_write<'a>(&self, writer: &dyn SqlWriter, out: &'a mut String) -> &'a mut String {
        <T as DataSet>::sql_write(self, writer, out)
    }
}
