#[cfg(test)]
mod tests {

    use std::sync::Arc;
    use std::time::Duration;
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

    #[test]
    fn test_3() {
        #[derive(Entity)]
        #[table_name("a_table")]
        struct MyEntity {
            alpha: Box<Box<Box<Box<Box<Box<Box<Box<Box<Box<Box<f64>>>>>>>>>>>,
            bravo: i16,
            charlie: Box<Box<Option<Option<Box<Box<Option<Box<rust_decimal::Decimal>>>>>>>>,
            delta: Duration,
            #[column_type = "DECIMAL(8, 2)"]
            echo: Option<Arc<rust_decimal::Decimal>>,
        }
        let columns = MyEntity::columns();

        assert_eq!(MyEntity::table_name(), "a_table");

        assert_eq!(columns[0].name, "alpha");
        assert!(matches!(columns[0].value, Value::Float64(None)));
        assert!(columns[0].nullable == false);

        assert_eq!(columns[1].name, "bravo");
        assert!(matches!(columns[1].value, Value::Int16(None)));
        assert!(columns[1].nullable == false);

        assert_eq!(columns[2].name, "charlie");
        assert!(matches!(columns[2].value, Value::Decimal(None, 0, 0)));
        assert!(columns[2].nullable == true);

        assert_eq!(columns[3].name, "delta");
        assert!(matches!(columns[3].value, Value::Interval(None)));
        assert!(columns[3].nullable == false);

        assert_eq!(columns[4].name, "echo");
        assert!(matches!(columns[4].value, Value::Decimal(None, 0, 0)));
        assert!(columns[4].nullable == true);

        assert_eq!(
            MyEntity::sql_create_table(&DRIVER, false),
            "CREATE TABLE a_table(alpha DOUBLE NOT NULL, bravo SMALLINT NOT NULL, charlie DECIMAL, delta INTERVAL NOT NULL, echo DECIMAL(8, 2))"
        );
    }
}
