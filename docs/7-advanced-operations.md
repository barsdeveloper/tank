# Advanced Operations
###### *Field Manual Section 7* - Tactical Coordination

In the field, isolated units rarely win the battle. Coordination is key. Joins let you link data across tables like synchronized squads advancing under fire.
In Tank, a join is a first class `DataSet`, just like a `TableRef`. That means you can call `select()` and then, filter, map, reduce, etc, using the same composable [Stream API](https://docs.rs/futures/latest/futures/prelude/trait.Stream.html) you already know.

## Schema In Play
Continuing with the `Operator` and `RadioLog` schema introduce earlier. The following examples show more advanced query capabilities, something that go beyour simple CRUD operations shown earlier but still without revolving to raw sql.
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

### Data
**Operators:**
| callsign    | rank  | enlisted   | is_certified |
| ----------- | ----- | ---------- | ------------ |
| SteelHammer | Major | 2015-06-20 | ✅ true      |
| Viper       | Sgt   | 2019-11-01 | ✅ true      |
| Rook        | Pvt   | 2023-01-15 | ❌ false     |

**Radio logs:**
| message                                  | unit_callsign | tx_time                | rssi |
| ---------------------------------------- | ------------- | ---------------------- | ---- |
| Radio check, channel 3. How copy?        | Alpha-1       | 2025-11-04T19:45:21+01 | −42  |
| Target acquired. Requesting coordinates. | Alpha-1       | 2025-11-04T19:54:12+01 | −55  |
| Heavy armor spotted, grid 4C.            | Alpha-1       | 2025-11-04T19:51:09+01 | −52  |
| Perimeter secure. All clear.             | Bravo-2       | 2025-11-04T19:51:09+01 | −68  |
| Radio check, grid 1A. Over.              | Charlie-3     | 2025-11-04T18:59:11+02 | −41  |
| Affirmative, engaging.                   | Alpha-1       | 2025-11-03T23:11:54+00 | −54  |

## Selecting & Ordering
Here is a minimal example illustrating the use of [`tank::cols!()`](https://docs.rs/tank/0.8.0/tank/macro.cols.html) which supports aliasing and ordering, alternatively if those features are not used, a more terse syntax is possible: `[RadioLog::signal_strength, Operator::callsign, RadioLog::message]` or `Entity::columns()`. Strongest certified transmissions:
```rust
let messages = join!(
    Operator JOIN RadioLog ON Operator::id == RadioLog::operator
)
.select(
    executor,
    cols!(
        RadioLog::signal_strength as strength DESC,
        Operator::callsign ASC,
        RadioLog::message,
    ),
    &expr!(Operator::is_certified && RadioLog::message != "Radio check%" as LIKE),
    Some(100),
)
.map(|row| {
    row.and_then(|row| {
        #[derive(Entity)]
        struct Row {
            message: String,
            callsign: String,
        }
        Row::from_row(row).and_then(|row| Ok((row.message, row.callsign)))
    })
})
.try_collect::<Vec<_>>()
.await?;
assert!(
    messages.iter().map(|(a, b)| (a.as_str(), b.as_str())).eq([
        ("Heavy armor spotted, grid 4C.", "SteelHammer"),
        ("Affirmative, engaging.", "SteelHammer"),
        ("Target acquired. Requesting coordinates.", "SteelHammer"),
        ("Perimeter secure. All clear.", "Viper"),
    ]
    .into_iter())
);
```

## Expr


## Cols
[`tank::cols!(col, ...)`](https://docs.rs/tank/0.8.0/tank/macro.cols.html) macro is more expressive syntax to select columns in a query, it supports:
- `Entity::column`
- `Entity::column as name` (aliasing)
- `Entity::column + 1` (expressions)
- `SUM(Entity::column)` (function calls)
- `COUNT(*)` (function calls)
- `*` (wildcard)
- `COUNT(*)` (function calls)
- `schema.table.column`
- `Entity::column DESC` (ordering)
- `SUM(Entity::column + table.column) as total DESC` (a combination)

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
