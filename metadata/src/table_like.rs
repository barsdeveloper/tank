use dyn_clone::DynClone;

use crate::ColumnDef;

pub trait TableLike: DynClone {
    fn columns(&self) -> &[ColumnDef];
}

dyn_clone::clone_trait_object!(TableLike);
