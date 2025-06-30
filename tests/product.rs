#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rust_decimal::Decimal;
    use std::borrow::Cow;
    use tank::{Entity, GenericSqlWriter, Passive, PrimaryKeyType, SqlWriter, TableRef, Value};
    use time::{Date, Month, PrimitiveDateTime, Time};

    #[derive(Entity)]
    #[tank(name = "products")]
    struct Product {
        #[tank(primary_key, auto_increment)]
        id: Passive<u64>,
        name: String,
        price: Decimal,
        available: bool,
        tags: Vec<String>,
        added_on: PrimitiveDateTime,
    }
    impl Product {
        pub fn sample() -> Self {
            Self {
                id: Passive::NotSet,
                name: "Smartphone".into(),
                price: Decimal::new(49999, 2),
                available: true,
                tags: vec!["electronics".into(), "mobile".into()],
                added_on: PrimitiveDateTime::new(
                    Date::from_calendar_date(2025, Month::June, 24).unwrap(),
                    Time::from_hms(10, 30, 07).unwrap(),
                ),
            }
        }
    }
    const WRITER: GenericSqlWriter = GenericSqlWriter {};

    #[tokio::test]
    async fn test_product() {
        assert!(matches!(
            Product::table_ref(),
            TableRef {
                name: "products",
                schema: "",
                alias: Cow::Borrowed(""),
            }
        ));

        let pk = Product::primary_key_def();
        assert_eq!(pk.len(), 1);
        assert!(pk[0].auto_increment);
        assert!(!pk[0].nullable);
        assert_eq!(pk[0].primary_key, PrimaryKeyType::PrimaryKey);

        let columns = Product::columns_def();
        assert_eq!(columns.len(), 6);
        assert!(matches!(columns[0].value, Value::UInt64(None, ..)));
        assert!(matches!(columns[1].value, Value::Varchar(None, ..)));
        assert!(matches!(columns[2].value, Value::Decimal(None, ..)));
        assert!(matches!(columns[3].value, Value::Boolean(None, ..)));
        assert!(matches!(columns[4].value, Value::List(None, ..)));
        assert!(matches!(columns[5].value, Value::Timestamp(None, ..)));
    }

    #[tokio::test]
    async fn test_product_create_table() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_create_table::<Product>(&mut query, false),
            indoc! {"
                CREATE TABLE products (
                id UBIGINT AUTOINCREMENT PRIMARY KEY,
                name VARCHAR NOT NULL,
                price DECIMAL NOT NULL,
                available BOOLEAN NOT NULL,
                tags VARCHAR[] NOT NULL,
                added_on TIMESTAMP NOT NULL
                )
            "}
            .trim()
        );
    }

    #[tokio::test]
    async fn test_product_insert() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_insert::<Product>(&mut query, &Product::sample(), false),
            indoc! {"
                INSERT INTO products (name, price, available, tags, added_on)
                VALUES ('Smartphone', 499.99, true, ['electronics','mobile'], '2025-06-24 10:30:07.0')
            "}
            .trim()
        );
    }
}
