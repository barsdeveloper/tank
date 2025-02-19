use super::{Fragment, Statement};
use crate::{Backend, WriteSql};
use macros::Fragment;
use metadata::ColumnRef;

mod tank {
    pub use crate::*;
}

#[derive(Default, Fragment)]
pub struct WhereFragment {
    columns: Vec<ColumnRef>,
}
impl WhereFragment {
    pub fn get_columns(&self) -> &[ColumnRef] {
        self.columns.as_slice()
    }
}

pub trait TransitionWhere {
    fn select(self, columns: Vec<ColumnRef>) -> Statement<WhereFragment>;
}

impl<F: Fragment> TransitionWhere for Statement<F>
where
    F: AllowWhere,
{
    fn select(self, columns: Vec<ColumnRef>) -> Statement<WhereFragment> {
        self.add_fragment(WhereFragment { columns }.into())
    }
}

impl WriteSql for WhereFragment {
    fn write_sql<B: Backend>(&self, backend: &B, output: &mut String) {
        output.push_str("SELECT ");
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
