#[cfg(test)]
mod tests {

    use tank::Entity;
    use tank::SqlWriter;
    use tank::Value;
    use tank_duckdb::DuckDBDriver;

    const DRIVER: DuckDBDriver = DuckDBDriver::new();

    #[test]
    fn test_1() {
        #[derive(Entity)]
        struct SomeEntity {
            a: i8,
            b: String,
        }
        let columns = SomeEntity::columns();

        assert_eq!(SomeEntity::table_name(), "some_entity");
        assert_eq!(columns[0].name, "a");
        assert!(matches!(columns[0].value, Value::Int8(None)));
        assert!(columns[0].nullable == false);
        assert_eq!(columns[1].name, "b");
        assert!(matches!(columns[1].value, Value::Varchar(None)));
        assert!(columns[1].nullable == false);
        assert_eq!(
            SomeEntity::sql_create_table(&DRIVER, false),
            "CREATE TABLE some_entity(a TINYINT NOT NULL, b VARCHAR NOT NULL)"
        );
    }

    #[test]
    fn test_2() {
        #[derive(Entity)]
        #[table_name("custom_table_name")]
        struct SomeEntity {
            first: u128,
            second: Option<time::Time>,
            third: Box<Option<Box<time::Date>>>,
        }
        let columns = SomeEntity::columns();

        assert_eq!(SomeEntity::table_name(), "custom_table_name");
        assert_eq!(columns[0].name, "first");
        assert!(matches!(columns[0].value, Value::UInt128(None)));
        assert!(columns[0].nullable == false);
        assert_eq!(columns[1].name, "second");
        assert!(matches!(columns[1].value, Value::Time(None)));
        assert!(columns[1].nullable == true);
        assert_eq!(columns[2].name, "third");
        assert!(matches!(columns[2].value, Value::Date(None)));
        assert!(columns[2].nullable == true);
        assert_eq!(
            SomeEntity::sql_create_table(&DRIVER, true),
            "CREATE TABLE IF NOT EXISTS custom_table_name(first UHUGEINT NOT NULL, second TIME, third DATE)"
        );
    }
}
