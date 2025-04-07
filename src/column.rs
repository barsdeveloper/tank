use metadata::ColumnDef;

pub trait ColumnTrait {
    fn def(&self) -> ColumnDef;
}
