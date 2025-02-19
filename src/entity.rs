use metadata::ColumnDef;

use crate::{InitialFragment, Statement};

pub trait Entity {
    type Table;
    type Column;

    fn table_name() -> &'static str;

    fn sql_create_table(if_not_exists: bool) -> Statement<InitialFragment>
    // TODO
    where
        Self: Sized;

    // fn sql_drop_table(if_exists: bool) -> Drop
    // where
    //     Self: Sized;

    // fn primary_key(&self) -> Vec<ColumnDef>;
}
