use std::collections::BTreeMap;
use tank::{Entity, Passive};
use uuid::Uuid;

#[derive(Entity)]
#[tank(name = "trade_executions", primary_key = ("trade_id", "execution_time"))]
pub struct TradeExecution {
    #[tank(name = "trade_id")]
    pub trade: u64,
    #[tank(name = "order_id", default = "241d362d-797e-4769-b3f6-412440c8cf68")]
    pub order: Uuid,
    pub symbol: String,
    pub price: rust_decimal::Decimal,
    pub quantity: u32,
    pub execution_time: Passive<time::PrimitiveDateTime>,
    pub currency: Option<String>,
    pub is_internalized: bool,
    pub venue: Option<String>,
    pub child_trade_ids: Option<Vec<i64>>,
    pub metadata: Option<Box<[u8]>>,
    pub tags: Option<BTreeMap<String, String>>,
}
