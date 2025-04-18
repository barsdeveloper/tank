use crate::ColumnDef;

pub trait ColumnTrait {
    fn def(&self) -> ColumnDef;
}
