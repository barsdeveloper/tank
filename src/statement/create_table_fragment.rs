use super::{Fragment, Statement};
use crate::{Backend, WriteSql};
use macros::Fragment;
use metadata::ColumnDef;

mod tank {
    pub use crate::*;
}

#[derive(Default, Fragment, Clone)]
pub struct CreateTableFragment {
    table_name: String,
    if_not_exists: bool,
    columns: Vec<ColumnDef>,
    temporary: bool,
}
impl CreateTableFragment {
    pub fn get_table_name(&self) -> &str {
        &self.table_name
    }
    pub fn get_columns(&self) -> &[ColumnDef] {
        self.columns.as_slice()
    }
    pub fn get_if_not_exists(&self) -> bool {
        self.if_not_exists
    }
}

pub trait TransitionCreateTable {
    fn create_table(self, table_name: String) -> Statement<CreateTableFragment>;
}

impl<F: Fragment> TransitionCreateTable for Statement<F>
where
    F: AllowCreateTable,
{
    fn create_table(self, table_name: String) -> Statement<CreateTableFragment> {
        self.add_fragment(
            CreateTableFragment {
                table_name,
                ..Default::default()
            }
            .into(),
        )
    }
}

pub trait CreateTableDefinition {
    fn temporary(self) -> Statement<CreateTableFragment>;
    fn columns(self, columns: Vec<ColumnDef>) -> Statement<CreateTableFragment>;
}

impl CreateTableDefinition for Statement<CreateTableFragment> {
    fn temporary(mut self) -> Statement<CreateTableFragment> {
        self.get_current_fragment().temporary = true;
        self
    }
    fn columns(mut self, columns: Vec<ColumnDef>) -> Statement<CreateTableFragment> {
        self.get_current_fragment().columns = columns;
        self
    }
}

impl WriteSql for CreateTableFragment {
    fn write_sql<B: Backend>(&self, backend: &B, output: &mut String) {
        output.push_str("CREATE ");
        if self.temporary {
            output.push_str("TEMPORARY ");
        }
        output.push_str("TABLE ");
        if self.if_not_exists {
            output.push_str("IF NOT EXISTS ");
        }
        let mut has_previous = false;
        self.get_columns().iter().for_each(|v| {
            if has_previous {
                output.push_str(",");
            }
            v.write_sql(backend, output);
            has_previous = true
        });
    }
}
