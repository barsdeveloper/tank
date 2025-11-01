use std::sync::LazyLock;
use tank::{Entity, Executor, Result};
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

pub async fn operations<E: Executor>(executor: &mut E) -> Result<()> {
    let _lock = MUTEX.lock().await;

    // Deployment
    Operator::create_table(&mut executor, true, true).await?;
    RadioLog::create_table(&mut executor, true, false).await?;

    RadioLog::drop_table(&mut executor, true, false).await?;
    Operator::drop_table(&mut executor, true, false).await?;

    // Insertion Tactics
    let operator = Operator {
        id: Uuid::new_v4(),
        callsign: "SteelHammer".into(),
        service_rank: "Lt".into(),
        enlisted: date!(2022 - 03 - 14),
        is_certified: true,
    };
    Operator::insert_one(&mut executor, &operator).await?;

    let op_id = operator.id;
    let logs: Vec<RadioLog> = (0..5).map(|i| RadioLog {
        id: Uuid::new_v4(),
        operator: op_id,
        message: format!("Ping #{i}"),
        unit_callsign: "Alpha-1".into(),
        transmission_time: OffsetDateTime::now_utc(),
        signal_strength: 42,
    }).collect();
    RadioLog::insert_many(&mut executor, &logs).await?;

    // Recon
    let found = Operator::find_pk(&mut executor, &operator.id).await?;
    if let Some(op) = found { /* confirm identity */ }

    let maybe = RadioLog::find_one(&mut executor, &expr!(RadioLog::unit_callsign == "Alpha-1"))
        .await?;

    let mut stream = RadioLog::find_many(
        &mut executor,
        &expr!(RadioLog::signal_strength >= 40),
        Some(100)
    );
    let mut log;
    while let Some(row) = stream.next().await {
        log = row?;
    }

    // Updating
    let mut operator = operator;
    operator.callsign = "SteelHammerX".into();
    operator.save(&mut executor).await?;

    let mut log = RadioLog::find_one(&mut executor, &expr!(RadioLog::message == "Ping #2"))
        .await?
        .expect("Missing log");
    log.message = "Ping #2 ACK".into();
    log.save(&mut executor).await?;

    // Deletion Maneuvers
    RadioLog::delete_one(&mut executor, log.id).await?

    RadioLog::delete_many(&mut executor, &expr!(RadioLog::operator == operator.id)).await?;

    operator.delete(&mut executor).await?;

    // Prepared Recon
    let mut query = RadioLog::table()
        .prepare([RadioLog::message], &mut executor, &expr!(RadioLog::signal_strength > ?), None)
        .await?;
    if let Query::Prepared(p) = &mut query {
        p.bind(40)?;
    }
    let messages: Vec<_> = query
        .fetch_many(&mut executor)
        .map_ok(|row| row.values[0].clone())
        .try_collect()
        .await?;

    // Multi-Statement Burst
    let writer = executor.driver().sql_writer();
    let mut sql = String::new();
    writer.write_delete::<RadioLog>(&mut sql, &expr!(RadioLog::signal_strength < 10));
    writer.write_insert(&mut sql, [&RadioLog {
        id: Uuid::new_v4(),
        operator: operator.id,
        message: "Status report".into(),
        unit_callsign: "Alpha-1".into(),
        transmission_time: OffsetDateTime::now_utc(),
        signal_strength: 55,
    }], false);
    writer.write_select(&mut sql, RadioLog::columns(), RadioLog::table(), &expr!(true), Some(50));
    let mut stream = executor.run(sql.into());

    Ok(())
}
