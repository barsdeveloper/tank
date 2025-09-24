#![feature(assert_matches)]
#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rust_decimal::{Decimal, prelude::FromPrimitive};
    use std::{assert_matches::assert_matches, borrow::Cow, sync::Arc, time::Duration};
    use tank::{Entity, GenericSqlWriter, PrimaryKeyType, SqlWriter, TableRef, Value, expr};

    #[derive(Entity)]
    #[tank(name = "a_table", primary_key = ("bravo", "delta"))]
    struct MyEntity {
        _alpha: Box<Box<Box<Box<Box<Box<Box<Box<Box<Box<Box<f64>>>>>>>>>>>,
        _bravo: i16,
        _charlie: Box<Box<Option<Option<Box<Box<Option<Box<rust_decimal::Decimal>>>>>>>>,
        #[tank(name = "delta")]
        _delta_duration: Duration,
        _echo: Option<Arc<rust_decimal::Decimal>>,
    }
    impl MyEntity {
        pub fn sample() -> Self {
            Self {
                _alpha: Box::new(Box::new(Box::new(Box::new(Box::new(Box::new(Box::new(
                    Box::new(Box::new(Box::new(Box::new(0.0)))),
                ))))))),
                _bravo: 2,
                _charlie: Box::new(Box::new(Some(Some(Box::new(Box::new(Some(Box::new(
                    Decimal::from_f64(10.2).unwrap(),
                )))))))),
                _delta_duration: Duration::from_secs(1),
                _echo: Some(Arc::new(Decimal::from_f64(23.44).unwrap())),
            }
        }
    }
    const WRITER: GenericSqlWriter = GenericSqlWriter {};

    #[tokio::test]
    async fn test_odd_entity() {
        assert_matches!(
            MyEntity::table_ref(),
            TableRef {
                name: "a_table",
                schema: "",
                alias: Cow::Borrowed(""),
            }
        );

        let pk: Vec<_> = MyEntity::primary_key_def().collect();
        assert_eq!(pk.len(), 2);
        assert_eq!(pk[0].nullable, false);
        assert_eq!(pk[0].primary_key, PrimaryKeyType::PartOfPrimaryKey);
        assert_eq!(pk[1].nullable, false);
        assert_eq!(pk[1].primary_key, PrimaryKeyType::PartOfPrimaryKey);

        let columns = MyEntity::columns();
        assert_eq!(columns.len(), 5);
        assert_eq!(columns[0].column_ref.name, "alpha");
        assert_eq!(columns[1].column_ref.name, "bravo");
        assert_eq!(columns[2].column_ref.name, "charlie");
        assert_eq!(columns[3].column_ref.name, "delta");
        assert_eq!(columns[4].column_ref.name, "echo");
        assert_eq!(columns[0].column_ref.table, "a_table");
        assert_eq!(columns[1].column_ref.table, "a_table");
        assert_eq!(columns[2].column_ref.table, "a_table");
        assert_eq!(columns[3].column_ref.table, "a_table");
        assert_eq!(columns[4].column_ref.table, "a_table");
        assert_eq!(columns[0].column_ref.schema, "");
        assert_eq!(columns[1].column_ref.schema, "");
        assert_eq!(columns[2].column_ref.schema, "");
        assert_eq!(columns[3].column_ref.schema, "");
        assert_eq!(columns[4].column_ref.schema, "");
        assert_matches!(columns[0].default, None);
        assert_matches!(columns[1].default, None);
        assert_matches!(columns[2].default, None);
        assert_matches!(columns[3].default, None);
        assert_matches!(columns[4].default, None);
        assert_matches!(columns[0].value, Value::Float64(..));
        assert_matches!(columns[1].value, Value::Int16(..));
        assert_matches!(columns[2].value, Value::Decimal(..));
        assert_matches!(columns[3].value, Value::Interval(..));
        assert_matches!(columns[4].value, Value::Decimal(..));
        assert_eq!(columns[0].nullable, false);
        assert_eq!(columns[1].nullable, false);
        assert_eq!(columns[2].nullable, true);
        assert_eq!(columns[3].nullable, false);
        assert_eq!(columns[4].nullable, true);
        assert_matches!(columns[0].default, None);
        assert_matches!(columns[1].default, None);
        assert_matches!(columns[2].default, None);
        assert_matches!(columns[3].default, None);
        assert_matches!(columns[4].default, None);
        assert_eq!(columns[0].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[1].primary_key, PrimaryKeyType::PartOfPrimaryKey);
        assert_eq!(columns[2].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[3].primary_key, PrimaryKeyType::PartOfPrimaryKey);
        assert_eq!(columns[4].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[0].unique, false);
        assert_eq!(columns[1].unique, false);
        assert_eq!(columns[2].unique, false);
        assert_eq!(columns[3].unique, false);
        assert_eq!(columns[4].unique, false);
        assert_eq!(columns[0].references, None);
        assert_eq!(columns[1].references, None);
        assert_eq!(columns[2].references, None);
        assert_eq!(columns[3].references, None);
        assert_eq!(columns[4].references, None);
        assert_eq!(columns[0].passive, false);
        assert_eq!(columns[1].passive, false);
        assert_eq!(columns[2].passive, false);
        assert_eq!(columns[3].passive, false);
        assert_eq!(columns[4].passive, false);
    }

    #[test]
    fn test_odd_entity_create_table() {
        let mut query = String::new();
        WRITER.write_create_table::<MyEntity, _>(&mut query, true);
        assert_eq!(
            query,
            indoc! {r#"
                CREATE TABLE IF NOT EXISTS "a_table" (
                "alpha" DOUBLE NOT NULL,
                "bravo" SMALLINT,
                "charlie" DECIMAL,
                "delta" INTERVAL,
                "echo" DECIMAL,
                PRIMARY KEY ("bravo", "delta")
                );
            "#}
            .trim()
        );
    }

    #[test]
    fn test_odd_entity_drop_table() {
        let mut query = String::new();
        WRITER.write_drop_table::<MyEntity, _>(&mut query, false);
        assert_eq!(query, r#"DROP TABLE "a_table";"#);
    }

    #[test]
    fn test_odd_entity_select() {
        let mut query = String::new();
        WRITER.write_select(
            &mut query,
            MyEntity::columns(),
            MyEntity::table_ref(),
            &expr!(MyEntity::_bravo < 0),
            Some(300),
        );
        assert_eq!(
            query,
            indoc! {r#"
                SELECT "alpha", "bravo", "charlie", "delta", "echo"
                FROM "a_table"
                WHERE "bravo" < 0
                LIMIT 300;
            "#}
            .trim()
        );
    }

    #[test]
    fn test_odd_entity_insert() {
        let mut query = String::new();
        WRITER.write_insert(&mut query, [&MyEntity::sample()], true);
        assert_eq!(
            query,
            indoc! {r#"
                INSERT INTO "a_table" ("alpha", "bravo", "charlie", "delta", "echo") VALUES
                (0.0, 2, 10.2, INTERVAL 1 SECOND, 23.44)
                ON CONFLICT ("bravo", "delta") DO UPDATE SET
                "alpha" = EXCLUDED."alpha",
                "charlie" = EXCLUDED."charlie",
                "echo" = EXCLUDED."echo";
            "#}
            .trim()
        );
    }

    #[test]
    fn test_odd_entity_delete() {
        let mut query = String::new();
        WRITER.write_delete::<MyEntity, _, _>(&mut query, &expr!(MyEntity::_echo == 5));
        assert_eq!(
            query,
            indoc! {r#"
                DELETE FROM "a_table"
                WHERE "echo" = 5;
            "#}
            .trim()
        );
    }
}
