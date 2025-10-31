# Entity Definition
###### *Field Manual Section 4* - Unit Schematics
Lock and load, soldier! In Tank's war machine, the "Entity" is your frontline fighter. A Rust struct rigged with the `#[derive(Entity)]` macro that maps straight to a database table and gives you convenient functions to access and modify the data. Tank automatically handles the heavy lifting of converting Rust values to database columns and back.

## Mission Briefing
Zero boilerplate. Define a struct, tag it, and deploy. Tank matches your Rust field types to the closest database type for each driver. Unified arsenal, same blueprint that works across all battlefields.

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

You have now forged a battle-ready map of your database. Create, destroy, deploy new records or extract targets one-by-one. Execute every maneuver with the support of a live connection or a locked-in transaction for maximum firepower.

## Attributes
Tank's `#[tank(...)]` attributes are your weapon mods, fine-tuning your structs for precision strikes in the database.
- <Badge type="tip" text="struct" /><Badge type="tip" text="field" /> `name = "the_name"` specifies the table name on a struct and the column name on a field. **Default**: snake_case formatted name.
- <Badge type="tip" text="struct" /> `schema = "your_schema"` sets the database schema. Default: no schema.
- <Badge type="tip" text="struct" /> `primary_key = "some_field"` or `primary_key = ("column_1", Self::column_2, ..)` specify the table primary key.
- <Badge type="tip" text="field" /> `primary_key` defines the field as primary key. Cannot be used in combination with struct level primary_key.
- <Badge type="tip" text="struct" /> `unique = "some_field"` or `unique = ("column_1", Self::column_2, ..)` define a unique constraint.
- <Badge type="tip" text="field" /> `unique` defines a unique constraint.
- <Badge type="tip" text="field" /> `default` specifies the default value for the column.
- <Badge type="tip" text="field" /> `references = OtherEntity::column` specifies a foreign key reference to another table.
- <Badge type="tip" text="field" /> `ignore` ignores the field: it will not be part of the table, nor it will be populated from the database.
- <Badge type="tip" text="field" /> `type` overrides the type of the column for the create table query. It does not play nice if you use the entity with multiple drivers.
