use tank_metadata::ColumnDef;

pub trait ColumnTrait {
    fn def(&self) -> ColumnDef;
}
