# Tank
Tank (Table Abstraction & Navigation Kit): the Rust data layer.

It's a simple and flexible ORM that allows to manage in a unified way data from different sources.

## Design Goals
- Async-first API
- Simple workflow — no hidden or complex queries
- Extensible driver system
- Works with SQL and non-SQL data sources
- Rich type support with automatic conversions
- Optional appender API for high-performance bulk inserts

## Non goals
- No schema migrations
- No implicit joins (no entities as fields, foreign keys are explicit)
- No complex query builder (use raw sql instead, type conversion is still supported)

## Getting started
1) Add tank to your project
```sh
cargo add tank
```

2) Add a driver crate
```sh
cargo add tank-duckdb
```

3) Declare a entity
```rust
use std::borrow::Cow;
use tank::{Entity, Executor, Result};

#[derive(Entity)]
#[tank(schema = "army")]
pub struct Tank {
    #[tank(primary_key)]
    pub name: String,
    pub country: Cow<'static, str>,
    #[tank(name = "caliber")]
    pub caliber_mm: u16,
    #[tank(name = "speed")]
    pub speed_kmh: f32,
    pub is_operational: bool,
    pub units_produced: Option<u32>,
}
```

4) Connect and query
```rust
use tank::Driver;
use tank_duckdb::DuckDBDriver;

async fn data() -> Result<()> {

    let driver = DuckDBDriver::new();
    let connection = driver
        .connect("duckdb://../target/debug/tests.duckdb?mode=rw".into())
        .await
        .expect("Could not open the database");

    let my_tank = Tank {
        name: "Tiger I".into(),
        country: "Germany".into(),
        caliber_mm: 88,
        speed_kmh: 45.4,
        is_operational: false,
        units_produced: Some(1_347),
    };

    /*
     * CREATE SCHEMA IF NOT EXISTS army;
     * CREATE TABLE IF NOT EXISTS army.tank (
     *     name VARCHAR PRIMARY KEY,
     *     country VARCHAR NOT NULL,
     *     caliber USMALLINT NOT NULL,
     *     speed FLOAT NOT NULL,
     *     is_operational BOOLEAN NOT NULL,
     *     units_produced UINTEGER
     * );
     */
    Tank::create_table(connection, true, true)
        .await
        .expect("Failed to create Tank table");

    /*
     * INSERT INTO army.tank (name, country, caliber, speed, is_operational, units_produced) VALUES
     *     ('Tiger I', 'Germany', 88, 45.4, false, 1347)
     * ON CONFLICT (name) DO UPDATE SET
     *     country = EXCLUDED.country,
     *     caliber = EXCLUDED.caliber,
     *     speed = EXCLUDED.speed,
     *     is_operational = EXCLUDED.is_operational,
     *     units_produced = EXCLUDED.units_produced;
     */
    my_tank.save(connection).await?;

    /*
     * INSERT INTO army.tank (name, country, caliber, speed, is_operational, units_produced) VALUES
     *    ('T-34/85', 'Soviet Union', 85, 53.0, false, 49200),
     *    ('M1 Abrams', 'USA', 120, 67.7, true, NULL);
     */
    Tank::insert_many(
        connection,
        &[
            Tank {
                name: "T-34/85".into(),
                country: "Soviet Union".into(),
                caliber_mm: 85,
                speed_kmh: 53.0,
                is_operational: false,
                units_produced: Some(49_200),
            },
            Tank {
                name: "M1 Abrams".into(),
                country: "USA".into(),
                caliber_mm: 120,
                speed_kmh: 67.7,
                is_operational: true,
                units_produced: None,
            },
        ],
    )
    .await?;

    /*
     * SELECT name, country, caliber, speed, is_operational, units_produced
     * FROM army.tank
     * WHERE is_operational = false
     * LIMIT 1000;
     */
    let tanks = Tank::find_many(
        connection,
        &expr!(Tank::is_operational == false),
        Some(1000),
    )
    .try_collect::<Vec<_>>()
    .await?;

    assert_eq!(
        tanks
            .iter()
            .map(|t| t.name.to_string())
            .collect::<HashSet<_>>(),
        HashSet::from_iter(["Tiger I".into(), "T-34/85".into()])
    );
    Ok(())
}
```
