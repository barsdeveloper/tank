mod resource {
    pub mod trade;
}

#[cfg(test)]
mod tests {

    use crate::resource::trade::TradeExecution;
    use rust_decimal::{prelude::FromPrimitive, Decimal};
    use std::{collections::HashMap, path::Path, sync::Mutex};
    use tank_core::Connection;
    use tank_duckdb::DuckDBConnection;
    use time::{macros::format_description, PrimitiveDateTime};
    use tokio::fs;
    use uuid::Uuid;

    static MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn create_database() {
        const DB_PATH: &'static str = "../target/debug/test_duckdb.db";
        let _guard = MUTEX.lock().unwrap();
        if Path::new(DB_PATH).exists() {
            fs::remove_file(DB_PATH).await.expect(
                format!("Failed to remove existing test database file {}", DB_PATH).as_str(),
            );
        }
        assert!(
            !Path::new(DB_PATH).exists(),
            "Database file should not exist before test"
        );
        DuckDBConnection::connect(format!("duckdb://{}?mode=rw", DB_PATH).as_str())
            .await
            .expect("Could not open the database");
        assert!(
            Path::new(DB_PATH).exists(),
            "Database file should be created after connection"
        );
    }

    #[tokio::test]
    async fn simple_entity() {
        const DB_PATH: &'static str = "../target/debug/test_duckdb.db";
        let _guard = MUTEX.lock().unwrap();
        DuckDBConnection::connect(format!("duckdb://{}?mode=rw", DB_PATH).as_str())
            .await
            .expect("Could not open the database");
        let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        let trade = TradeExecution {
            trade_id: 100001,
            order_id: Uuid::parse_str("a3f1e8b4-4df4-4b8d-8e0e-7b9f5a7e1abc").unwrap(),
            symbol: "AAPL".to_string(),
            price: Decimal::from_f64(172.3450).unwrap(),
            quantity: 250,
            execution_time: PrimitiveDateTime::parse(
                "2025-05-27T14:32:10",
                format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
            )
            .unwrap(),
            currency: Some("USD".to_string()),
            is_internalized: false,
            venue: Some("XNAS".to_string()),
            child_trade_ids: Some(vec![300101, 300102]),
            metadata: Some(
                b"risk_flag=low;strategy=alpha\0"
                    .to_vec()
                    .into_boxed_slice(),
            ),
            tags: Some(HashMap::from([
                ("strategy".to_string(), "alpha".to_string()),
                ("region".to_string(), "US".to_string()),
            ])),
        };
    }
}
