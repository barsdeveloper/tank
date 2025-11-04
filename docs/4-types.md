# Types
###### *Field Manual Section 4* - Payload Specs

Tank brings a full type arsenal to the field. The `Entity` derive macro identifies the type you're using by inspecting its final path segment (the "trailer"). For example, `std::collections::VecDeque`, `collections::VecDeque`, or simply `VecDeque` all resolve to the same list type.

Tank maps ordinary Rust types (numbers, strings, times, collections) to the closest column types each driver supports, falling back to generic representations when appropriate. Below is the standard mapping of Rust types to each driver's column type. `:x:` indicates no native support at this time. Collection types may be emulated in some drivers using generic JSON/text representations.

## Column Types
| Rust                       | DuckDB       | SQLite  | Postgres     | MySQL                |
| -------------------------- | ------------ | ------- | ------------ | -------------------- |
| `bool`                     | BOOLEAN      | INTEGER | BOOLEAN      | BOOLEAN              |
| `i8`                       | TINYINT      | INTEGER | SMALLINT     | TINYINT              |
| `i16`                      | SMALLINT     | INTEGER | SMALLINT     | SMALLINT             |
| `i32`                      | INTEGER      | INTEGER | INTEGER      | INTEGER              |
| `i64`                      | BIGINT       | INTEGER | BIGINT       | BIGINT               |
| `i128`                     | HUGEINT      | ❌      | NUMERIC(38)  | NUMERIC(38)          |
| `u8`                       | UTINYINT     | INTEGER | SMALLINT     | TINYINT UNSIGNED     |
| `u16`                      | USMALLINT    | INTEGER | INTEGER      | SMALLINT UNSIGNED    |
| `u32`                      | UINTEGER     | INTEGER | BIGINT       | INTEGER UNSIGNED     |
| `u64`                      | UBIGINT      | INTEGER | NUMERIC(19)  | BIGINT UNSIGNED      |
| `u128`                     | UHUGEINT     | ❌      | NUMERIC(38)  | NUMERIC(38) UNSIGNED |
| `isize`                    | BIGINT ⚠️    | INTEGER | NUMERIC(38)  | BIGINT ⚠️            |
| `usize`                    | UBIGINT ⚠️   | INTEGER | NUMERIC(38)  | BIGINT UNSIGNED ⚠️   |
| `f32`                      | FLOAT        | REAL    | REAL         | FLOAT                |
| `f64`                      | DOUBLE       | REAL    | DOUBLE       | DOUBLE               |
| `rust_decimal::Decimal`    | DECIMAL      | REAL    | NUMERIC      | NUMERIC              |
| `tank::FixedDecimal<W, S>` | DECIMAL(W,S) | REAL    | NUMERIC(W,S) | NUMERIC(W,S)         |
| `char`                     | CHAR(1)      | TEXT    | CHAR(1)      | CHAR(1)              |
| `String`                   | TEXT         | TEXT    | TEXT         | TEXT                 |
| `Box<[u8]>`                | BLOB         | BLOB    | BYTEA        | BLOB                 |
| `time::Date`               | DATE         | TEXT ⚠️ | DATE         | DATE                 |
| `time::Time`               | TIME         | TEXT ⚠️ | TIME         | TIME                 |
| `time::PrimitiveDateTime`  | TIMESTAMP    | TEXT ⚠️ | TIMESTAMP    | DATETIME             |
| `time::OffsetDateTime`     | TIMESTAMPTZ  | TEXT ⚠️ | TIMESTAMPTZ  | TIMESTAMP            |
| `std::time::Duration`      | INTERVAL     | ❌      | INTERVAL     | ❌                   |
| `time::Duration`           | INTERVAL     | ❌      | INTERVAL     | ❌                   |
| `tank::Interval`           | INTERVAL     | ❌      | INTERVAL     | ❌                   |
| `uuid::Uuid`               | UUID         | TEXT    | UUID         | CHAR(36)             |
| `[T; N]`                   | T[N]         | ❌      | T[N]         | JSON                 |
| `Vec<T>`                   | T[]          | ❌      | T[]          | JSON                 |
| `VecDeque<T>`              | T[]          | ❌      | T[]          | JSON                 |
| `LinkedList<T>`            | T[]          | ❌      | T[]          | JSON                 |
| `HashMap<K, V>`            | MAP(K,V)     | ❌      | ❌           | JSON                 |
| `BTreeMap<K, V>`           | MAP(K,V)     | ❌      | ❌           | JSON                 |

> [!WARNING]
> When a type falls back to a generic representation (e.g. `TEXT` or `JSON`), Tank encodes it predictably so equality / ordering comparisons (where meaningful) behave as expected. Advanced indexing or operator support may vary by driver.
>
> The special `size` types maps to the corresponding 64 bits integer or 32 bits integer depending on the size.

## Wrapper Types
Beyond the standard munitions listed above, Tank supports a range of wrapper types you can deploy directly in your entities. The resulting SQL type is inferred from the inner payload your wrapper carries into battle.

Supported wrappers:
- `tank::Passive<T>`: Omit on update / allow default generation on insert.
- `Option<T>`: Nullable column.
- `Box<T>`
- `Cell<T>`
- `RefCell<T>`
- `RwLock<T>`
- `Arc<T>`
- `Rc<T>`

*With this arsenal, your entities hit every target, every time.*
