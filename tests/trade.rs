#![feature(box_patterns)]

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rust_decimal::Decimal;
    use std::{
        borrow::Cow,
        collections::{BTreeMap, HashMap},
        iter,
    };
    use tank::{
        Entity, Expression, GenericSqlWriter, Operand, Passive, PrimaryKeyType, SqlWriter,
        TableRef, Value, expr,
    };
    use time::macros::datetime;
    use uuid::Uuid;

    #[derive(Entity)]
    #[tank(schema = "trading.company", name = "trade_execution", primary_key = ("trade_id", "execution_time"))]
    pub struct Trade {
        #[tank(name = "trade_id")]
        pub trade: u64,
        #[tank(name = "order_id", default = "241d362d-797e-4769-b3f6-412440c8cf68")]
        pub order: Uuid,
        /// Ticker symbol
        pub symbol: String,
        pub price: rust_decimal::Decimal,
        pub quantity: u32,
        pub execution_time: Passive<time::PrimitiveDateTime>,
        pub currency: Option<String>,
        pub is_internalized: bool,
        /// Exchange
        pub venue: Option<String>,
        pub child_trade_ids: Option<Vec<i64>>,
        pub metadata: Option<Box<[u8]>>,
        pub tags: Option<BTreeMap<String, String>>,
    }
    impl Trade {
        pub fn sample() -> Self {
            Self {
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
            }
        }
    }
    const WRITER: GenericSqlWriter = GenericSqlWriter {};

    #[tokio::test]
    async fn test_trade() {
        assert!(matches!(
            Trade::table_ref(),
            TableRef {
                name: "trade_execution",
                schema: "trading.company",
                alias: Cow::Borrowed(""),
            }
        ));

        assert_eq!(Trade::primary_key_def().len(), 2);
        let columns = Trade::columns_def();
        assert_eq!(columns.len(), 12);
        assert_eq!(columns[0].reference.name, "trade_id");
        assert_eq!(columns[1].reference.name, "order_id");
        assert_eq!(columns[2].reference.name, "symbol");
        assert_eq!(columns[3].reference.name, "price");
        assert_eq!(columns[4].reference.name, "quantity");
        assert_eq!(columns[5].reference.name, "execution_time");
        assert_eq!(columns[6].reference.name, "currency");
        assert_eq!(columns[7].reference.name, "is_internalized");
        assert_eq!(columns[8].reference.name, "venue");
        assert_eq!(columns[9].reference.name, "child_trade_ids");
        assert_eq!(columns[10].reference.name, "metadata");
        assert_eq!(columns[11].reference.name, "tags");
        assert_eq!(columns[0].reference.table, "trade_execution");
        assert_eq!(columns[1].reference.table, "trade_execution");
        assert_eq!(columns[2].reference.table, "trade_execution");
        assert_eq!(columns[3].reference.table, "trade_execution");
        assert_eq!(columns[4].reference.table, "trade_execution");
        assert_eq!(columns[5].reference.table, "trade_execution");
        assert_eq!(columns[6].reference.table, "trade_execution");
        assert_eq!(columns[7].reference.table, "trade_execution");
        assert_eq!(columns[8].reference.table, "trade_execution");
        assert_eq!(columns[9].reference.table, "trade_execution");
        assert_eq!(columns[10].reference.table, "trade_execution");
        assert_eq!(columns[11].reference.table, "trade_execution");
        assert_eq!(columns[0].reference.schema, "trading.company");
        assert_eq!(columns[1].reference.schema, "trading.company");
        assert_eq!(columns[2].reference.schema, "trading.company");
        assert_eq!(columns[3].reference.schema, "trading.company");
        assert_eq!(columns[4].reference.schema, "trading.company");
        assert_eq!(columns[5].reference.schema, "trading.company");
        assert_eq!(columns[6].reference.schema, "trading.company");
        assert_eq!(columns[7].reference.schema, "trading.company");
        assert_eq!(columns[8].reference.schema, "trading.company");
        assert_eq!(columns[9].reference.schema, "trading.company");
        assert_eq!(columns[10].reference.schema, "trading.company");
        assert_eq!(columns[11].reference.schema, "trading.company");
        assert!(matches!(columns[0].value, Value::UInt64(..)));
        assert!(matches!(columns[1].value, Value::Uuid(..)));
        assert!(matches!(columns[2].value, Value::Varchar(..)));
        assert!(matches!(columns[3].value, Value::Decimal(..)));
        assert!(matches!(columns[4].value, Value::UInt32(..)));
        assert!(matches!(columns[5].value, Value::Timestamp(..)));
        assert!(matches!(columns[6].value, Value::Varchar(..)));
        assert!(matches!(columns[7].value, Value::Boolean(..)));
        assert!(matches!(columns[8].value, Value::Varchar(..)));
        assert!(matches!(
            columns[9].value,
            Value::List(_, box Value::Int64(..), ..)
        ));
        assert!(matches!(columns[10].value, Value::Blob(..)));
        assert!(matches!(
            columns[11].value,
            Value::Map(_, box Value::Varchar(..), box Value::Varchar(..), ..)
        ));
        assert_eq!(columns[0].nullable, false);
        assert_eq!(columns[1].nullable, false);
        assert_eq!(columns[2].nullable, false);
        assert_eq!(columns[3].nullable, false);
        assert_eq!(columns[4].nullable, false);
        assert_eq!(columns[5].nullable, false);
        assert_eq!(columns[6].nullable, true);
        assert_eq!(columns[7].nullable, false);
        assert_eq!(columns[8].nullable, true);
        assert_eq!(columns[9].nullable, true);
        assert_eq!(columns[10].nullable, true);
        assert_eq!(columns[11].nullable, true);
        assert!(matches!(columns[0].default, None));
        let column1_default =
            columns[1].default.as_deref().unwrap() as *const dyn Expression as *const Operand;
        assert!(matches!(
            unsafe { &*column1_default },
            Operand::LitStr("241d362d-797e-4769-b3f6-412440c8cf68"),
        ));
        assert!(matches!(columns[2].default, None));
        assert!(matches!(columns[3].default, None));
        assert!(matches!(columns[4].default, None));
        assert!(matches!(columns[5].default, None));
        assert!(matches!(columns[6].default, None));
        assert!(matches!(columns[7].default, None));
        assert!(matches!(columns[8].default, None));
        assert!(matches!(columns[9].default, None));
        assert!(matches!(columns[10].default, None));
        assert!(matches!(columns[11].default, None));
        assert_eq!(columns[0].primary_key, PrimaryKeyType::PartOfPrimaryKey);
        assert_eq!(columns[1].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[2].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[3].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[4].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[5].primary_key, PrimaryKeyType::PartOfPrimaryKey);
        assert_eq!(columns[6].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[7].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[8].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[9].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[10].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[11].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[0].unique, false);
        assert_eq!(columns[1].unique, false);
        assert_eq!(columns[2].unique, false);
        assert_eq!(columns[3].unique, false);
        assert_eq!(columns[4].unique, false);
        assert_eq!(columns[5].unique, false);
        assert_eq!(columns[6].unique, false);
        assert_eq!(columns[7].unique, false);
        assert_eq!(columns[8].unique, false);
        assert_eq!(columns[9].unique, false);
        assert_eq!(columns[10].unique, false);
        assert_eq!(columns[11].unique, false);
        assert_eq!(columns[0].passive, false);
        assert_eq!(columns[1].passive, false);
        assert_eq!(columns[2].passive, false);
        assert_eq!(columns[3].passive, false);
        assert_eq!(columns[4].passive, false);
        assert_eq!(columns[5].passive, true);
        assert_eq!(columns[6].passive, false);
        assert_eq!(columns[7].passive, false);
        assert_eq!(columns[8].passive, false);
        assert_eq!(columns[9].passive, false);
        assert_eq!(columns[10].passive, false);
        assert_eq!(columns[11].passive, false);
    }

    #[test]
    fn test_trade_create_table() {
        let mut query = String::new();
        WRITER.write_create_table::<Trade>(&mut query, false);
        assert_eq!(
            query,
            indoc! {"
                CREATE TABLE trading.company.trade_execution (
                trade_id UBIGINT,
                order_id UUID NOT NULL DEFAULT '241d362d-797e-4769-b3f6-412440c8cf68',
                symbol VARCHAR NOT NULL COMMENT 'Ticker symbol',
                price DECIMAL NOT NULL,
                quantity UINTEGER NOT NULL,
                execution_time TIMESTAMP,
                currency VARCHAR,
                is_internalized BOOLEAN NOT NULL,
                venue VARCHAR COMMENT 'Exchange',
                child_trade_ids BIGINT[],
                metadata BLOB,
                tags MAP(VARCHAR, VARCHAR),
                PRIMARY KEY (trade_id, execution_time)
                )
            "}
            .trim()
        );
    }

    #[test]
    fn test_trade_drop_table() {
        let mut query = String::new();
        WRITER.write_drop_table::<Trade>(&mut query, true);
        assert_eq!(
            query,
            "DROP TABLE IF EXISTS trading.company.trade_execution"
        );
    }

    #[test]
    fn test_trade_select() {
        let mut query = String::new();
        WRITER.write_select::<Trade, _, _>(
            &mut query,
            Trade::table_ref(),
            &expr!(Trade::quantity >= 100 && Trade::price > 1000),
            None,
        );
        assert_eq!(
            query,
            indoc! {"
                SELECT trade_id, order_id, symbol, price, quantity, execution_time, currency, is_internalized, venue, child_trade_ids, metadata, tags
                FROM trading.company.trade_execution
                WHERE quantity >= 100 AND price > 1000
            "}
            .trim()
        );
    }

    #[test]
    fn test_employee_insert() {
        let mut docs = HashMap::new();
        docs.insert("contract.pdf".to_string(), vec![1, 2, 3, 4]);
        let employee = Trade::sample();
        let mut query = String::new();
        WRITER.write_insert(&mut query, iter::once(&employee), false);
        assert!(
            // Last part of the query (the map) is removed becaus order of keys is not defined. Value stores a HashMap
            query.starts_with(indoc! {"
                INSERT INTO trading.company.trade_execution (trade_id, order_id, symbol, price, quantity, execution_time, currency, is_internalized, venue, child_trade_ids, metadata, tags)
                VALUES (46923, '550e8400-e29b-41d4-a716-446655440000', 'AAPL', 192.55, 50, '2025-06-07 14:32:00.0', 'USD', true, 'NASDAQ', [36209,85320], '\\x4D\\x65\\x74\\x61\\x64\\x61\\x74\\x61\\x20\\x42\\x79\\x74\\x65\\x73', 
            "}.trim())
    );
    }

    #[test]
    fn test_sql_delete() {
        let mut query = String::new();
        WRITER.write_delete::<Trade, _>(&mut query, &expr!(Trade::trade == 68391));
        assert_eq!(
            query,
            indoc! {"
                DELETE FROM trading.company.trade_execution
                WHERE trade_id = 68391
            "}
            .trim()
        );
    }
}
