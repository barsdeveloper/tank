use tank::Entity;
use uuid::Uuid;

#[derive(Entity)]
#[table_name("trade_executions")]
#[primary_key("trade_id")]
pub struct TradeExecution {
    pub trade_id: u64,
    pub order_id: Uuid,
    pub symbol: String,
    pub price: rust_decimal::Decimal,
    pub quantity: u32,
    pub execution_time: time::PrimitiveDateTime,
    pub currency: Option<String>,
    pub is_internalized: bool,
    pub venue: Option<String>,
    pub child_trade_ids: Option<Vec<i64>>,
    pub metadata: Option<Box<[u8]>>,
    pub tags: Option<std::collections::HashMap<String, String>>,
}
