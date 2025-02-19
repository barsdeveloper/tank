use super::{Fragment, Statement};
use crate::{Backend, WriteSql};
use macros::Fragment;
use metadata::ColumnRef;
use tank::InitialFragment;

mod tank {
    pub use crate::*;
}

#[derive(Default, Fragment, Clone)]
pub struct SelectFragment {
    columns: Vec<ColumnRef>,
}
impl SelectFragment {
    pub fn get_columns(&self) -> &[ColumnRef] {
        self.columns.as_slice()
    }
}

pub trait TransitionSelect {
    fn select(self, columns: Vec<ColumnRef>) -> Statement<SelectFragment>;
}

impl<F: Fragment> TransitionSelect for Statement<F>
where
    F: AllowSelect,
{
    fn select(self, columns: Vec<ColumnRef>) -> Statement<SelectFragment> {
        self.add_fragment(SelectFragment { columns }.into())
    }
}

impl AllowSelect for InitialFragment {}

impl WriteSql for SelectFragment {
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
