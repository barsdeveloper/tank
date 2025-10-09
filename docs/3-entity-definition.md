# Entity Definition
###### *Field Manual Section 3* - Units Schematics
Lock and load, soldier! In Tank's war machine, the "Entity" is your frontline fighter. A Rust struct rigged with the `#[derive(Entity)]` macro that maps straight to a database table. Tank automatically handles the heavy lifting of converting Rust values to database columns and back.

## Mission Briefing
Zero boilerplate. Define a struct, tag it, and deploy. Tank matches your Rust field types to the closest database type for each driver. Unified arsenal. Same blueprint works across all battlefields.

## Entity
Start with a plain Rust struct and derive the `tank::Entity` trait.
```rust
#[derive(Entity)]
#[tank(schema = "ops", name = "missions", primary_key = (Self::code_name, Self::start_time))]
pub struct Mission {
    pub code_name: String,
    pub start_time: Passive<PrimitiveDateTime>,
    #[tank(references = armory.weapons(serial_number))]
    pub primary_weapon: Option<i64>,
    pub objectives: Vec<String>,
    pub success_rate: f32,
    pub casualties: Option<u16>,
}
```
*Notes:*
* `tank::Passive<..>` allows to update a entity without setting a specific field, or let the database set the default value.
* `Option<..>` specifies that the field is nullable.

You have now forged a battle-ready map of your database. Create, destroy,  deploy new records or extract targets one-by-one. Execute every manouvre with the support of a live connection or a locked-in transaction for maximum firepower.

## Values
Tank brings a full type arsenal to the field. The `Entity` derive macro identifies the type you’re using by inspecting its final path segment, the “trailer.” For example, `std::collections::VecDeque`, `collections::VecDeque`, or simply `VecDeque` all resolve to the same list type. No matter how you call in your reinforcements, Tank recognizes the formation.

| Rust                               | DuckDB                   | Sqlite  | Postgres                 |
| ---------------------------------- | ------------------------ | ------- | ------------------------ |
| `bool`                             | BOOLEAN                  | INTEGER | BOOLEAN                  |
| `i8`                               | TINYINT                  | INTEGER | SMALLINT                 |
| `i16`                              | SMALLINT                 | INTEGER | SMALLINT                 |
| `i32`                              | INTEGER                  | INTEGER | INTEGER                  |
| `i64`                              | BIGINT                   | INTEGER | BIGINT                   |
| `i128`                             | HUGEINT                  | :x:     | NUMERIC(38)              |
| `u8`                               | UTINYINT                 | INTEGER | SMALLINT                 |
| `u16`                              | USMALLINT                | INTEGER | INTEGER                  |
| `u32`                              | UINTEGER                 | INTEGER | BIGINT                   |
| `u64`                              | UBIGINT                  | INTEGER | NUMERIC(19)              |
| `u128`                             | UHUGEINT                 | :x:     | NUMERIC(38)              |
| `f32`                              | FLOAT                    | REAL    | REAL                     |
| `f64`                              | DOUBLE                   | REAL    | DOUBLE                   |
| `rust_decimal::Decimal`            | DECIMAL                  | REAL    | NUMERIC                  |
| `tank::FixedDecimal<W, S>`         | DECIMAL(W,S)             | REAL    | NUMERIC(W,S)             |
| `char`                             | CHAR(1)                  | TEXT    | CHARACTER(1)             |
| `String`                           | TEXT                     | TEXT    | TEXT                     |
| `Box<[u8]>`                        | BLOB                     | BLOB    | BYTEA                    |
| `time::Date`                       | DATE                     | TEXT    | DATE                     |
| `time::Time`                       | TIME                     | TEXT    | TIME                     |
| `time::PrimitiveDateTime`          | TIMESTAMP                | TEXT    | TIMESTAMP                |
| `time::OffsetDateTime`             | TIMESTAMP WITH TIME ZONE | TEXT    | TIMESTAMP WITH TIME ZONE |
| `std::time::Duration`              | INTERVAL                 | :x:     | INTERVAL                 |
| `time::Duration`                   | INTERVAL                 | :x:     | INTERVAL                 |
| `tank::Interval`                   | INTERVAL                 | :x:     | INTERVAL                 |
| `uuid::Uuid`                       | UUID                     | TEXT    | UUID                     |
| `[T; N]`                           | T[N]                     | :x:     | T[N]                     |
| `std::vec::Vec`                    | T[]                      | :x:     | T[]                      |
| `std::collections::VecDeque<T>`    | T[]                      | :x:     | T[]                      |
| `std::collections::LinkedList<T>`  | T[]                      | :x:     | T[]                      |
| `std::collections::HashMap<K, V>`  | MAP(K,V)                 | :x:     | :x:                      |
| `std::collections::BTreeMap<K, V>` | MAP(K,V)                 | :x:     | :x:                      |

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

## Attributes
Tank's `#[tank(...)]` attributes are your weapon mods, fine-tuning your structs for precision strikes in the database.
- <Badge type="tip" text="struct" /><Badge type="tip" text="field" /> `name = "the_name"` specifies the table name on a struct and the column name on a field. **Default**: snake_case formatted name.
- <Badge type="tip" text="struct" /> `schema = "your_schema"` sets the database schema. Default: no schema.
- <Badge type="tip" text="struct" /> `primary_key = "some_field"` or `primary_key = ("column_1", Self::column_2, ..)` specify the the table primary key.
- <Badge type="tip" text="field" /> `primary_key` defines the field as primary key. Cannot be used in combination with struct level primary_key.
- <Badge type="tip" text="struct" /> `unique = "some_field"` or `unique = ("column_1", Self::column_2, ..)` define a unique constraint.
- <Badge type="tip" text="field" /> `unique` defines a unique constraint.
- <Badge type="tip" text="field" /> `default` specifies the default value for the column.
- <Badge type="tip" text="field" /> `references = OtherEntity::column` specifies the default value for the column.
- <Badge type="tip" text="field" /> `ignore` ignores the field: it will not be part of the table, nor it will be populated from the database.
- <Badge type="tip" text="field" /> `type` overrides the type of the column for the create table query. It does not play nice if you use the entity with multiple drivers.
