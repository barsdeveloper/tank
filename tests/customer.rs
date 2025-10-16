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
            Customer::table(),
            TableRef {
                name: "customers",
                schema: "",
                alias: Cow::Borrowed(""),
            }
        ));

        assert_eq!(Customer::primary_key_def().len(), 0);
        let columns = Customer::columns();
        assert_eq!(columns.len(), 5);
        assert_eq!(columns[0].column_ref.name, "transaction_ids");
        assert_eq!(columns[1].column_ref.name, "settings");
        assert_eq!(columns[2].column_ref.name, "values");
        assert_eq!(columns[3].column_ref.name, "signup_duration");
        assert_eq!(columns[4].column_ref.name, "recent_purchases");
        assert_eq!(columns[0].column_ref.table, "customers");
        assert_eq!(columns[1].column_ref.table, "customers");
        assert_eq!(columns[2].column_ref.table, "customers");
        assert_eq!(columns[3].column_ref.table, "customers");
        assert_eq!(columns[4].column_ref.table, "customers");
        assert_eq!(columns[0].column_ref.schema, "");
        assert_eq!(columns[1].column_ref.schema, "");
        assert_eq!(columns[2].column_ref.schema, "");
        assert_eq!(columns[3].column_ref.schema, "");
        assert_eq!(columns[4].column_ref.schema, "");
        assert!(matches!(
            columns[0].value,
            Value::List(_, ref ty) if matches!(**ty, Value::UInt64(..))
        ));
        assert!(matches!(
            columns[1].value,
            Value::List(.., ref ty) if matches!(**ty, Value::Varchar(..))
        ));
        assert!(matches!(
            columns[2].value,
            Value::List(.., ref ty) if matches!(**ty, Value::Float32(..))
        ));
        assert!(matches!(columns[3].value, Value::Interval(..)));
        assert!(matches!(
            columns[4].value,
            Value::List(.., ref ty) if matches!(**ty, Value::List(.., ref ty) if matches!(**ty, Value::Int64(..))),
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
        assert_eq!(columns[0].references, None);
        assert_eq!(columns[1].references, None);
        assert_eq!(columns[2].references, None);
        assert_eq!(columns[3].references, None);
        assert_eq!(columns[4].references, None);
        assert_eq!(columns[0].on_delete, None);
        assert_eq!(columns[1].on_delete, None);
        assert_eq!(columns[2].on_delete, None);
        assert_eq!(columns[3].on_delete, None);
        assert_eq!(columns[4].on_delete, None);
        assert_eq!(columns[0].on_update, None);
        assert_eq!(columns[1].on_update, None);
        assert_eq!(columns[2].on_update, None);
        assert_eq!(columns[3].on_update, None);
        assert_eq!(columns[4].on_update, None);
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
            indoc! {r#"
                CREATE TABLE "customers" (
                "transaction_ids" UBIGINT[] NOT NULL,
                "settings" VARCHAR[] DEFAULT ['discount', 'newsletter'],
                "values" FLOAT[],
                "signup_duration" INTERVAL NOT NULL,
                "recent_purchases" BIGINT[][]);
                COMMENT ON COLUMN "customers"."recent_purchases" IS 'List of all the full cart products\nIt''s a list of lists of ids\n\nCan also be empty';
            "#}
            .trim()
        );
    }

    #[test]
    fn test_customer_drop_table() {
        let mut query = String::new();
        WRITER.write_drop_table::<Customer>(&mut query, false);
        assert_eq!(query, r#"DROP TABLE "customers";"#);
    }

    #[test]
    fn test_customer_select() {
        let mut query = String::new();
        WRITER.write_select(
            &mut query,
            Customer::columns(),
            Customer::table(),
            &expr!(len(Customer::_values) > 10),
            Some(10),
        );
        assert_eq!(
            query,
            indoc! {r#"
                SELECT "transaction_ids", "settings", "values", "signup_duration", "recent_purchases"
                FROM "customers"
                WHERE len("values") > 10
                LIMIT 10;
            "#}
            .trim()
        );
    }

    #[test]
    fn test_customer_delete() {
        let mut query = String::new();
        WRITER.write_delete::<Customer, _>(&mut query, &expr!(true));
        assert_eq!(
            query,
            indoc! {r#"
                DELETE FROM "customers"
                WHERE true;
            "#}
            .trim()
        );
    }
}
