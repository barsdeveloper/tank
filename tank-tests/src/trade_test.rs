use futures::StreamExt;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use tank::{Connection, Entity};
use time::macros::datetime;
use uuid::Uuid;

use crate::TradeExecution;

pub async fn trade_test_setup<C: Connection>(connection: &mut C) {
    assert!(TradeExecution::drop_table(connection, true).await.is_ok());
    assert!(TradeExecution::create_table(connection, false)
        .await
        .is_ok());
    let trade = TradeExecution {
        trade: 46923,
        order: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        symbol: "AAPL".to_string(),
        price: Decimal::new(19255, 2), // 192.55
        quantity: 50,
        execution_time: datetime!(2025-06-07 14:32:00).into(),
        currency: "USD".to_string().into(),
        is_internalized: true,
        venue: "NASDAQ".to_string().into(),
        child_trade_ids: vec![36209, 85320].into(),
        metadata: b"Metadata Bytes".to_vec().into_boxed_slice().into(),
        tags: BTreeMap::from_iter([
            ("source".into(), "internal".into()),
            ("strategy".into(), "scalping".into()),
        ])
        .into(),
    };
    assert!(TradeExecution::find_one(connection, &trade.primary_key())
        .await
        .is_err());
    assert_eq!(TradeExecution::find_many(connection, true).count().await, 0);
    assert!(trade.save(connection).await.is_ok());
    assert!(TradeExecution::find_one(connection, &trade.primary_key())
        .await
        .is_ok());
    assert_eq!(TradeExecution::find_many(connection, true).count().await, 1);
}
