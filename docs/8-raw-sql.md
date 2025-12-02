# Raw SQL
###### *Field Manual Section 8* - Precision Fire

Sometimes you need to drop the abstractions and put steel directly on target. Tank lets you fire raw SQL or multi‑statement (if supported) batches while still decoding rows into typed entities. This section covers: building raw statements, executing mixed result streams, and converting rows back into your structs.

## Entry Points
Three firing modes:
- `executor.run(sql)`: Streams a mix of `QueryResult::{Row, Affected}` for all statements contained in SQL. Not all drivers support multiple statements in the same query.
- `executor.fetch(sql)`: Convenience method to extract only rows (skips inspecting affected counts)
- `executor.execute(sql)`: Damage report only. Aggregates all `RowsAffected` counts across the batch and returns a single total. Rows (if any) are discarded.

## Composing SQL With `SqlWriter`
Every driver exposes a `SqlWriter` that produces dialect-correct fragments. You can concatenate multiple statements into one `String` and then fire them in one go. Writers append the necessary separators (`;`) at the end of every statement.

Example building 8 statements (one *CREATE SCHEMA* (as part of the *CREATE TABLE*), two *CREATE TABLE*, three *INSERT INTO* and two *SELECT*):
```rust
let writer = executor.driver().sql_writer();
let mut sql = String::new();
writer.write_create_table::<One>(&mut sql, true);
writer.write_create_table::<Two>(&mut sql, false);
writer.write_insert(&mut sql, &[One { string: "ddd".into() }, One { string: "ccc".into() }], false);
writer.write_insert(&mut sql, &[Two { a2: 21, string: "aaa".into() }, Two { a2: 22, string: "bbb".into() }], false);
writer.write_insert(&mut sql, &[One { a1: 11, string: "zzz".into(), c1: 512 }], false);
writer.write_select(&mut sql, [One::a1, One::string, One::c1], One::table(), &true, None);
writer.write_select(&mut sql, Two::columns(), Two::table(), &true, None);
// Fire the batch
let results = executor.run(sql).try_collect::<Vec<_>>().await?;
```

### Mixed Results
In a composite batch, each statement yields either an `Affected` count or one or more `Row` values. Aggregate the stream, then filter as needed:
```rust
let mut rows = results
    .into_iter()
    .filter_map(|r| if let QueryResult::Row(row) = r { Some(row) } else { None })
    .collect::<Vec<_>>();
```

## Decoding Rows Into Entities
Raw result rows carry column labels. Any type with `#[derive(Entity)]` can reconstruct itself using `Entity::from_row(row)`. The labels must align with the field mapping (renames via `#[tank(name = "...")]` are respected). Missing or mismatched labels produce an error.
```rust
#[derive(Entity)]
struct Two { a2: u32, string: String }
// After collecting rows
let entity = Two::from_row(row)?; // Strongly typed reconstruction
```

You can interleave custom decoding logic for ad-hoc structs defined inline, useful when projecting reduced column sets:
```rust
#[derive(Entity)]
struct Projection { callsign: String, strength: i8 }
let (callsign, strength) = Projection::from_row(row)?.and_then(|p| Ok((p.callsign, p.strength)))?;
```

*Raw fire authorized. Execute with precision. Tank out.*
