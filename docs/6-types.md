# Types
###### *Field Manual Section 6* - Type Conversion Schematics

Tank brings a full type arsenal to the field. The `Entity` derive macro identifies the type you’re using by inspecting its final path segment, the “trailer.” For example, `std::collections::VecDeque`, `collections::VecDeque`, or simply `VecDeque` all resolve to the same list type. No matter how you call in your reinforcements, Tank recognizes the formation.

| Rust                       | DuckDB       | Sqlite  | Postgres     | MySQL                |
| -------------------------- | ------------ | ------- | ------------ | -------------------- |
| `bool`                     | BOOLEAN      | INTEGER | BOOLEAN      | BOOLEAN              |
| `i8`                       | TINYINT      | INTEGER | SMALLINT     | TINYINT              |
| `i16`                      | SMALLINT     | INTEGER | SMALLINT     | SMALLINT             |
| `i32`                      | INTEGER      | INTEGER | INTEGER      | INTEGER              |
| `i64`                      | BIGINT       | INTEGER | BIGINT       | BIGINT               |
| `i128`                     | HUGEINT      | :x:     | NUMERIC(38)  | NUMERIC(38)          |
| `u8`                       | UTINYINT     | INTEGER | SMALLINT     | TINYINT UNSIGNED     |
| `u16`                      | USMALLINT    | INTEGER | INTEGER      | SMALLINT UNSIGNED    |
| `u32`                      | UINTEGER     | INTEGER | BIGINT       | INTEGER UNSIGNED     |
| `u64`                      | UBIGINT      | INTEGER | NUMERIC(19)  | BIGINT UNSIGNED      |
| `u128`                     | UHUGEINT     | :x:     | NUMERIC(38)  | NUMERIC(38) UNSIGNED |
| `f32`                      | FLOAT        | REAL    | REAL         | FLOAT                |
| `f64`                      | DOUBLE       | REAL    | DOUBLE       | DOUBLE               |
| `rust_decimal::Decimal`    | DECIMAL      | REAL    | NUMERIC      | NUMERIC              |
| `tank::FixedDecimal<W, S>` | DECIMAL(W,S) | REAL    | NUMERIC(W,S) | NUMERIC(W,S)         |
| `char`                     | CHAR(1)      | TEXT    | CHAR(1)      | CHAR(1)              |
| `String`                   | TEXT         | TEXT    | TEXT         | TEXT                 |
| `Box<[u8]>`                | BLOB         | BLOB    | BYTEA        | BLOB                 |
| `time::Date`               | DATE         | TEXT    | DATE         | DATE                 |
| `time::Time`               | TIME         | TEXT    | TIME         | TIME                 |
| `time::PrimitiveDateTime`  | TIMESTAMP    | TEXT    | TIMESTAMP    | DATETIME             |
| `time::OffsetDateTime`     | TIMESTAMPTZ  | TEXT    | TIMESTAMPTZ  | TIMESTAMP            |
| `std::time::Duration`      | INTERVAL     | :x:     | INTERVAL     | :x:                  |
| `time::Duration`           | INTERVAL     | :x:     | INTERVAL     | :x:                  |
| `tank::Interval`           | INTERVAL     | :x:     | INTERVAL     | :x:                  |
| `uuid::Uuid`               | UUID         | TEXT    | UUID         | CHAR(36)             |
| `[T; N]`                   | T[N]         | :x:     | T[N]         | JSON                 |
| `Vec<T>`                   | T[]          | :x:     | T[]          | JSON                 |
| `VecDeque<T>`              | T[]          | :x:     | T[]          | JSON                 |
| `LinkedList<T>`            | T[]          | :x:     | T[]          | JSON                 |
| `HashMap<K, V>`            | MAP(K,V)     | :x:     | :x:          | JSON                 |
| `BTreeMap<K, V>`           | MAP(K,V)     | :x:     | :x:          | JSON                 |

> [!NOTE]
> If a type is not supported directly but uses the general `TEXT` type, it is generally also rendered in a way to support comparison operators like equals, less than etc.

### Wrapper values
Beyond the standard munitions listed above, Tank supports a range of wrapper types you can deploy directly in your entities. The resulting SQL type is automatically inferred from the inner payload, the value your wrapper carries into battle. Here are the supported types:
* `tank::Passive<T>`
* `Option<T>`
* `Box<T>`
* `Cell<T>`
* `RefCell<T>`
* `RwLock<T>`
* `Arc<T>`
* `Rc<T>`

*Mission Complete: With these mappings in your arsenal, your entities will never misfire on deployment.*
