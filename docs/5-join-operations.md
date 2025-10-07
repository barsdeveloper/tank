# Join Operations
###### *Field Manual Section 5* – Tactical Coordination
In the field, isolated units rarely win the battle. Coordination is key. Joins let you link data across tables like synchronized squads advancing under fire.
In Tank, a join is a first class `DataSet`, just like a `TableRef`. That means you can call `select()` and then, filter, map, reduce, etc, using the same clean, composable [Stream API](https://docs.rs/futures/latest/futures/prelude/trait.Stream.html) you already know.

