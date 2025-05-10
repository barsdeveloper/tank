#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rust_decimal::Decimal;
    use tank::expr;
    use tank::Entity;
    use tank::SqlWriter;
    use tank_duckdb::DuckDBSqlWriter;
    use time::PrimitiveDateTime;
    use uuid::Uuid;

    const WRITER: DuckDBSqlWriter = DuckDBSqlWriter::new();

    #[test]
    fn test_1() {
        #[derive(Entity)]
        #[table_name("my_table")]
        struct Table {
            #[column_name("special_column")]
            _first_column: Option<String>,
            _second_column: Box<f64>,
            _third_column: i32,
        }
        {
            let mut out = String::new();
            WRITER.sql_create_table::<Table>(&mut out, false);
            assert_eq!(
                out,
                indoc! {"
                    CREATE TABLE my_table(
                    special_column VARCHAR,
                    second_column DOUBLE NOT NULL,
                    third_column INTEGER NOT NULL
                    )
                "}
                .trim()
            )
        }
        {
            let mut out = String::new();
            WRITER.sql_drop_table::<Table>(&mut out, true);
            assert_eq!(out, "DROP TABLE IF EXISTS my_table")
        }
        {
            let mut out = String::new();
            WRITER.sql_select::<Table, _, _>(
                &mut out,
                Table::table_ref(),
                &expr!(Table::_second_column < 100 && Table::_first_column == "OK"),
                None,
            );
            assert_eq!(
                out,
                indoc! {"
                    SELECT special_column, second_column, third_column
                    FROM my_table
                    WHERE second_column < 100 AND special_column = 'OK'
                "}
                .trim()
            )
        }
    }

    fn test_2() {
        #[derive(Entity)]
        #[table_name("cart")]
        struct Cart {
            id: Uuid,
            user_id: Uuid,
            created_at: PrimitiveDateTime,
            items: Vec<u32>,
            is_active: bool,
            total_price: Decimal, // (Decimal, prec, scale)
        }

        #[derive(Debug)]
        struct CartItem {
            product_id: Uuid,
            quantity: u32,
            price_each: f64,
            notes: Option<String>,
        }
    }
}
