#![feature(box_patterns)]

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use std::{any::Any, borrow::Cow};
    use tank::{
        expr, Entity, Expression, GenericSqlWriter, Operand, PrimaryKeyType, SqlWriter, TableRef,
        Value,
    };

    #[derive(Entity)]
    #[tank(name = "customers")]
    struct Customer {
        _transaction_ids: Vec<u64>,
        #[tank(default = ["discount", "newsletter"], name = "settings")]
        _preferences: Option<Vec<String>>,
        _values: Box<Option<Vec<f32>>>,
        _signup_duration: std::time::Duration,
        _recent_purchases: Option<Vec<Option<Box<Vec<i64>>>>>,
    }
    const WRITER: GenericSqlWriter = GenericSqlWriter {};

    #[tokio::test]
    async fn test_simple_entity() {
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
        let xx = &columns[1].default.as_deref().unwrap() as &dyn Any;
        let xx = xx.downcast_ref::<Operand>();
        assert!(matches!(
            xx,
            Some(Operand::LitArray(&[
                Operand::LitStr("discount"),
                Operand::LitStr("newsletter"),
            ]))
        ));
        assert!(matches!(columns[2].default, None));
        assert!(matches!(columns[3].default, None));
        assert!(matches!(columns[4].default, None));
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
        assert_eq!(columns[0].auto_increment, false);
        assert_eq!(columns[1].auto_increment, false);
        assert_eq!(columns[2].auto_increment, false);
        assert_eq!(columns[3].auto_increment, false);
        assert_eq!(columns[4].auto_increment, false);
        assert_eq!(columns[0].passive, false);
        assert_eq!(columns[1].passive, false);
        assert_eq!(columns[2].passive, false);
        assert_eq!(columns[3].passive, false);
        assert_eq!(columns[4].passive, false);
    }

    #[test]
    fn test_simple_entity_create_table() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_create_table::<Customer>(&mut query, false),
            indoc! {"
                CREATE TABLE customers (
                transaction_ids UBIGINT[] NOT NULL,
                preference VARCHAR[] DEFAULT DEFAULT ['discount','newsletter'],
                lifetime_value FLOAT[],
                signup_duration INTERVAL NOT NULL,
                recent_purchases BIGINT[]
                )
            "}
            .trim()
        );
    }

    #[test]
    fn test_simple_entity_drop_table() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_drop_table::<Customer>(&mut query, false),
            "DROP TABLE customers"
        );
    }

    #[test]
    fn test_simple_entity_select() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_select::<Customer, _, _>(
                &mut query,
                Customer::table_ref(),
                ::tank::BinaryOp {
                    op: ::tank::BinaryOpType::Greater,
                    lhs: ::tank::Operand::Call(&[::tank::Operand::Column(
                        Customer::_values.into()
                    )]),
                    rhs: ::tank::Operand::LitInt(10),
                },
                Some(10),
            ),
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
    fn test_simple_entity_delete() {
        let mut query = String::new();
        assert_eq!(
            WRITER.sql_delete::<Customer, _>(&mut query, &expr!()),
            indoc! {"
                DELETE FROM settings
                WHERE true
            "}
            .trim()
        );
    }
}
