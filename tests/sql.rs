#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use tank::{Entity, Passive, SqlWriter, expr};
    use time::{Date, Month, PrimitiveDateTime, Time};
    use uuid::Uuid;

    struct Writer;
    impl SqlWriter for Writer {
        fn as_dyn(&self) -> &dyn SqlWriter {
            self
        }
    }

    const WRITER: Writer = Writer {};

    #[test]
    fn test_sql_simple_table() {
        #[derive(Default, Entity)]
        #[tank(name = "my_table")]
        struct Table {
            #[tank(name = "special_column")]
            _first_column: Option<String>,
            _second_column: Box<f64>,
            #[tank(primary_key)]
            _third_column: i32,
        }
        {
            let mut buff = String::new();
            WRITER.write_create_table::<Table, _>(&mut buff, false);
            assert_eq!(
                buff,
                indoc! {r#"
                    CREATE TABLE "my_table" (
                    "special_column" VARCHAR,
                    "second_column" DOUBLE NOT NULL,
                    "third_column" INTEGER PRIMARY KEY
                    );
                "#}
                .trim()
            )
        }
        {
            let mut buff = String::new();
            WRITER.write_drop_table::<Table, _>(&mut buff, true);
            assert_eq!(buff, r#"DROP TABLE IF EXISTS "my_table";"#)
        }
        {
            let mut buff = String::new();
            WRITER.write_select(
                &mut buff,
                Table::columns(),
                Table::table_ref(),
                &expr!(Table::_second_column < 100 && Table::_first_column == "OK"),
                None,
            );
            assert_eq!(
                buff,
                indoc! {r#"
                    SELECT "special_column", "second_column", "third_column"
                    FROM "my_table"
                    WHERE "second_column" < 100 AND "special_column" = 'OK';
                "#}
                .trim()
            )
        }
        {
            let mut buff = String::new();
            let table = Table::default();
            WRITER.write_insert(&mut buff, [&table], false);
            assert_eq!(
                buff,
                indoc! {r#"
                    INSERT INTO "my_table" ("special_column", "second_column", "third_column") VALUES
                    (NULL, 0.0, 0);
                "#}
                .trim()
            )
        }
        {
            let mut buff = String::new();
            let table = Table {
                _first_column: Some("hello".into()),
                _second_column: 512.5.into(),
                _third_column: 478,
            };
            WRITER.write_insert(&mut buff, [&table], true);
            assert_eq!(
                buff,
                indoc! {r#"
                    INSERT INTO "my_table" ("special_column", "second_column", "third_column") VALUES
                    ('hello', 512.5, 478)
                    ON CONFLICT ("third_column") DO UPDATE SET
                    "special_column" = EXCLUDED."special_column",
                    "second_column" = EXCLUDED."second_column";
                "#}
                .trim()
            )
        }
    }

    #[test]
    fn test_sql_cart() {
        #[derive(Entity)]
        #[tank(name = "cart")]
        struct Cart {
            #[tank(primary_key)]
            id: Box<Passive<u32>>,
            user_id: Uuid,
            created_at: PrimitiveDateTime,
            items: Vec<Uuid>,
            is_active: bool,
            total_price: Decimal,
        }

        {
            let mut buff = String::new();
            WRITER.write_create_table::<Cart, _>(&mut buff, true);
            assert_eq!(
                buff,
                indoc! {r#"
                    CREATE TABLE IF NOT EXISTS "cart" (
                    "id" UINTEGER PRIMARY KEY,
                    "user_id" UUID NOT NULL,
                    "created_at" TIMESTAMP NOT NULL,
                    "items" UUID[] NOT NULL,
                    "is_active" BOOLEAN NOT NULL,
                    "total_price" DECIMAL NOT NULL
                    );
                "#}
                .trim()
            )
        }
        {
            let mut buff = String::new();
            WRITER.write_drop_table::<Cart, _>(&mut buff, false);
            assert_eq!(buff, r#"DROP TABLE "cart";"#)
        }
        {
            let mut buff = String::new();
            WRITER.write_select(
                &mut buff,
                Cart::columns(),
                Cart::table_ref(),
                &expr!(Cart::is_active == true && Cart::total_price > 100),
                Some(1000),
            );
            assert_eq!(
                buff,
                indoc! {r#"
                    SELECT "id", "user_id", "created_at", "items", "is_active", "total_price"
                    FROM "cart"
                    WHERE "is_active" = true AND "total_price" > 100
                    LIMIT 1000;
                "#}
                .trim()
            )
        }
        {
            let mut buff = String::new();
            let cart = Cart {
                id: Default::default(),
                user_id: Uuid::from_str("b0fa843f-6ae4-4a16-a13c-ddf5512f3bb2").unwrap(),
                created_at: PrimitiveDateTime::new(
                    Date::from_calendar_date(2025, Month::May, 31).unwrap(),
                    Time::from_hms(12, 30, 11).unwrap(),
                ),
                items: Default::default(),
                is_active: Default::default(),
                total_price: Default::default(),
            };
            WRITER.write_insert(&mut buff, [&cart], false);
            assert_eq!(
                buff,
                indoc! {r#"
                    INSERT INTO "cart" ("user_id", "created_at", "items", "is_active", "total_price") VALUES
                    ('b0fa843f-6ae4-4a16-a13c-ddf5512f3bb2', '2025-05-31T12:30:11.0', [], false, 0);
                "#}
                .trim()
            )
        }
        {
            let mut buff = String::new();
            let cart = Cart {
                id: Default::default(),
                user_id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
                created_at: PrimitiveDateTime::new(
                    Date::from_calendar_date(2020, Month::January, 19).unwrap(),
                    Time::from_hms(19, 26, 54).unwrap(),
                ),
                items: vec![
                    Uuid::from_str("30c68157-5c43-452d-8caa-300776260b3f").unwrap(),
                    Uuid::from_str("772ba17d-b3bd-4771-a34e-2926d4731b44").unwrap(),
                    Uuid::from_str("3d4e9cb1-021f-48ab-848e-6c97d0ad670d").unwrap(),
                ],
                is_active: true,
                total_price: Decimal::new(2599, 2), // 25.99
            };
            WRITER.write_insert(&mut buff, [&cart], true);
            assert_eq!(
                buff,
                indoc! {r#"
                    INSERT INTO "cart" ("user_id", "created_at", "items", "is_active", "total_price") VALUES
                    ('22222222-2222-2222-2222-222222222222', '2020-01-19T19:26:54.0', ['30c68157-5c43-452d-8caa-300776260b3f','772ba17d-b3bd-4771-a34e-2926d4731b44','3d4e9cb1-021f-48ab-848e-6c97d0ad670d'], true, 25.99)
                    ON CONFLICT ("id") DO UPDATE SET
                    "user_id" = EXCLUDED."user_id",
                    "created_at" = EXCLUDED."created_at",
                    "items" = EXCLUDED."items",
                    "is_active" = EXCLUDED."is_active",
                    "total_price" = EXCLUDED."total_price";
                "#}
                .trim()
            )
        }
    }
}
