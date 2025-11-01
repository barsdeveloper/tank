# Entity Operations
###### *Field Manual Section 6* - Front-Line Extraction

The Entity is your combat unit, a Rust struct mapped one-to-one with a database table. This section trains you on the basic maneuvers every unit must master: insertions, deletions, and extractions.

## Mission Scope
List of every tactical primitive you execute against an `Entity`. Each item maps to a single, clear action. Almost all higher-level patterns are just combinations of these fundamentals.
* `Entity::create_table()`: establish operating base
* `Entity::drop_table()`: break camp
* `Entity::insert_one()`: deploy a single unit
* `Entity::insert_many()`: bulk deployment
* `Entity::find_pk()`: identify the target
* `Entity::find_one()`: silent recon
* `Entity::find_many()`: wide-area sweep
* `Entity::delete_one()`: precision strike
* `Entity::delete_many()`: scorched-earth withdrawal
* `entity.save()`: resupply and hold the position
* `entity.delete()`: stand-down order

## Forward Operations Schema
This is the schema we will use for every operation example that follows. All CRUD, streaming, prepared, and batching demonstrations below act on these two tables so you can focus on behavior instead of switching contexts. `Operator` is the identity table, `RadioLog` references an operator (foreign key) to record transmissions.
::: code-group
```rust [Rust]
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
    pub transmission_time: OffsetDateTime,
    #[tank(name = "rssi")]
    pub signal_strength: i8,
}
```
```sql [SQL]
CREATE TABLE IF NOT EXISTS operations.radio_operator (
    id UUID PRIMARY KEY,
    callsign VARCHAR NOT NULL,
    rank VARCHAR NOT NULL,
    enlistment_date DATE NOT NULL,
    is_certified BOOLEAN NOT NULL);

CREATE TABLE IF NOT EXISTS operations.radio_log (
    id UUID PRIMARY KEY,
    operator UUID NOT NULL REFERENCES operations.radio_operator(id),
    message VARCHAR NOT NULL,
    unit_callsign VARCHAR NOT NULL,
    tx_time TIMESTAMP WITH TIME ZONE NOT NULL,
    rssi TINYINT NOT NULL);"
```
:::

## Deployment
Deployment is the initial insertion of your units into the theater: creating tables (and schema) before any data flows, and tearing them down when the operation ends.
```rust
Operator::create_table(&mut executor, true, true).await?;
RadioLog::create_table(&mut executor, true, false).await?;

RadioLog::drop_table(&mut executor, true, false).await?;
Operator::drop_table(&mut executor, true, false).await?;
```

Key points:
- `if_not_exists` / `if_exists` guard repeated ops.
- Schema creation runs before the table when requested.
- Foreign key in `RadioLog.operator` enforces referential discipline.

## Insertion Tactics
Single unit insertion:
```rust
let operator = Operator {
    id: Uuid::new_v4(),
    callsign: "SteelHammer".into(),
    service_rank: "Lt".into(),
    enlisted: date!(2022 - 03 - 14),
    is_certified: true,
};
Operator::insert_one(&mut executor, &operator).await?;
```

Bulk deployment of logs:
```rust
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
```

## Recon
Find by primary key:
```rust
let found = Operator::find_pk(&mut executor, &operator.id).await?;
if let Some(op) = found { /* confirm identity */ }
```

First matching row:
```rust
let maybe = RadioLog::find_one(&mut executor, &expr!(RadioLog::unit_callsign == "Alpha-1"))
    .await?;
```

All matching transmissions with limit:
```rust
let mut stream = RadioLog::find_many(
    &mut executor,
    &expr!(RadioLog::signal_strength >= 40),
    Some(100)
);
let mut log;
while let Some(row) = stream.next().await {
    log = row?;
}
```

Under the hood: `find_one` is just `find_many` with a limit of 1.

## Updating
`save()` attempts insert or update if the driver supports conflict clauses.
```rust
let mut operator = operator;
operator.callsign = "SteelHammerX".into();
operator.save(&mut executor).await?;
```

RadioLog also has a primary key, so editing a message:
```rust
let mut log = RadioLog::find_one(&mut executor, &expr!(RadioLog::message == "Ping #2"))
    .await?
    .expect("Missing log");
log.message = "Ping #2 ACK".into();
log.save(&mut executor).await?;
```

If a table has no primary key, `save()` returns an error, use `insert_one` instead.

## Deletion Maneuvers
Precision strike:
```rust
RadioLog::delete_one(&mut executor, log.id).await?;
```

Scorched earth pattern:
```rust
RadioLog::delete_many(&mut executor, &expr!(RadioLog::operator == operator.id)).await?;
```

Instance form (validates exactly one row):
```rust
operator.delete(&mut executor).await?;
```

## Prepared Recon
Filter transmissions after a threshold:
```rust
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
```

## Multi-Statement Burst
Combine delete + insert + select in one roundtrip:
```rust
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
```

Process `QueryResult::Affected` then `Row` items sequentially.

## Error Signals & Edge Cases
- `save()` / `delete()` on entities without PK result in immediate error.
- `delete()` with affected rows != 1 logs diagnostic messages.
- Prepared binds validate conversion; failure returns `Result::Err`.

## Performance Hints (Radio Theater)
- Group logs with `insert_many` (thousands per statement) to cut network overhead.
- Use prepared statements for hot paths (changing only parameters).
- Limit streaming scans with a numeric `limit` to avoid unbounded pulls.

*Targets locked. Orders executed. Tank out.*
