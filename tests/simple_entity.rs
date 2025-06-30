#[cfg(test)]
mod tests {
    use indoc::indoc;
    use std::borrow::Cow;
    use tank::{expr, Entity, GenericSqlWriter, PrimaryKeyType, SqlWriter, TableRef, Value};

    #[derive(Entity)]
    #[tank(name = "simple_entity")]
    struct SomeSimpleEntity {
        a: i8,
        b: Option<String>,
    }
    impl SomeSimpleEntity {
        pub fn make_some() -> Self {
            Self {
                a: 40,
                b: Some("hello".into()),
            }
        }
    }
    const WRITER: GenericSqlWriter = GenericSqlWriter {};

    #[tokio::test]
    async fn test_simple_entity() {
        assert!(matches!(
            SomeSimpleEntity::table_ref(),
            TableRef {
                name: "simple_entity",
                schema: "",
                alias: Cow::Borrowed(""),
            }
        ));

        assert_eq!(SomeSimpleEntity::primary_key_def().len(), 0);

        let columns = SomeSimpleEntity::columns_def();
        assert_eq!(columns.len(), 2);
        assert!(matches!(columns[0].value, Value::Int8(..)));
        assert!(matches!(columns[1].value, Value::Varchar(..)));
        assert_eq!(columns[0].nullable, false);
        assert_eq!(columns[1].nullable, true);
        assert!(matches!(columns[0].default, None));
        assert!(matches!(columns[1].default, None));
        assert_eq!(columns[0].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[1].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[0].unique, false);
        assert_eq!(columns[1].unique, false);
        assert_eq!(columns[0].auto_increment, false);
        assert_eq!(columns[1].auto_increment, false);
        assert_eq!(columns[0].passive, false);
        assert_eq!(columns[1].passive, false);
    }

    #[test]
    fn test_simple_entity_create_table() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_create_table::<SomeSimpleEntity>(&mut query, false),
            indoc! {"
                CREATE TABLE simple_entity (
                a TINYINT NOT NULL,
                b VARCHAR
                )
            "}
            .trim()
        );
    }

    #[test]
    fn test_simple_entity_drop_table() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_drop_table::<SomeSimpleEntity>(&mut query, true),
            "DROP TABLE IF EXISTS simple_entity"
        );
    }

    #[test]
    fn test_simple_entity_select() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_select::<SomeSimpleEntity, _, _>(
                &mut query,
                SomeSimpleEntity::table_ref(),
                &expr!(SomeSimpleEntity::a > 100),
                Some(1000),
            ),
            indoc! {"
                SELECT a, b
                FROM simple_entity
                WHERE a > 100
                LIMIT 1000
            "}
            .trim()
        );
    }

    #[test]
    fn test_simple_entity_insert() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_insert::<SomeSimpleEntity>(&mut query, &SomeSimpleEntity::make_some(), true),
            indoc! {"
                INSERT OR REPLACE INTO simple_entity (a, b)
                VALUES (40, 'hello')
            "}
            .trim()
        );
    }

    #[test]
    fn test_simple_entity_delete() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_delete::<SomeSimpleEntity, _>(
                &mut query,
                &expr!(SomeSimpleEntity::b != "hello")
            ),
            indoc! {"
                DELETE FROM simple_entity
                WHERE b != 'hello'
            "}
            .trim()
        );
    }
}
