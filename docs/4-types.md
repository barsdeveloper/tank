# Types
###### *Field Manual Section 4* - Type Conversion Schematics
Tank brings a full type arsenal to the field. The `Entity` derive macro identifies the type you’re using by inspecting its final path segment, the “trailer.” For example, `std::collections::VecDeque`, `collections::VecDeque`, or simply `VecDeque` all resolve to the same list type. No matter how you call in your reinforcements, Tank recognizes the formation.

| Rust                       | DuckDB       | Sqlite  | Postgres     |
| -------------------------- | ------------ | ------- | ------------ |
| `bool`                     | BOOLEAN      | INTEGER | BOOLEAN      |
| `i8`                       | TINYINT      | INTEGER | SMALLINT     |
| `i16`                      | SMALLINT     | INTEGER | SMALLINT     |
| `i32`                      | INTEGER      | INTEGER | INTEGER      |
| `i64`                      | BIGINT       | INTEGER | BIGINT       |
| `i128`                     | HUGEINT      | :x:     | NUMERIC(38)  |
| `u8`                       | UTINYINT     | INTEGER | SMALLINT     |
| `u16`                      | USMALLINT    | INTEGER | INTEGER      |
| `u32`                      | UINTEGER     | INTEGER | BIGINT       |
| `u64`                      | UBIGINT      | INTEGER | NUMERIC(19)  |
| `u128`                     | UHUGEINT     | :x:     | NUMERIC(38)  |
| `f32`                      | FLOAT        | REAL    | REAL         |
| `f64`                      | DOUBLE       | REAL    | DOUBLE       |
| `rust_decimal::Decimal`    | DECIMAL      | REAL    | NUMERIC      |
| `tank::FixedDecimal<W, S>` | DECIMAL(W,S) | REAL    | NUMERIC(W,S) |
| `char`                     | CHAR(1)      | TEXT    | CHARACTER(1) |
| `String`                   | TEXT         | TEXT    | TEXT         |
| `Box<[u8]>`                | BLOB         | BLOB    | BYTEA        |
| `time::Date`               | DATE         | TEXT    | DATE         |
| `time::Time`               | TIME         | TEXT    | TIME         |
| `time::PrimitiveDateTime`  | TIMESTAMP    | TEXT    | TIMESTAMP    |
| `time::OffsetDateTime`     | TIMESTAMPTZ  | TEXT    | TIMESTAMPTZ  |
| `std::time::Duration`      | INTERVAL     | :x:     | INTERVAL     |
| `time::Duration`           | INTERVAL     | :x:     | INTERVAL     |
| `tank::Interval`           | INTERVAL     | :x:     | INTERVAL     |
| `uuid::Uuid`               | UUID         | TEXT    | UUID         |
| `[T; N]`                   | T[N]         | :x:     | T[N]         |
| `Vec<T>`                   | T[]          | :x:     | T[]          |
| `VecDeque<T>`              | T[]          | :x:     | T[]          |
| `LinkedList<T>`            | T[]          | :x:     | T[]          |
| `HashMap<K, V>`            | MAP(K,V)     | :x:     | :x:          |
| `BTreeMap<K, V>`           | MAP(K,V)     | :x:     | :x:          |

> [!NOTE]
> If a type is not supported directly but uses the general `TEXT` type, it is generally also rendered in a way to support comparison operators like equals, less then etc.

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
