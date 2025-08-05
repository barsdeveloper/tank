use rust_decimal::Decimal;
use std::{array, collections::BTreeMap, str::FromStr, sync::LazyLock};
use tank::{Connection, Entity, Passive, stream::StreamExt, stream::TryStreamExt};
use time::macros::datetime;
use tokio::sync::Mutex;
use uuid::Uuid;

static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Entity, Debug)]
#[tank(schema = "trading", name = "trade_execution", primary_key = ("trade_id", "execution_time"))]
pub struct Trade {
    #[tank(name = "trade_id")]
    pub trade: u64,
    #[tank(name = "order_id", default = "241d362d-797e-4769-b3f6-412440c8cf68")]
    pub order: Uuid,
    /// Ticker symbol
    pub symbol: String,
    pub isin: [char; 12],
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

pub async fn trade_simple<C: Connection>(connection: &mut C) {
    let _lock = MUTEX.lock().await;

    // Setup
    Trade::drop_table(connection, true, false)
        .await
        .expect("Failed to drop Trade table");
    Trade::create_table(connection, false, true)
        .await
        .expect("Failed to create Trade table");

    // Trade object
    let trade = Trade {
        trade: 46923,
        order: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        symbol: "RIVN".to_string(),
        isin: array::from_fn(|i| "US76954A1034".chars().nth(i).unwrap()),
        price: Decimal::new(1226, 2), // 12.26
        quantity: 500,
        execution_time: datetime!(2025-06-07 14:32:00).into(),
        currency: Some("USD".into()),
        is_internalized: true,
        venue: Some("NASDAQ".into()),
        child_trade_ids: vec![36209, 85320].into(),
        metadata: b"Metadata Bytes".to_vec().into_boxed_slice().into(),
        tags: BTreeMap::from_iter([
            ("source".into(), "internal".into()),
            ("strategy".into(), "scalping".into()),
        ])
        .into(),
    };

    // Expect to find no trades
    let result = Trade::find_pk(connection, &trade.primary_key())
        .await
        .expect("Failed to find trade by primary key");
    assert!(result.is_none(), "Expected no trades at this time");
    assert_eq!(Trade::find_many(connection, &true, None).count().await, 0);

    // Delete unexisting trade
    trade
        .delete(connection)
        .await
        .expect_err("Expected to fail delete");

    // Save a trade
    trade.save(connection).await.expect("Failed to save trade");

    // Expect to find the only trade
    let result = Trade::find_pk(connection, &trade.primary_key())
        .await
        .expect("Failed to find trade");
    assert!(
        result.is_some(),
        "Expected Trade::find_pk to return some result",
    );
    let result = result.unwrap();
    assert_eq!(result.trade, 46923);
    assert_eq!(
        result.order,
        Uuid::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
    );
    assert_eq!(result.symbol, "RIVN");
    assert_eq!(
        result
            .isin
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(""),
        "US76954A1034"
    );
    assert_eq!(result.price, Decimal::new(1226, 2));
    assert_eq!(result.quantity, 50);
    assert_eq!(
        result.execution_time,
        Passive::Set(datetime!(2025-06-07 14:32:00))
    );
    assert_eq!(result.currency, Some("USD".into()));
    assert_eq!(result.is_internalized, true);
    assert_eq!(result.venue, Some("NASDAQ".into()));
    assert_eq!(result.child_trade_ids, Some(vec![36209, 85320]));
    assert_eq!(
        result.metadata,
        Some(b"Metadata Bytes".to_vec().into_boxed_slice())
    );
    let Some(tags) = result.tags else {
        unreachable!("Tag is expected");
    };
    assert_eq!(tags.len(), 2);
    assert_eq!(
        tags,
        BTreeMap::from_iter([
            ("source".into(), "internal".into()),
            ("strategy".into(), "scalping".into())
        ])
    );

    assert_eq!(Trade::find_many(connection, &true, None).count().await, 1);
}

pub async fn trade_multiple<C: Connection>(connection: &mut C) {
    let _lock = MUTEX.lock().await;

    // Setup
    Trade::drop_table(connection, false, false)
        .await
        .expect("Failed to drop Trade table");
    Trade::create_table(connection, false, true)
        .await
        .expect("Failed to create Trade table");

    // Trade objects
    let trades = vec![
        Trade {
            trade: 10001,
            order: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            symbol: "AAPL".to_string(),
            isin: array::from_fn(|i| "US0378331005".chars().nth(i).unwrap()),
            price: Decimal::new(15000, 2),
            quantity: 10,
            execution_time: datetime!(2025-06-01 09:00:00).into(),
            currency: Some("USD".into()),
            is_internalized: false,
            venue: Some("NASDAQ".into()),
            child_trade_ids: Some(vec![101, 102]),
            metadata: Some(b"First execution".to_vec().into_boxed_slice()),
            tags: Some(BTreeMap::from_iter([
                ("source".into(), "algo".into()),
                ("strategy".into(), "momentum".into()),
            ])),
        },
        Trade {
            trade: 10002,
            order: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
            symbol: "GOOG".to_string(),
            isin: array::from_fn(|i| "US02079K3059".chars().nth(i).unwrap()),
            price: Decimal::new(280000, 3), // 280.000
            quantity: 5,
            execution_time: datetime!(2025-06-02 10:15:30).into(),
            currency: Some("USD".into()),
            is_internalized: true,
            venue: Some("NYSE".into()),
            child_trade_ids: Some(vec![]),
            metadata: Some(b"Second execution".to_vec().into_boxed_slice()),
            tags: Some(BTreeMap::from_iter([
                ("source".into(), "internal".into()),
                ("strategy".into(), "mean_reversion".into()),
            ])),
        },
        Trade {
            trade: 10003,
            order: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
            symbol: "MSFT".to_string(),
            isin: array::from_fn(|i| "US5949181045".chars().nth(i).unwrap()),
            price: Decimal::new(32567, 2), // 325.67
            quantity: 20,
            execution_time: datetime!(2025-06-03 11:45:00).into(),
            currency: Some("USD".into()),
            is_internalized: false,
            venue: Some("BATS".into()),
            child_trade_ids: Some(vec![301]),
            metadata: Some(b"Third execution".to_vec().into_boxed_slice()),
            tags: Some(BTreeMap::from_iter([
                ("source".into(), "external".into()),
                ("strategy".into(), "arbitrage".into()),
            ])),
        },
        Trade {
            trade: 10004,
            order: Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap(),
            symbol: "TSLA".to_string(),
            isin: array::from_fn(|i| "US88160R1014".chars().nth(i).unwrap()),
            price: Decimal::new(62000, 2), // 620.00
            quantity: 15,
            execution_time: datetime!(2025-06-04 14:00:00).into(),
            currency: Some("USD".into()),
            is_internalized: true,
            venue: Some("CBOE".into()),
            child_trade_ids: None,
            metadata: None,
            tags: Some(BTreeMap::from_iter([
                ("source".into(), "manual".into()),
                ("strategy".into(), "news_event".into()),
            ])),
        },
        Trade {
            trade: 10005,
            order: Uuid::parse_str("55555555-5555-5555-5555-555555555555").unwrap(),
            symbol: "AMZN".to_string(),
            isin: array::from_fn(|i| "US0231351067".chars().nth(i).unwrap()),
            price: Decimal::new(134899, 3), // 1348.99
            quantity: 8,
            execution_time: datetime!(2025-06-05 16:30:00).into(),
            currency: Some("USD".into()),
            is_internalized: false,
            venue: Some("NASDAQ".into()),
            child_trade_ids: Some(vec![501, 502, 503]),
            metadata: Some(b"Fifth execution".to_vec().into_boxed_slice()),
            tags: Some(BTreeMap::from_iter([
                ("source".into(), "internal".into()),
                ("strategy".into(), "scalping".into()),
            ])),
        },
    ];

    // Insert 5 trades
    for trade in &trades {
        trade
            .save(connection)
            .await
            .expect(&format!("Failed to save save {} trade", trade.symbol));
    }

    // Find 5 trades
    let data = Trade::find_many(connection, &true, None)
        .try_collect::<Vec<_>>()
        .await
        .expect("Failed to query threads");
    assert_eq!(data.len(), 5, "Expect to find 5 trades");

    // Verify data integrity
    for (i, expected) in trades.iter().enumerate() {
        let actual_a = &data[i];
        let actual_b = Trade::find_pk(connection, &expected.primary_key())
            .await
            .expect(&format!("Failed to find trade {} by pk", data[i].symbol));
        let Some(actual_b) = actual_b else {
            panic!("Trade {} not found", expected.trade);
        };

        assert_eq!(actual_a.trade, expected.trade);
        assert_eq!(actual_b.trade, expected.trade);

        assert_eq!(actual_a.order, expected.order);
        assert_eq!(actual_b.order, expected.order);

        assert_eq!(actual_a.symbol, expected.symbol);
        assert_eq!(actual_b.symbol, expected.symbol);

        assert_eq!(actual_a.price, expected.price);
        assert_eq!(actual_b.price, expected.price);

        assert_eq!(actual_a.quantity, expected.quantity);
        assert_eq!(actual_b.quantity, expected.quantity);

        assert_eq!(actual_a.execution_time, expected.execution_time);
        assert_eq!(actual_b.execution_time, expected.execution_time);

        assert_eq!(actual_a.currency, expected.currency);
        assert_eq!(actual_b.currency, expected.currency);

        assert_eq!(actual_a.is_internalized, expected.is_internalized);
        assert_eq!(actual_b.is_internalized, expected.is_internalized);

        assert_eq!(actual_a.venue, expected.venue);
        assert_eq!(actual_b.venue, expected.venue);

        assert_eq!(actual_a.child_trade_ids, expected.child_trade_ids);
        assert_eq!(actual_b.child_trade_ids, expected.child_trade_ids);

        assert_eq!(actual_a.metadata, expected.metadata);
        assert_eq!(actual_b.metadata, expected.metadata);

        assert_eq!(actual_a.tags, expected.tags);
        assert_eq!(actual_b.tags, expected.tags);
    }
}
