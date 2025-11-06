<div align="center">
    <img width="300" height="300" src="../docs/public/logo.png" alt="Tank: Table Abstraction & Navigation Kit logo featuring a green tank with a gear background and stacked database cylinders" />
</div>

# tank-postgres

Postgres driver implementation for [Tank](https://crates.io/crates/tank): the Rust data layer.

Implements Tank’s `Driver` and related traits for Postgres, mapping Tank operations and queries into direct Postgres commands. It does not replace the main [`tank`](https://crates.io/crates/tank) crate. you still use it to define entities, manage schemas, and build queries.

https://barsdeveloper.github.io/tank/

https://github.com/barsdeveloper/tank ⭐

https://crates.io/crates/tank

## Overview
Features:
- Async connection & execution via `tokio-postgres`
- Rich type mapping (including `uuid`, `interval`, `time`, `rust_decimal`)

## Install
```sh
cargo add tank
cargo add tank-postgres
```

## Quick Start
```rust
use tank_postgres::PostgresConnection;

let connection = PostgresConnection::connect("postgres://user:pass@localhost:5432/dbname".into()).await?;
```

## Running Tests
Tests need a Postgres instance. Provide a connection URL via `TANK_POSTGRES_TEST`. If absent, a containerized PostgreSQL will be launched automatically using [testcontainers-modules](https://crates.io/crates/testcontainers-modules).

1. Ensure Docker is running:
```sh
systemctl status docker
```
2. (Linux) Add your user to the `docker` group if needed.
```sh
sudo usermod -aG docker $USER
```

> [!CAUTION]
> Avoid aborting tests mid‑run (e.g. killing the process at a breakpoint). Containers might be left running and consume resources.

> List containers:
> ```sh
> docker ps
> ```
> Stop container:
> ```sh
> docker kill <container_id_or_name>
> ```
