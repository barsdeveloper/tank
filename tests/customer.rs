#![feature(box_patterns)]

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use std::borrow::Cow;
    use tank::{
        Entity, Expression, GenericSqlWriter, Operand, PrimaryKeyType, SqlWriter, TableRef, Value,
        expr,
    };

    #[derive(Entity)]
    #[tank(name = "customers")]
    struct Customer {
        _transaction_ids: Vec<u64>,
        #[tank(default = ["discount", "newsletter"], name = "settings")]
        _preferences: Option<Vec<String>>,
        _values: Box<Option<Vec<f32>>>,
        _signup_duration: std::time::Duration,
        /// List of all the full cart products
        /// It's a list of lists of ids
        ///
        /// Can also be empty
        _recent_purchases: Option<Vec<Option<Box<Vec<i64>>>>>,
    }
    const WRITER: GenericSqlWriter = GenericSqlWriter {};

    #[tokio::test]
    async fn test_customer() {
        assert!(matches!(
            Customer::table_ref(),
            TableRef {
                name: "customers",
                schema: "",
                alias: Cow::Borrowed(""),
            }
        ));

        assert_eq!(Customer::primary_key_def().len(), 0);

        let columns = Customer::columns_def();
        assert_eq!(columns.len(), 5);
        assert_eq!(columns[0].reference.name, "transaction_ids");
        assert_eq!(columns[1].reference.name, "settings");
        assert_eq!(columns[2].reference.name, "values");
        assert_eq!(columns[3].reference.name, "signup_duration");
        assert_eq!(columns[4].reference.name, "recent_purchases");
        assert_eq!(columns[0].reference.table, "customers");
        assert_eq!(columns[1].reference.table, "customers");
        assert_eq!(columns[2].reference.table, "customers");
        assert_eq!(columns[3].reference.table, "customers");
        assert_eq!(columns[4].reference.table, "customers");
        assert_eq!(columns[0].reference.schema, "");
        assert_eq!(columns[1].reference.schema, "");
        assert_eq!(columns[2].reference.schema, "");
        assert_eq!(columns[3].reference.schema, "");
        assert_eq!(columns[4].reference.schema, "");
        assert!(matches!(
            columns[0].value,
            Value::List(_, box Value::UInt64(..))
        ));
        assert!(matches!(
            columns[1].value,
            Value::List(.., box Value::Varchar(..))
        ));
        assert!(matches!(
            columns[2].value,
            Value::List(.., box Value::Float32(..))
        ));
        assert!(matches!(columns[3].value, Value::Interval(..)));
        assert!(matches!(
            columns[4].value,
            Value::List(.., box Value::List(.., box Value::Int64(..)))
        ));
        assert_eq!(columns[0].nullable, false);
        assert_eq!(columns[1].nullable, true);
        assert_eq!(columns[2].nullable, true);
        assert_eq!(columns[3].nullable, false);
        assert_eq!(columns[4].nullable, true);
        assert!(matches!(columns[0].default, None));
        let column1_default =
            columns[1].default.as_deref().unwrap() as *const dyn Expression as *const Operand;
        assert!(matches!(
            unsafe { &*column1_default },
            Operand::LitArray([Operand::LitStr("discount"), Operand::LitStr("newsletter"),])
        ));
        assert!(matches!(columns[2].default, None));
        assert!(matches!(columns[3].default, None));
        assert!(matches!(columns[4].default, None));
        assert_eq!(columns[0].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[1].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[2].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[3].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[4].primary_key, PrimaryKeyType::None);
        assert_eq!(columns[0].unique, false);
        assert_eq!(columns[1].unique, false);
        assert_eq!(columns[2].unique, false);
        assert_eq!(columns[3].unique, false);
        assert_eq!(columns[4].unique, false);
        assert_eq!(columns[0].passive, false);
        assert_eq!(columns[1].passive, false);
        assert_eq!(columns[2].passive, false);
        assert_eq!(columns[3].passive, false);
        assert_eq!(columns[4].passive, false);
        assert_eq!(columns[0].comment, "");
        assert_eq!(columns[1].comment, "");
        assert_eq!(columns[2].comment, "");
        assert_eq!(columns[3].comment, "");
        assert_eq!(
            columns[4].comment,
            indoc! {"
                List of all the full cart products
                It's a list of lists of ids

                Can also be empty
            "}
            .trim()
        );
    }

    #[test]
    fn test_customer_create_table() {
        let mut query = String::new();
        WRITER.write_create_table::<Customer>(&mut query, false);
        assert_eq!(
            query,
            indoc! {"
                CREATE TABLE customers (
                transaction_ids UBIGINT[] NOT NULL,
                settings VARCHAR[] DEFAULT ['discount', 'newsletter'],
                values FLOAT[],
                signup_duration INTERVAL NOT NULL,
                recent_purchases BIGINT[][] COMMENT 'List of all the full cart products\nIt''s a list of lists of ids\n\nCan also be empty'
                )
            "}
            .trim()
        );
    }

    #[test]
    fn test_customer_drop_table() {
        let mut query = String::new();
        WRITER.write_drop_table::<Customer>(&mut query, false);
        assert_eq!(query, "DROP TABLE customers");
    }

    #[test]
    fn test_customer_select() {
        let mut query = String::new();
        WRITER.write_select::<Customer, _, _>(
            &mut query,
            Customer::table_ref(),
            &expr!(len(Customer::_values) > 10),
            Some(10),
        );
        assert_eq!(
            query,
            indoc! {"
                SELECT transaction_ids, settings, values, signup_duration, recent_purchases
                FROM customers
                WHERE len(values) > 10
                LIMIT 10
            "}
            .trim()
        );
    }

    #[test]
    fn test_customer_delete() {
        let mut query = String::new();
        WRITER.write_delete::<Customer, _>(&mut query, &expr!(true));
        assert_eq!(
            query,
            indoc! {"
                DELETE FROM customers
                WHERE true
            "}
            .trim()
        );
    }
}
