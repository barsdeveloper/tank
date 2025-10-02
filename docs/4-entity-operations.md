# Entity Operations
###### *Field Manual Section 4* - Front-Line Extraction
The Entity is your combat unity, a Rust struct mapped one-to-one with a database table. This section traines you on the basic maneuvers every unit must master: insertions, updates, and extractions.

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
:::tabs
== Rust
```rust
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
```
== SQL
```sql
```
:::
