<div align="center">
    <img width="300" height="300" src="https://github.com/barsdeveloper/tank/blob/master/docs/logo.png?raw=true" alt="Tank: Table Abstraction & Navigation Kit logo featuring a green tank with a gear background and stacked database cylinders" />
</div>

# Tank
Tank (Table Abstraction & Navigation Kit): the Rust data layer.

Simple and flexible ORM that manages in a unified way data from different sources.

https://github.com/barsdeveloper/tank ⭐

https://crates.io/crates/tank

**Known battlefields**:
- DuckDB
- SQLite
- PostgreSQL (Coming soon)
- MySQL (Coming soon)
- Cassandra/ScyllaDB (Coming soon)
- More to be decided...

## Mission objectives
- Async-first API - fire and forget.
- Simple workflow - every query is visible on your tactical map.
- Extensible driver system - swap databases like changing magazines mid-battle.
- SQL and NoSQL support: one tank, all terrains.
- Rich type arsenal with automatic conversions.
- Optional appender API for high caliber bulk inserts.

## No-fly zone
- No schema migrations
- No implicit joins (no entities as fields, every alliance is signed)
- No complex query builder (write raw SQL and take full credit)

## Getting started
1) Arm your cargo
```sh
cargo add tank
```

2) Choose your ammunition
```sh
cargo add tank-duckdb
```

3) Define unit schematics
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

4) Fire for effect
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
     * DROP TABLE IF EXISTS "army"."tank";
     */
    Tank::drop_table(connection, true, false)
        .await
        .expect("Failed to drop Tank table");

    /*
     * CREATE SCHEMA IF NOT EXISTS "army";
     * CREATE TABLE IF NOT EXISTS "army"."tank" (
     *     "name" VARCHAR PRIMARY KEY,
     *     "country" VARCHAR NOT NULL,
     *     "caliber" USMALLINT NOT NULL,
     *     "speed" FLOAT NOT NULL,
     *     "is_operational" BOOLEAN NOT NULL,
     *     "units_produced" UINTEGER
     * );
     */
    Tank::create_table(connection, true, true)
        .await
        .expect("Failed to create Tank table");

    /*
     * INSERT INTO "army"."tank" ("name", "country", "caliber", "speed", "is_operational", "units_produced") VALUES
     *     ('Tiger I', 'Germany', 88, 45.4, false, 1347)
     * ON CONFLICT ("name") DO UPDATE SET
     *     "country" = EXCLUDED."country",
     *     "caliber" = EXCLUDED."caliber",
     *     "speed" = EXCLUDED."speed",
     *     "is_operational" = EXCLUDED."is_operational",
     *     "units_produced" = EXCLUDED."units_produced";
     */
    my_tank.save(connection).await?;

    /*
     * INSERT INTO "army"."tank" ("name", "country", "caliber", "speed", "is_operational", "units_produced") VALUES
     *     ('T-34/85', 'Soviet Union', 85, 53.0, false, 49200),
     *     ('M1 Abrams', 'USA', 120, 72.0, true, NULL);
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
                speed_kmh: 72.0,
                is_operational: true,
                units_produced: None,
            },
        ],
    )
    .await?;

    /*
     * SELECT "name", "country", "caliber", "speed", "is_operational", "units_produced"
     * FROM "army"."tank"
     * WHERE "is_operational" = false
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

*Rustaceans don't hide behind ORMs, they drive Tanks.*
