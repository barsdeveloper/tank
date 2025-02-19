use super::{Fragment, Statement};
use crate::{Backend, WriteSql};
use dyn_clone::clone_box;
use macros::Fragment;
use metadata::TableLike;
use tank::SelectFragment;

mod tank {
    pub use crate::*;
}

#[derive(Fragment, Clone)]
pub struct FromFragment {
    source: Box<dyn TableLike>,
}
impl FromFragment {
    pub fn get_source(&self) -> &dyn TableLike {
        self.source.as_ref()
    }
}

pub trait TransitionFrom {
    fn from<T: TableLike + 'static>(self, columns: &T) -> Statement<FromFragment>;
}

impl<F: Fragment> TransitionFrom for Statement<F>
where
    F: AllowFrom,
{
    fn from<T: TableLike + 'static>(self, source: &T) -> Statement<FromFragment> {
        let source = clone_box(source);
        self.add_fragment(FromFragment { source }.into())
    }
}

impl AllowFrom for SelectFragment {}

impl WriteSql for FromFragment {
    fn write_sql<B: Backend>(&self, backend: &B, output: &mut String) {
        output.push_str("FROM ");
    }
}
