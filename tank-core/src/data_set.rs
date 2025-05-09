use crate::SqlWriter;

pub trait DataSet {
    /// Must qualify the column names with the table name
    const QUALIFIED_COLUMNS: bool;

    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
    ) -> &'a mut String;
}

impl<T: DataSet> DataSet for &T {
    const QUALIFIED_COLUMNS: bool = <T as DataSet>::QUALIFIED_COLUMNS;
    fn sql_write<'a, W: SqlWriter + ?Sized>(
        &self,
        writer: &W,
        out: &'a mut String,
    ) -> &'a mut String {
        <T as DataSet>::sql_write(self, writer, out)
    }
}
