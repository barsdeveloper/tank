#[cfg(test)]
mod tests {
    use indoc::indoc;
    use tank::expr;
    use tank::join;
    use tank::Entity;
    use tank::SqlWriter;
    use tank_duckdb::DuckDBSqlWriter;

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
}
