# Tank
Tank (Table Abstraction & Navigation Kit): the Rust data layer.

It's a simple and flexible ORM that allows to manage in a unified way data from different sources.

## Design goals
- Simple workflow, no hidden queries
- Extensible design to implement additional drivers
- Supports not just SQL
- Many data types and automatic conversions
- Custom queries and column extraction

## Getting started
1) Declara a entity
```Rust
#[derive(Entity)]
struct Tank {
}
```
