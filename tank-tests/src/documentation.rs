use std::sync::LazyLock;
use tank::{Entity, Executor};
use time::{Date, OffsetDateTime};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Entity)]
#[tank(schema = "operations", name = "radio_operator")]
pub struct Operator {
    #[tank(primary_key)]
    pub id: Uuid,
    pub callsign: String,
    #[tank(name = "rank")]
    pub service_rank: String,
    #[tank(name = "enlistment_date")]
    pub enlisted: Date,
    pub is_certified: bool,
}

#[derive(Entity)]
#[tank(schema = "operations")]
pub struct RadioLog {
    #[tank(primary_key)]
    pub id: Uuid,
    #[tank(references = Operator::id)]
    pub operator: Uuid,
    pub message: String,
    pub unit_callsign: String,
    #[tank(name = "tx_time")]
    pub transmissione_time: OffsetDateTime,
    #[tank(name = "rssi")]
    pub signal_strength: i8,
}
static MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub async fn documentation<E: Executor>(executor: &mut E) {
    let _lock = MUTEX.lock().await;
    Operator::drop_table(executor, true, true)
        .await
        .expect("Failed to drop Operator table");
    RadioLog::drop_table(executor, true, true)
        .await
        .expect("Failed to drop RadioLog table");

    Operator::create_table(executor, true, true)
        .await
        .expect("Failed to drop Operator table");
    RadioLog::create_table(executor, true, true)
        .await
        .expect("Failed to drop RadioLog table");
}
