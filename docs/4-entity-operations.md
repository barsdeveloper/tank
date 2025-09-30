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
pub struct Base {
    #[tank(primary_key)]
    pub id: Passive<Uuid>,
    pub codename: String,
    pub location: String,
    pub established: Passive<DateTime<Utc>>,
}
#[derive(Entity)]
pub struct Operative {
    pub id: Passive<Uuid>,
    pub name: String,
    pub rank: String,
    pub specialty: Option<String>,
    pub base_id: Uuid,
}
#[derive(Entity)]
pub struct Mission {
    pub mission_id: Passive<Uuid>,
    pub codename: String,
    pub objective: String,
    pub launch_time: DateTime<Utc>,
}
#[derive(Entity)]
#[tank(table_name = "assignment")]
pub struct MissionAssignment {
    pub mission_id: Uuid,
    pub operative_id: Uuid,
    pub role: Option<String>,
}
```
== SQL
```sql
CREATE TABLE bases (
    base_id UUID PRIMARY KEY,
    codename TEXT NOT NULL,
    location TEXT NOT NULL,
    established TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE operatives (
    operative_id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    rank TEXT NOT NULL,
    specialty TEXT,
    base_id UUID NOT NULL REFERENCES bases(base_id) ON DELETE CASCADE
);
CREATE TABLE missions (
    mission_id UUID PRIMARY KEY,
    codename TEXT NOT NULL,
    objective TEXT NOT NULL,
    launch_time TIMESTAMP NOT NULL
);
CREATE TABLE mission_assignments (
    mission_id UUID NOT NULL REFERENCES missions(mission_id) ON DELETE CASCADE,
    operative_id UUID NOT NULL REFERENCES operatives(operative_id) ON DELETE CASCADE,
    role TEXT,
    PRIMARY KEY (mission_id, operative_id)
);
```
:::
