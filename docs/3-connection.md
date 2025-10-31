# Connection
###### *Field Manual Section 3* - Supply Lines

Welcome to the heavy metal parade, commander. Before you can unleash Tank's firepower, you have to secure your supply lines. Open a **Connection** to your database, and when the mission escalates, lock operations inside a **Transaction**. No connection, no combat. It's that simple.

## Connection: Opening the Channel
Every database connection abstraction implements the `Connection` trait. This is your communication link to the database server. You call `connect()` with a URL, and Tank establishes the line. Every driver is its own crate. Load only what you need for the operation. Check the [equipment](1-introduction.md#equipment) to see the available connections.

### DuckDB
DuckDB is your embedded artillery piece: fast, local, and always ready. Perfect for rapid deployment scenarios and testing under fire.

```rust
use tank::Driver;
use tank_duckdb::{DuckDBConnection, DuckDBDriver};

async fn establish_duckdb_connection() -> Result<DuckDBConnection> {
    let driver = DuckDBDriver::new();
    let connection = driver
        .connect("duckdb://../target/debug/combat.duckdb?mode=rw".into())
        .await?;
    Ok(connection)
}
```

**URL Format**: `duckdb://path/to/database.duckdb?mode=rw`
- `mode=rw`: Read-write access (existing database)
- `mode=rwc`: Create if not exists
- In-memory combat zone: `duckdb://:memory:`

### SQLite
SQLite is your trusty sidearm: lightweight, reliable, zero configuration. Deploy anywhere, anytime.

```rust
use tank::Driver;
use tank_sqlite::{SqliteConnection, SqliteDriver};

async fn establish_sqlite_connection() -> Result<SqliteConnection> {
    let driver = SqliteDriver::new();
    let connection = driver
        .connect("sqlite://../target/debug/operations.sqlite?mode=rwc".into())
        .await?;
    Ok(connection)
}
```

**URL Format**: `sqlite://path/to/database.sqlite?mode=rwc`
- Same mode flags as DuckDB
- In-memory operations: `sqlite://:memory:`

### PostgreSQL
PostgreSQL is your heavy artillery: powerful, networked, built for sustained campaigns with multiple units coordinating strikes.

```rust
use tank_postgres::PostgresConnection;
use tank::Connection;

async fn establish_postgres_connection() -> Result<PostgresConnection> {
    let connection = PostgresConnection::connect(
        "postgres://tank-user:armored@127.0.0.1:5432/military".into()
    )
    .await?;
    Ok(connection)
}
```

**URL Format**: `postgres://user:password@host:port/database`
- Standard PostgreSQL connection string
- Supports all libpq parameters
- Connection pooling handled automatically

## Transaction: Coordinated Strikes

Sometimes you need to execute multiple operations as a single atomic mission—all or nothing. That's where **Transactions** come in. You begin a transaction, execute your operations, then either **commit** (mission success) or **rollback** (abort and retreat).

Transactions ensure that if any operation fails mid-mission, the entire operation is aborted and the database returns to its pre-mission state. No partial victories, no collateral damage to data integrity.

### Basic Transaction Flow

```rust
use tank::{Connection, Entity, Transaction};

#[derive(Entity)]
struct Deployment {
    #[tank(primary_key)]
    unit_id: i32,
    location: String,
    status: String,
}

async fn execute_coordinated_strike<C: Connection>(connection: &mut C) -> Result<()> {
    // Initiate the operation
    let mut transaction = connection
        .begin()
        .await
        .expect("Failed to begin transaction");

    // Execute the mission plan
    Deployment::insert_many(
        &mut transaction,
        &[
            Deployment {
                unit_id: 1,
                location: "Sector Alpha".into(),
                status: "Active".into(),
            },
            Deployment {
                unit_id: 2,
                location: "Sector Bravo".into(),
                status: "Standby".into(),
            },
        ],
    )
    .await
    .expect("Failed to deploy units");

    // Mission accomplished—lock it in
    transaction
        .commit()
        .await
        .expect("Failed to commit transaction");

    Ok(())
}
```

### Rollback: Tactical Retreat

When things go sideways on the battlefield, you need to abort the mission and pull back. The `rollback()` method does exactly that—it cancels all operations performed within the transaction.

```rust
async fn abort_on_failure<C: Connection>(connection: &mut C) -> Result<()> {
    let mut transaction = connection
        .begin()
        .await
        .expect("Failed to begin transaction");

    // Initial strikes
    Deployment::insert_one(
        &mut transaction,
        &Deployment {
            unit_id: 99,
            location: "Hot Zone".into(),
            status: "Infiltrating".into(),
        },
    )
    .await
    .expect("Failed to insert deployment");

    // Mission compromised! Abort! Abort!
    transaction
        .rollback()
        .await
        .expect("Failed to rollback transaction");

    // All operations cancelled—database unchanged
    Ok(())
}
```

### Multi-Stage Operations

Real combat involves multiple phases. Transactions let you coordinate complex operations across different entities and stages.

```rust
#[derive(Entity)]
struct Arsenal {
    #[tank(primary_key)]
    weapon_id: String,
    ammunition: i32,
}

#[derive(Entity)]
struct Mission {
    #[tank(primary_key)]
    mission_code: String,
    weapon_assigned: String,
}

async fn complex_operation<C: Connection>(connection: &mut C) -> Result<()> {
    let mut transaction = connection.begin().await?;

    // Phase 1: Prepare the arsenal
    Arsenal::drop_table(&mut transaction, true, false).await?;
    Arsenal::create_table(&mut transaction, true, true).await?;

    Arsenal::insert_many(
        &mut transaction,
        &[
            Arsenal {
                weapon_id: "M4A1".into(),
                ammunition: 210,
            },
            Arsenal {
                weapon_id: "M249".into(),
                ammunition: 600,
            },
        ],
    )
    .await?;

    // Phase 2: Assign missions
    Mission::drop_table(&mut transaction, true, false).await?;
    Mission::create_table(&mut transaction, true, true).await?;

    Mission::insert_one(
        &mut transaction,
        &Mission {
            mission_code: "OP-THUNDER".into(),
            weapon_assigned: "M4A1".into(),
        },
    )
    .await?;

    // All phases complete—commit the entire operation
    transaction.commit().await?;

    Ok(())
}
```

## Battle Protocols

### Connection Lifecycle
- **Establish**: Call `connect()` with your database URL
- **Deploy**: Use the connection for queries, inserts, updates, and deletes
- **Maintain**: Connection pooling is handled automatically by the driver
- **Terminate**: Connections close automatically when dropped

### Transaction Discipline
- **Begin**: Start a transaction with `connection.begin()`
- **Execute**: Perform all operations on the transaction handle
- **Decide**: Either `commit()` for success or `rollback()` for abort
- **Never abandon**: Always explicitly commit or rollback—Tank will panic on drop of an uncommitted transaction to prevent silent data corruption

### Mission-Critical Rules
1. **One mission, one transaction**: Keep transaction scope focused and tight
2. **Fail fast**: Don't let errors linger—handle them immediately
3. **Never nest**: Transactions don't nest—complete one before starting another
4. **Check your six**: Always verify connection strings before deployment

## Next Objectives

Now that your supply lines are secure and you know how to coordinate multi-phase operations, it's time to define your combat units. Move out to [Field Manual Section 4 - Unit Schematics](4-entity-definition.md) to learn how to blueprint your entities and deploy them into battle.

Remember soldier: A connection without a transaction is like a rifle on semi-auto—each shot fires independently. A transaction is full-auto suppressive fire—all rounds count as one engagement. Choose your weapon wisely.

**Stay frosty. Stay connected. Tank out.**
