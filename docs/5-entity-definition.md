# Entity Definition
###### *Field Manual Section 5* - Unit Schematics

Lock and load, soldier! In Tank's war machine, the "Entity" is your frontline fighter. A Rust struct rigged with the `#[derive(Entity)]` macro that maps straight to a database table and gives you convenient functions to access and modify the data. Tank automatically handles the heavy lifting of converting Rust values to database columns and back.

Before you can deploy entities, you must have an operational connection (see [*Field Manual Section 3 - Supply Lines*](3-connection.md#connect)). All entity operations execute through either a live connection or a locked transaction; without a supply line, your units can't move.

## Mission Briefing
Zero boilerplate. Define a struct, tag it, deploy. Tank matches your Rust field types to the closest database type for each driver. Unified arsenal, same blueprint across all battlefields.

## Entity
Start with a plain Rust struct and derive the `tank::Entity` trait. The fields can have any of the types supported (see [*Field Manual Section 4* - Type Conversion Schematics](4-types.md))
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
* `tank::Passive<T>` lets the database provide or retain a value: omit it when updating, or allow default generation on insert.
* `Option<T>` marks the column nullable.

You have now forged a battle-ready map of your database. Create, destroy, deploy new records or extract targets one-by-one. Execute every maneuver through a live connection or a locked transaction for maximum firepower.

## Attributes
Tank's `#[tank(...)]` attributes are your weapon mods, fine-tuning structs for precision strikes.
- <Badge type="tip" text="struct" /><Badge type="tip" text="field" /> `name = "the_name"`: Table name on a struct / column name on a field. **Default**: snake_case of identifier.
- <Badge type="tip" text="struct" /> `schema = "your_schema"`: Database schema. Default: none.
- <Badge type="tip" text="struct" /> `primary_key = "some_field"` or `primary_key = ("column_1", Self::column_2, ..)`: Table primary key.
- <Badge type="tip" text="field" /> `primary_key`: Marks field as part of primary key. Cannot be combined with struct-level `primary_key`.
- <Badge type="tip" text="struct" /> `unique = "some_field"` or `unique = ("column_1", Self::column_2, ..)`: Unique constraint.
- <Badge type="tip" text="field" /> `unique`: Field-level unique constraint.
- <Badge type="tip" text="field" /> `default`: Default value expression for the column.
- <Badge type="tip" text="field" /> `references = OtherEntity::column`: Foreign key reference.
- <Badge type="tip" text="field" /> `ignore`: Excludes field from table and from row materialization.
- <Badge type="tip" text="field" /> `type = "RAW_DB_TYPE"`: Override column type in DDL (driver-portability risk).

*All units accounted for. Stand by.*
