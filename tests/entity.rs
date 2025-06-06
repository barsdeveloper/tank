mod resource {
    pub mod trade;
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use std::{borrow::Cow, sync::Arc, time::Duration};
    use tank::{ColumnDef, ColumnRef, Entity, SqlWriter, Value};

    use crate::resource::trade::TradeExecution;

    struct Writer;
    impl SqlWriter for Writer {}

    const WRITER: Writer = Writer {};

    #[tokio::test]
    async fn test_1() {
        #[derive(Entity)]
        struct SomeEntity {
            a: i8,
            b: String,
        }
        let columns = SomeEntity::columns();

        assert_eq!(SomeEntity::table_name(), "some_entity");
        assert_eq!(SomeEntity::primary_key().len(), 0);

        assert_eq!(columns[0].name(), "a");
        assert!(matches!(columns[0].value, Value::Int8(None, ..)));
        assert!(columns[0].nullable == false);

        assert_eq!(columns[1].name(), "b");
        assert!(matches!(columns[1].value, Value::Varchar(None, ..)));
        assert!(columns[1].nullable == false);

        let mut query = String::new();
        assert_eq!(
            WRITER.sql_create_table::<SomeEntity>(&mut query, false),
            indoc! {"
                CREATE TABLE some_entity(
                a TINYINT NOT NULL,
                b VARCHAR NOT NULL
                )
            "}
            .trim()
        );
    }

    #[tokio::test]
    async fn test_2() {
        #[derive(Entity)]
        #[table_name("custom_table_name")]
        struct SomeEntity {
            #[primary_key]
            first: u128,
            second: Option<time::Time>,
            third: Box<Option<Box<time::Date>>>,
        }
        let columns = SomeEntity::columns();

        assert_eq!(SomeEntity::table_name(), "custom_table_name");
        assert_eq!(SomeEntity::primary_key().len(), 1);
        assert_eq!(SomeEntity::primary_key()[0].name(), "first");

        assert_eq!(columns[0].name(), "first");
        assert!(matches!(columns[0].value, Value::UInt128(None, ..)));
        assert!(columns[0].nullable == false);

        assert_eq!(columns[1].name(), "second");
        assert!(matches!(columns[1].value, Value::Time(None, ..)));
        assert!(columns[1].nullable == true);

        assert_eq!(columns[2].name(), "third");
        assert!(matches!(columns[2].value, Value::Date(None, ..)));
        assert!(columns[2].nullable == true);

        let mut query = String::new();
        assert_eq!(
            WRITER.sql_create_table::<SomeEntity>(&mut query, true),
            indoc! {"
                CREATE TABLE IF NOT EXISTS custom_table_name(
                first UHUGEINT PRIMARY KEY,
                second TIME,
                third DATE
                )
            "}
            .trim()
        );
    }

    #[test]
    fn test_3() {
        #[derive(Entity)]
        #[table_name("a_table")]
        #[primary_key("bravo", "delta")]
        struct MyEntity {
            _alpha: Box<Box<Box<Box<Box<Box<Box<Box<Box<Box<Box<f64>>>>>>>>>>>,
            _bravo: i16,
            _charlie: Box<Box<Option<Option<Box<Box<Option<Box<rust_decimal::Decimal>>>>>>>>,
            _delta: Duration,
            #[column_type("DECIMAL(8, 2)")]
            _echo: Option<Arc<rust_decimal::Decimal>>,
        }
        let columns = MyEntity::columns();

        assert_eq!(MyEntity::table_name(), "a_table");
        assert_eq!(
            MyEntity::primary_key()
                .iter()
                .map(|k| k.name().clone())
                .collect::<Vec<_>>(),
            ["bravo", "delta"]
        );

        assert!(matches!(
            columns[0],
            ColumnDef {
                reference: ColumnRef {
                    name: Cow::Borrowed("alpha"),
                    table: Cow::Borrowed("a_table"),
                    schema: Cow::Borrowed(""),
                },
                column_type: Cow::Borrowed(""),
                value: Value::Float64(None, ..),
                nullable: false,
                default: None,
                primary_key: false,
                unique: false,
                auto_increment: false,
                passive: false,
            }
        ));

        assert!(matches!(
            columns[1],
            ColumnDef {
                reference: ColumnRef {
                    name: Cow::Borrowed("bravo"),
                    table: Cow::Borrowed("a_table"),
                    schema: Cow::Borrowed(""),
                },
                column_type: Cow::Borrowed(""),
                value: Value::Int16(None, ..),
                nullable: false,
                default: None,
                primary_key: false,
                // unique: false,
                // auto_increment: false,
                ..
            }
        ));

        assert!(matches!(
            columns[2],
            ColumnDef {
                reference: ColumnRef {
                    name: Cow::Borrowed("charlie"),
                    table: Cow::Borrowed("a_table"),
                    schema: Cow::Borrowed(""),
                },
                column_type: Cow::Borrowed(""),
                value: Value::Decimal(None, ..),
                nullable: true,
                default: None,
                primary_key: false,
                unique: false,
                auto_increment: false,
                passive: false,
            }
        ));

        assert!(matches!(
            columns[3],
            ColumnDef {
                reference: ColumnRef {
                    name: Cow::Borrowed("delta"),
                    table: Cow::Borrowed("a_table"),
                    schema: Cow::Borrowed(""),
                },
                // column_type: Cow::Borrowed(""),
                // value: Value::Interval(None, ..),
                // nullable: false,
                // default: None,
                // primary_key: false,
                // unique: false,
                // auto_increment: false,
                ..
            }
        ));

        assert!(matches!(
            columns[4],
            ColumnDef {
                reference: ColumnRef {
                    name: Cow::Borrowed("echo"),
                    table: Cow::Borrowed("a_table"),
                    schema: Cow::Borrowed(""),
                },
                column_type: Cow::Borrowed("DECIMAL(8, 2)"),
                value: Value::Decimal(None, ..),
                nullable: true,
                default: None,
                primary_key: false,
                unique: false,
                auto_increment: false,
                passive: false,
            }
        ));

        let mut query = String::new();
        assert_eq!(
            WRITER.sql_create_table::<MyEntity>(&mut query, false),
            indoc! {"
                CREATE TABLE a_table(
                alpha DOUBLE NOT NULL,
                bravo SMALLINT NOT NULL,
                charlie DECIMAL,
                delta INTERVAL NOT NULL,
                echo DECIMAL(8, 2),
                PRIMARY KEY (bravo, delta)
                )
            "}
            .trim()
        );
    }

    #[test]
    fn test_customer_schema() {
        #[derive(Entity)]
        #[table_name("customers")]
        struct Customer {
            _transaction_ids: Vec<u64>,
            _preferences: Option<Vec<String>>,
            _lifetime_value: Box<Option<Vec<rust_decimal::Decimal>>>,
            _signup_duration: std::time::Duration,
            #[column_type("DECIMAL(10, 4)[][]")]
            _recent_purchases: Option<Vec<Option<Box<Vec<rust_decimal::Decimal>>>>>,
        }

        let columns = Customer::columns();

        assert_eq!(Customer::table_name(), "customers");
        assert_eq!(Customer::primary_key().len(), 0);

        assert_eq!(columns[0].name(), "transaction_ids");
        assert!(match &columns[0].value {
            Value::List(None, v, ..) => match v.as_ref() {
                Value::UInt64(None, ..) => true,
                _ => false,
            },
            _ => false,
        });
        assert!(!columns[0].nullable);

        assert_eq!(columns[1].name(), "preferences");
        assert!(match &columns[1].value {
            Value::List(None, v, ..) => match v.as_ref() {
                Value::Varchar(None, ..) => true,
                _ => false,
            },
            _ => false,
        });
        assert!(columns[1].nullable);

        assert_eq!(columns[2].name(), "lifetime_value");
        assert!(match &columns[2].value {
            Value::List(None, v, ..) => match v.as_ref() {
                Value::Decimal(None, ..) => true,
                _ => false,
            },
            _ => false,
        });
        assert!(columns[2].nullable);

        assert_eq!(columns[3].name(), "signup_duration");
        assert!(matches!(columns[3].value, Value::Interval(None)));
        assert!(!columns[3].nullable);

        assert_eq!(columns[4].name(), "recent_purchases");
        assert!(match &columns[4].value {
            Value::List(None, v) => match v.as_ref() {
                Value::List(None, v) => match v.as_ref() {
                    Value::Decimal(None, ..) => true,
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        });
        assert!(columns[4].nullable);
        assert_eq!(columns[4].column_type, "DECIMAL(10, 4)[][]");

        let mut query = String::new();
        assert_eq!(
            WRITER.sql_create_table::<Customer>(&mut query, false),
            indoc! {"
                CREATE TABLE customers(
                transaction_ids UBIGINT[] NOT NULL,
                preferences VARCHAR[],
                lifetime_value DECIMAL[],
                signup_duration INTERVAL NOT NULL,
                recent_purchases DECIMAL(10, 4)[][]
                )
            "}
            .trim()
        );
    }

    #[test]
    fn test_trade_execution_schema() {
        assert_eq!(TradeExecution::table_name(), "trade_executions");
        assert_eq!(TradeExecution::primary_key().len(), 2);
        {
            let mut query = String::new();
            assert_eq!(
                WRITER.sql_create_table::<TradeExecution>(&mut query, false),
                indoc! {"
                CREATE TABLE trade_executions(
                trade_id UBIGINT NOT NULL,
                order_id UUID NOT NULL,
                symbol VARCHAR NOT NULL,
                price DECIMAL NOT NULL,
                quantity UINTEGER NOT NULL,
                execution_time TIMESTAMP NOT NULL,
                currency VARCHAR,
                is_internalized BOOLEAN NOT NULL,
                venue VARCHAR,
                child_trade_ids BIGINT[],
                metadata BLOB,
                tags MAP(VARCHAR, VARCHAR),
                PRIMARY KEY (trade_id, execution_time)
                )
            "}
                .trim()
            );
        }
    }
}
