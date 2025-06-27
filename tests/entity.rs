mod resource {
    pub mod trade;
}

#[cfg(test)]
mod tests {
    use crate::resource::trade::TradeExecution;
    use indoc::indoc;
    use regex::Regex;
    use rust_decimal::Decimal;
    use std::{collections::BTreeMap, sync::Arc, time::Duration};
    use tank::{expr, ColumnDef, ColumnRef, Entity, Passive, PrimaryKeyType, SqlWriter, Value};
    use time::macros::datetime;
    use uuid::Uuid;

    struct Writer;
    impl SqlWriter for Writer {
        fn as_dyn(&self) -> &dyn SqlWriter {
            self
        }
    }

    const WRITER: Writer = Writer {};

    #[test]
    fn test_customer_schema() {
        #[derive(Entity)]
        #[tank(name = "customers")]
        struct Customer {
            _transaction_ids: Vec<u64>,
            _preferences: Option<Vec<String>>,
            _lifetime_value: Box<Option<Vec<rust_decimal::Decimal>>>,
            _signup_duration: std::time::Duration,
            #[tank(type = "DECIMAL(10, 4)[][]")]
            _recent_purchases: Option<Vec<Option<Box<Vec<rust_decimal::Decimal>>>>>,
        }

        assert_eq!(Customer::table_ref().name, "customers");
        assert_eq!(Customer::primary_key_def().len(), 0);

        let columns = Customer::columns_def();
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
                CREATE TABLE customers (
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
        assert_eq!(TradeExecution::table_ref().name, "trade_executions");
        assert_eq!(TradeExecution::primary_key_def().len(), 2);
        {
            let mut query = String::new();
            assert_eq!(
                WRITER.sql_create_table::<TradeExecution>(&mut query, false),
                indoc! {"
                CREATE TABLE trade_executions (
                trade_id UBIGINT NOT NULL,
                order_id UUID NOT NULL DEFAULT '241d362d-797e-4769-b3f6-412440c8cf68',
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
        {
            let trade = TradeExecution {
                trade: 46923,
                order: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
                symbol: "AAPL".to_string(),
                price: Decimal::new(19255, 2), // 192.55
                quantity: 50,
                execution_time: Passive::Set(datetime!(2025-06-07 14:32:00)),
                currency: Some("USD".to_string()),
                is_internalized: true,
                venue: Some("NASDAQ".to_string()),
                child_trade_ids: Some(vec![36209, 85320]),
                metadata: Some(b"Metadata Bytes".to_vec().into_boxed_slice()),
                tags: Some(BTreeMap::from_iter([
                    ("source".into(), "internal".into()),
                    ("strategy".into(), "scalping".into()),
                ])),
            };
            {
                let mut query = String::new();
                WRITER.sql_insert(&mut query, &trade, false);
                let expected = [
                "{'source':'internal','strategy':'scalping'}",
                "{'strategy':'scalping','source':'internal'}",
            ]
            .map(|v| {
                Regex::new(r" {2,}")
                    .expect("Regex must be correct")
                    .replace_all(&format!("
                        INSERT INTO trade_executions (trade_id, order_id, symbol, price, quantity, execution_time, currency, is_internalized, venue, child_trade_ids, metadata, tags)
                        VALUES (46923, '550e8400-e29b-41d4-a716-446655440000', 'AAPL', 192.55, 50, '2025-06-07 14:32:00.0', 'USD', true, 'NASDAQ', [36209,85320], '\\4D\\65\\74\\61\\64\\61\\74\\61\\20\\42\\79\\74\\65\\73', {})
                    ", v), "")
                    .trim()
                    .to_string()
            });
                assert!(expected.iter().any(|v| query == *v));
            };
        }
    }
}
