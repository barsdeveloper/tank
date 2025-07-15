#[cfg(test)]
mod tests {
    use tank::{ColumnRef, Entity};

    #[test]
    fn test_column_conversions() {
        #[derive(Entity)]
        #[tank(schema = "the_schema", name = "my_table")]
        struct Entity {
            #[tank(name = "solo_column")]
            col: i32,
        }

        let column = &Entity::columns()[0];
        assert_eq!(column.table(), "my_table");
        assert_eq!(column.schema(), "the_schema");
        assert_eq!(column.name(), "solo_column");

        let col_ref: &ColumnRef = column.into();
        assert_eq!(col_ref.table, "my_table");
        assert_eq!(col_ref.schema, "the_schema");
        assert_eq!(col_ref.name, "solo_column");
    }
}
