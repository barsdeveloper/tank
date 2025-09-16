#![feature(box_patterns)]
#![feature(assert_matches)]

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use std::{assert_matches::assert_matches, borrow::Cow, collections::HashMap};
    use tank::{
        Entity, Expression, GenericSqlWriter, Operand, Passive, PrimaryKeyType, SqlWriter,
        TableRef, Value, expr,
    };
    use time::{Date, Month, Time};
    use uuid::Uuid;

    #[derive(Entity)]
    #[tank(schema = "company", primary_key = "id")]
    struct Employee {
        id: u32,
        #[tank(unique)]
        name: String,
        hire_date: Date,
        working_hours: Option<[Time; 2]>,
        salary: f64,
        skills: Vec<String>,
        documents: Option<Box<HashMap<String, Box<[u8]>>>>,
        #[tank(unique)]
        access: Passive<::uuid::Uuid>,
        #[tank(default = false)]
        deleted: bool,
    }
    impl Employee {
        pub fn sample() -> Self {
            let mut docs = HashMap::new();
            docs.insert(
                "contract.pdf".into(),
                vec![0x25, 0x50, 0x44, 0x46].into_boxed_slice(),
            );

            Self {
                id: 501,
                name: "Bob Smith".into(),
                hire_date: Date::from_calendar_date(2022, Month::January, 20).unwrap(),
                working_hours: Some([
                    Time::from_hms(9, 0, 0).unwrap(),
                    Time::from_hms(18, 0, 0).unwrap(),
                ]),
                salary: 75000.00,
                skills: vec!["Rust".into(), "SQL".into()],
                documents: Some(Box::new(docs)),
                access: Passive::NotSet,
                deleted: true,
            }
        }
    }
    const WRITER: GenericSqlWriter = GenericSqlWriter {};

    #[tokio::test]
    async fn test_employee() {
        assert_matches!(
            Employee::table_ref(),
            TableRef {
                name: "employee",
                schema: "company",
                alias: Cow::Borrowed(""),
            }
        );

        assert_eq!(
            Employee::primary_key_def()
                .map(|c| c.column_ref.name)
                .collect::<Vec<_>>(),
            ["id"]
        );
        let columns = Employee::columns();
        assert_eq!(columns.len(), 9);
        assert_eq!(columns[0].column_ref.name, "id");
        assert_eq!(columns[1].column_ref.name, "name");
        assert_eq!(columns[2].column_ref.name, "hire_date");
        assert_eq!(columns[3].column_ref.name, "working_hours");
        assert_eq!(columns[4].column_ref.name, "salary");
        assert_eq!(columns[5].column_ref.name, "skills");
        assert_eq!(columns[6].column_ref.name, "documents");
        assert_eq!(columns[7].column_ref.name, "access");
        assert_eq!(columns[8].column_ref.name, "deleted");
        assert_eq!(columns[0].column_ref.table, "employee");
        assert_eq!(columns[1].column_ref.table, "employee");
        assert_eq!(columns[2].column_ref.table, "employee");
        assert_eq!(columns[3].column_ref.table, "employee");
        assert_eq!(columns[4].column_ref.table, "employee");
        assert_eq!(columns[5].column_ref.table, "employee");
        assert_eq!(columns[6].column_ref.table, "employee");
        assert_eq!(columns[7].column_ref.table, "employee");
        assert_eq!(columns[8].column_ref.table, "employee");
        assert_eq!(columns[0].column_ref.schema, "company");
        assert_eq!(columns[1].column_ref.schema, "company");
        assert_eq!(columns[2].column_ref.schema, "company");
        assert_eq!(columns[3].column_ref.schema, "company");
        assert_eq!(columns[4].column_ref.schema, "company");
        assert_eq!(columns[5].column_ref.schema, "company");
        assert_eq!(columns[6].column_ref.schema, "company");
        assert_eq!(columns[7].column_ref.schema, "company");
        assert_eq!(columns[8].column_ref.schema, "company");
        assert_matches!(columns[0].value, Value::UInt32(..));
        assert_matches!(columns[1].value, Value::Varchar(..));
        assert_matches!(columns[2].value, Value::Date(..));
        assert_matches!(columns[3].value, Value::Array(_, box Value::Time(..), 2));
        assert_matches!(columns[4].value, Value::Float64(..));
        assert_matches!(columns[5].value, Value::List(_, box Value::Varchar(..)));
        assert_matches!(columns[6].value, Value::Map(..));
        assert_matches!(columns[7].value, Value::Uuid(..));
        assert_matches!(columns[8].value, Value::Boolean(..));
        assert_eq!(columns[0].nullable, false);
        assert_eq!(columns[1].nullable, false);
        assert_eq!(columns[2].nullable, false);
        assert_eq!(columns[3].nullable, true);
        assert_eq!(columns[4].nullable, false);
        assert_eq!(columns[5].nullable, false);
        assert_eq!(columns[6].nullable, true);
        assert_eq!(columns[7].nullable, false);
        assert_eq!(columns[8].nullable, false);
        assert_matches!(columns[0].default, None);
        assert_matches!(columns[1].default, None);
        assert_matches!(columns[2].default, None);
        assert_matches!(columns[3].default, None);
        assert_matches!(columns[4].default, None);
        assert_matches!(columns[5].default, None);
        assert_matches!(columns[6].default, None);
        assert_matches!(columns[7].default, None);
        let column8_default =
            columns[8].default.as_deref().unwrap() as *const dyn Expression as *const Operand;
        assert_matches!(unsafe { &*column8_default }, Operand::LitBool(false),);
        assert_eq!(columns[0].primary_key, PrimaryKeyType::PrimaryKey);
        assert_eq!(columns[1].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[2].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[3].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[4].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[5].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[6].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[7].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[8].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[0].unique, false);
        assert_eq!(columns[1].unique, true);
        assert_eq!(columns[2].unique, false);
        assert_eq!(columns[3].unique, false);
        assert_eq!(columns[4].unique, false);
        assert_eq!(columns[5].unique, false);
        assert_eq!(columns[6].unique, false);
        assert_eq!(columns[7].unique, true);
        assert_eq!(columns[8].unique, false);
        assert_eq!(columns[0].passive, false);
        assert_eq!(columns[1].passive, false);
        assert_eq!(columns[2].passive, false);
        assert_eq!(columns[3].passive, false);
        assert_eq!(columns[4].passive, false);
        assert_eq!(columns[5].passive, false);
        assert_eq!(columns[6].passive, false);
        assert_eq!(columns[7].passive, true);
        assert_eq!(columns[8].passive, false);
    }

    #[test]
    fn test_employee_create_table() {
        let mut query = String::new();
        WRITER.write_create_table::<Employee>(&mut query, false);
        assert_eq!(
            query,
            indoc! {r#"
                CREATE TABLE "company"."employee" (
                "id" UINTEGER PRIMARY KEY,
                "name" VARCHAR NOT NULL UNIQUE,
                "hire_date" DATE NOT NULL,
                "working_hours" TIME[2],
                "salary" DOUBLE NOT NULL,
                "skills" VARCHAR[] NOT NULL,
                "documents" MAP(VARCHAR,BLOB),
                "access" UUID NOT NULL UNIQUE,
                "deleted" BOOLEAN NOT NULL DEFAULT false
                );
            "#}
            .trim()
        );
    }

    #[test]
    fn test_employee_drop_table() {
        let mut query = String::new();
        WRITER.write_drop_table::<Employee>(&mut query, true);
        assert_eq!(query, r#"DROP TABLE IF EXISTS "company"."employee";"#);
    }

    #[test]
    fn test_employee_select() {
        let mut query = String::new();
        WRITER.write_select(
            &mut query,
            Employee::columns(),
            Employee::table_ref(),
            &expr!(Employee::salary > 50000),
            Some(10),
        );
        assert_eq!(
            query,
            indoc! {r#"
                SELECT "id", "name", "hire_date", "working_hours", "salary", "skills", "documents", "access", "deleted"
                FROM "company"."employee"
                WHERE "salary" > 50000
                LIMIT 10;
            "#}
            .trim()
        );
    }

    #[test]
    fn test_employee_insert() {
        let mut docs = HashMap::new();
        docs.insert("contract.pdf".to_string(), vec![1, 2, 3, 4]);
        let employee = Employee::sample();
        let mut query = String::new();
        WRITER.write_insert(&mut query, [&employee], false);
        assert_eq!(
            query,
            indoc! {r#"
                INSERT INTO "company"."employee" ("id", "name", "hire_date", "working_hours", "salary", "skills", "documents", "deleted") VALUES
                (501, 'Bob Smith', '2022-01-20', ['9:00:00.0','18:00:00.0'], 75000.0, ['Rust','SQL'], {'contract.pdf':'\x25\x50\x44\x46'}, true);
            "#}
            .trim()
        );
        let employee = Employee {
            access: Uuid::parse_str("8f8fcc51-2fa9-4118-b14f-af2d8301a89a")
                .unwrap()
                .into(),
            ..Employee::sample()
        };
        let mut query = String::new();
        WRITER.write_insert(&mut query, [&employee], false);
        assert_eq!(
            query,
            indoc! {r#"
                INSERT INTO "company"."employee" ("id", "name", "hire_date", "working_hours", "salary", "skills", "documents", "access", "deleted") VALUES
                (501, 'Bob Smith', '2022-01-20', ['9:00:00.0','18:00:00.0'], 75000.0, ['Rust','SQL'], {'contract.pdf':'\x25\x50\x44\x46'}, '8f8fcc51-2fa9-4118-b14f-af2d8301a89a', true);
            "#}
            .trim()
        );
    }

    #[test]
    fn test_sql_delete() {
        let mut query = String::new();
        WRITER.write_delete::<Employee, _>(&mut query, &expr!(Employee::name == "Bob"));
        assert_eq!(
            query,
            indoc! {r#"
                DELETE FROM "company"."employee"
                WHERE "name" = 'Bob';
            "#}
            .trim()
        );
    }
}
