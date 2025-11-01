# Entity Operations
###### *Field Manual Section 6* - Front-Line Extraction

The Entity is your combat unit, a Rust struct mapped one-to-one with a database table. This section trains you on the basic maneuvers every unit must master: insertions, updates, and extractions.

## Mission Scope
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

*Targets locked. Orders executed. Tank out.*
