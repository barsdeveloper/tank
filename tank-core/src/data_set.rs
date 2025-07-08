use crate::SqlWriter;

pub trait DataSet {
    /// Must qualify the column names with the table name
    fn qualified_columns() -> bool
    where
        Self: Sized;
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String);
}

impl DataSet for &dyn DataSet {
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        todo!()
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
}
