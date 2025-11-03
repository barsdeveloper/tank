# Advanced Operations
###### *Field Manual Section 7* - Tactical Coordination

In the field, isolated units rarely win the battle. Coordination is key. Joins let you link data across tables like synchronized squads advancing under fire.
In Tank, a join is a first class `DataSet`, just like a `TableRef`. That means you can call `select()` and then, filter, map, reduce, etc, using the same clean, composable [Stream API](https://docs.rs/futures/latest/futures/prelude/trait.Stream.html) you already know.

## Schema In Play
Continuing with the `Operator` and `RadioLog` schema introduce earlier. Think of the join as a temporary tactical net: it composes existing tables into a streaming view you shape on demand.
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
    rssi TINYINT NOT NULL);
```
:::

## Selecting & Ordering
You almost never need “all columns”. Projecting only `callsign`, `signal_strength`, and `message` cuts I/O and decoding. Column-level ordering keeps intent local (no detached `ORDER BY` to mentally reconcile).

Minimal example (strongest certified transmissions):
```rust
let ds = join!(Operator JOIN RadioLog ON Operator::id == RadioLog::operator);
let rows = ds
    .select(
        executor,
        cols!(RadioLog::signal_strength as strength DESC, Operator::callsign ASC, RadioLog::message),
        &expr!(Operator::is_certified == true && RadioLog::message NOT LINE "Ping %"),
        Some(100)
    );
```

## Aliasing
Aliases are not just cosmetics, they stabilize downstream decoding when underlying DB column names differ (`service_rank` vs `rank`) or when two tables export the same column name (`id`).

Example (rank + message):
```rust
cols!(Operator::service_rank as rank, RadioLog::message as msg)
```

## Custom Hydration (Just Enough Structure)
Define custom entities to read from join results or just a subset of a table.

```rust
#[derive(Entity, Debug)]
struct Transmission {
    callsign: String,
    signal_strength: i8,
}

let transmissions: Vec<Transmission> = ds
    .select(
        executor,
        cols!(Operator::callsign, RadioLog::signal_strength DESC),
        &expr!(RadioLog::signal_strength >= 40),
        None
    )
    .map_ok(Transmission::from_row)
    .map(Result::flatten)
    .try_collect()
    .await?;
```

## Prepared Joins


## Aggregation (Client-Side When Light)
Until server aggregates are wrapped, lightweight reductions happen in-stream:
```rust
let (sum, n) = ds
  .select(executor, cols!(RadioLog::signal_strength), &expr!(true), None)
  .map_ok(|r| r.values[0].clone().try_into().unwrap() as i64)
  .try_fold((0,0), |(s,c), v| async move { Ok((s+v, c+1)) })
  .await?;
```
If `n` is large or you only need a simple aggregate, consider pushing logic server-side once API support lands; for now this is pragmatic and clear.

## Partial Reads (Skim What You Need)
Grabbing only messages:
```rust
let messages: Vec<String> = ds
  .select(executor, cols!(RadioLog::message ASC), &expr!(true), Some(50))
  .map_ok(|r| r.values[0].clone().try_into().unwrap())
  .try_collect()
  .await?;
```
Do this instead of hydrating full entities when only one column drives a decision.

## Performance Notes
- Request only the necessary columns.
- Stream + early break for threshold scans (do not SELECT COUNT then SELECT data).
- Prepared joins: stabilize SQL for hot loops.
- Alias recurrent long names (micro but accumulative in tight loops).
- Prefer limits (`Some(n)`) when you are sampling or confirming existence.

## Edge / Failure Signals
- Referencing a non-joined table column: immediate construction error (fast feedback).
- Duplicate alias: conflict error (prevents silent shadowing).
- LEFT join + non-Option decode on NULL: conversion error (fix by making field optional).
- Type mismatch in `from_row`: bubbles as `Err`, surface early in tests.
- Unused prepared parameter (bind count mismatch): error before execution.

*Units in position. Advance. Tank out.*
