use metadata::ColumnDef;

pub trait Entity {
    type Column;

    fn table_name() -> &'static str;

    fn columns() -> &'static [ColumnDef];

    fn sql_create_table(if_not_exists: bool) -> String
    where
        Self: Sized;

    // fn sql_drop_table(if_exists: bool) -> Drop
    // where
    //     Self: Sized;

    // fn primary_key(&self) -> Vec<ColumnDef>;
}
