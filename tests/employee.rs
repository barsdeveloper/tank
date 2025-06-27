#![feature(box_patterns)]

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use std::{borrow::Cow, collections::HashMap};
    use tank::{expr, Entity, GenericSqlWriter, PrimaryKeyType, SqlWriter, TableRef, Value};
    use time::{Date, Month, Time};

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
            }
        }
    }
    const WRITER: GenericSqlWriter = GenericSqlWriter {};

    #[tokio::test]
    async fn test_employee() {
        assert!(matches!(
            Employee::table_ref(),
            TableRef {
                name: "employee",
                schema: "company",
                alias: Cow::Borrowed(""),
            }
        ));

        assert_eq!(Employee::primary_key_def().len(), 1);
        let columns = Employee::columns_def();
        assert_eq!(columns.len(), 7);
        assert!(matches!(columns[0].value, Value::UInt32(..)));
        assert!(matches!(columns[1].value, Value::Varchar(..)));
        assert!(matches!(columns[2].value, Value::Date(..)));
        assert!(matches!(
            columns[3].value,
            Value::Array(_, box Value::Time(..), 2)
        ));
        assert!(matches!(columns[5].value, Value::List(..)));
        assert!(matches!(columns[6].value, Value::Map(..)));
        assert_eq!(columns[0].nullable, false);
        assert_eq!(columns[1].nullable, false);
        assert_eq!(columns[2].nullable, false);
        assert_eq!(columns[3].nullable, true);
        assert_eq!(columns[4].nullable, false);
        assert_eq!(columns[5].nullable, false);
        assert!(matches!(columns[0].default, None));
        assert!(matches!(columns[1].default, None));
        assert!(matches!(columns[2].default, None));
        assert!(matches!(columns[3].default, None));
        assert!(matches!(columns[4].default, None));
        assert!(matches!(columns[5].default, None));
        assert_eq!(columns[0].primary_key, PrimaryKeyType::PrimaryKey);
        assert_eq!(columns[1].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[2].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[3].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[4].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[5].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[0].unique, false);
        assert_eq!(columns[1].unique, true);
        assert_eq!(columns[2].unique, false);
        assert_eq!(columns[3].unique, false);
        assert_eq!(columns[4].unique, false);
        assert_eq!(columns[5].unique, false);
        assert_eq!(columns[0].auto_increment, false);
        assert_eq!(columns[1].auto_increment, false);
        assert_eq!(columns[2].auto_increment, false);
        assert_eq!(columns[3].auto_increment, false);
        assert_eq!(columns[4].auto_increment, false);
        assert_eq!(columns[5].auto_increment, false);
        assert_eq!(columns[0].passive, false);
        assert_eq!(columns[1].passive, false);
        assert_eq!(columns[2].passive, false);
        assert_eq!(columns[3].passive, false);
        assert_eq!(columns[4].passive, false);
        assert_eq!(columns[5].passive, false);
    }

    #[test]
    fn test_employee_create_table() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_create_table::<Employee>(&mut query, false),
            indoc! {"
                CREATE TABLE company.employee (
                id UINTEGER PRIMARY KEY,
                name VARCHAR NOT NULL UNIQUE,
                hire_date DATE NOT NULL,
                working_hours TIME[2],
                salary DOUBLE NOT NULL,
                skills VARCHAR[] NOT NULL,
                documents MAP(VARCHAR, BLOB)
                )
            "}
            .trim()
        );
    }

    #[test]
    fn test_employee_drop_table() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_drop_table::<Employee>(&mut query, true),
            "DROP TABLE IF EXISTS company.employee"
        );
    }

    #[test]
    fn test_employee_select() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_select::<Employee, _, _>(
                &mut query,
                Employee::table_ref(),
                &expr!(Employee::salary > 50000),
                Some(10),
            ),
            indoc! {"
                SELECT id, name, hire_date, working_hours, salary, skills, documents
                FROM company.employee
                WHERE salary > 50000
                LIMIT 10
            "}
            .trim()
        );
    }

    #[test]
    fn test_employee_insert() {
        let mut query = String::new();
        let mut docs = HashMap::new();
        docs.insert("contract.pdf".to_string(), vec![1, 2, 3, 4]);
        let employee = Employee::sample();
        assert_eq!(
        WRITER.sql_insert::<Employee>(&mut query, &employee, false),
        indoc! {"
            INSERT INTO company.employee (id, name, hire_date, working_hours, salary, skills, documents)
            VALUES (501, 'Bob Smith', '2022-01-20', ['9:00:00.0','18:00:00.0'], 75000, ['Rust','SQL'], {'contract.pdf':'\\x25\\x50\\x44\\x46'})
        "}
        .trim()
    );
    }

    #[test]
    fn test_sql_delete() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_delete::<Employee, _>(&mut query, &expr!(Employee::name == "Bob")),
            indoc! {"
                DELETE FROM company.employee
                WHERE name = 'Bob'
            "}
            .trim()
        );
    }
}
