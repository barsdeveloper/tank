#[cfg(test)]
mod tests {
    use tank::{
        BinaryOp, BinaryOpType, Expression, Operand, SqlWriter, UnaryOp, UnaryOpType, Value,
    };
    use tank_macros::{expr, Entity};

    struct Writer;
    impl SqlWriter for Writer {
        fn as_dyn(&self) -> &dyn SqlWriter {
            self
        }
    }

    const WRITER: Writer = Writer {};

    #[test]
    fn simple() {
        let expr = expr!();
        assert!(matches!(expr, ()));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "");

        let expr = expr!(1 + 2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Addition,
                lhs: Operand::LitInt(1),
                rhs: Operand::LitInt(2),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "1 + 2");

        let expr = expr!(5 * 1.2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Multiplication,
                lhs: Operand::LitInt(5),
                rhs: Operand::LitFloat(1.2),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "5 * 1.2");

        let expr = expr!(true && false);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::And,
                lhs: Operand::LitBool(true),
                rhs: Operand::LitBool(false),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "true AND false");

        let expr = expr!(45 | -90);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::BitwiseOr,
                lhs: Operand::LitInt(45),
                rhs: UnaryOp {
                    op: UnaryOpType::Negative,
                    v: Operand::LitInt(90),
                },
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "45 | -90");

        let expr = expr!(true as i32);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Cast,
                lhs: Operand::LitBool(true),
                rhs: Operand::Type(Value::Int32(..)),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "CAST(true AS INTEGER)");

        let expr = expr!("1.5" as f64);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Cast,
                lhs: Operand::LitStr("1.5"),
                rhs: Operand::Type(Value::Float64(..)),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "CAST('1.5' AS DOUBLE)");

        let expr = expr!(["a", "b", "c"]);
        assert!(matches!(
            expr,
            Operand::LitArray([
                Operand::LitStr("a"),
                Operand::LitStr("b"),
                Operand::LitStr("c"),
            ])
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "['a', 'b', 'c']");

        let expr = expr!([11, 22, 33][1]);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Indexing,
                lhs: Operand::LitArray([
                    Operand::LitInt(11),
                    Operand::LitInt(22),
                    Operand::LitInt(33),
                ]),
                rhs: Operand::LitInt(1),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "[11, 22, 33][1]");

        let expr = expr!("hello" == "hell_" as LIKE);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Like,
                lhs: Operand::LitStr("hello"),
                rhs: Operand::LitStr("hell_"),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "'hello' LIKE 'hell_'");

        let expr = expr!("abc" != "A%" as LIKE);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::NotLike,
                lhs: Operand::LitStr("abc"),
                rhs: Operand::LitStr("A%"),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "'abc' NOT LIKE 'A%'");

        let expr = expr!("log.txt" != "src/**/log.{txt,csv}" as GLOB);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::NotGlob,
                lhs: Operand::LitStr("log.txt"),
                rhs: Operand::LitStr("src/**/log.{txt,csv}"),
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "'log.txt' NOT GLOB 'src/**/log.{txt,csv}'");

        let expr = expr!(true as i32);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Cast,
                lhs: Operand::LitBool(true),
                rhs: Operand::Type(Value::Int32(..))
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "CAST(true AS INTEGER)");

        let expr = expr!("value" != NULL);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::IsNot,
                lhs: Operand::LitStr("value"),
                rhs: Operand::Null,
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "'value' IS NOT NULL");
    }

    #[test]
    fn complex() {
        let expr = expr!(90.5 - -0.54 * 2 < 7 / 2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Less,
                lhs: BinaryOp {
                    op: BinaryOpType::Subtraction,
                    lhs: Operand::LitFloat(90.5),
                    rhs: BinaryOp {
                        op: BinaryOpType::Multiplication,
                        lhs: UnaryOp {
                            op: UnaryOpType::Negative,
                            v: Operand::LitFloat(0.54),
                        },
                        rhs: Operand::LitInt(2),
                    },
                },
                rhs: BinaryOp {
                    op: BinaryOpType::Division,
                    lhs: Operand::LitInt(7),
                    rhs: Operand::LitInt(2),
                },
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "90.5 - -0.54 * 2 < 7 / 2");

        let expr = expr!((2 + 3) * (4 - 1) >> 1 & (8 | 3));
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::BitwiseAnd,
                lhs: BinaryOp {
                    op: BinaryOpType::ShiftRight,
                    lhs: BinaryOp {
                        op: BinaryOpType::Multiplication,
                        lhs: BinaryOp {
                            op: BinaryOpType::Addition,
                            lhs: Operand::LitInt(2),
                            rhs: Operand::LitInt(3),
                        },
                        rhs: BinaryOp {
                            op: BinaryOpType::Subtraction,
                            lhs: Operand::LitInt(4),
                            rhs: Operand::LitInt(1),
                        },
                    },
                    rhs: Operand::LitInt(1),
                },
                rhs: BinaryOp {
                    op: BinaryOpType::BitwiseOr,
                    lhs: Operand::LitInt(8),
                    rhs: Operand::LitInt(3),
                },
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "(2 + 3) * (4 - 1) >> 1 & (8 | 3)");

        let expr = expr!(-(-PI) + 2 * (5 % (2 + 1)) == 7 && !(4 < 2));
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::And,
                lhs: BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: BinaryOp {
                        op: BinaryOpType::Addition,
                        lhs: UnaryOp {
                            op: UnaryOpType::Negative,
                            v: UnaryOp {
                                op: UnaryOpType::Negative,
                                v: Operand::LitIdent("PI"),
                            },
                        },
                        rhs: BinaryOp {
                            op: BinaryOpType::Multiplication,
                            lhs: Operand::LitInt(2),
                            rhs: BinaryOp {
                                op: BinaryOpType::Remainder,
                                lhs: Operand::LitInt(5),
                                rhs: BinaryOp {
                                    op: BinaryOpType::Addition,
                                    lhs: Operand::LitInt(2),
                                    rhs: Operand::LitInt(1),
                                },
                            },
                        },
                    },
                    rhs: Operand::LitInt(7),
                },
                rhs: UnaryOp {
                    op: UnaryOpType::Not,
                    v: BinaryOp {
                        op: BinaryOpType::Less,
                        lhs: Operand::LitInt(4),
                        rhs: Operand::LitInt(2),
                    },
                },
            }
        ));
        let mut out = String::new();
        expr.sql_write(&WRITER, &mut out, false);
        assert_eq!(out, "-(-PI) + 2 * (5 % (2 + 1)) = 7 AND NOT 4 < 2");
    }

    #[test]
    fn variables() {
        let one = 1;
        let three = 3;
        let expr = expr!(#one + 2 == #three);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Equal,
                lhs: BinaryOp {
                    op: BinaryOpType::Addition,
                    lhs: Operand::Variable(Value::Int32(Some(1))),
                    rhs: Operand::LitInt(2),
                },
                rhs: Operand::Variable(Value::Int32(Some(3))),
            }
        ));

        let vec = vec![-1, -2, -3, -4];
        let index = 2;
        let expr = expr!(#vec[#index + 1] + 60);
        assert!(matches!(
                expr,
                BinaryOp {
                    op: BinaryOpType::Addition,
                    lhs: BinaryOp {
                        op: BinaryOpType::Indexing,
                        lhs: Operand::Variable(Value::List(Some(ref vec), ..)),
                        rhs: BinaryOp {
                            op: BinaryOpType::Addition,
                            lhs: Operand::Variable(Value::Int32(Some(2))),
                            rhs: Operand::LitInt(1),
                        },
                    },
                    rhs: Operand::LitInt(60),
                }if vec.as_slice() == &[
            Value::Int32(Some(-1)),
            Value::Int32(Some(-2)),
            Value::Int32(Some(-3)),
            Value::Int32(Some(-4)),
        ]
            ));
    }

    #[test]
    fn columns() {
        // #[derive(Entity)]
        // #[table_name("the_table")]
        struct MyEntity {
            _first: i128,
            _second: String,
            _third: Vec<f64>,
        }

        // Recursive expansion of Entity macro
        // ====================================

        trait MyEntityFromRowTrait {
            fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<MyEntity>;
        }
        #[derive(Default)]
        struct MyEntityFromRowFactory<T>(std::marker::PhantomData<T>);

        impl<T: Default + Into<MyEntity>> MyEntityFromRowFactory<T> {
            fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<MyEntity> {
                let mut result = T::default().into();
                ::std::iter::zip(row.labels.iter(), row.values.into_vec().into_iter())
                    .try_for_each(|(name, value)| {
                        if name == "_first" {
                            result._first = <i128 as ::tank::AsValue>::try_from_value(value)?;
                        } else if name == "_second" {
                            result._second = <String as ::tank::AsValue>::try_from_value(value)?;
                        } else if name == "_third" {
                            result._third = <Vec<f64> as ::tank::AsValue>::try_from_value(value)?;
                        }
                        Ok::<_, ::tank::Error>(())
                    });
                Ok(result)
            }
        }
        impl<T> MyEntityFromRowTrait for MyEntityFromRowFactory<T> {
            fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<MyEntity> {
                let mut _first: Option<i128> = None;
                let mut _second: Option<String> = None;
                let mut _third: Option<Vec<f64>> = None;
                ::std::iter::zip(row.labels.iter(), row.values.into_vec().into_iter())
                    .try_for_each(|(name, value)| {
                        if name == "_first" {
                            _first = Some(<i128 as ::tank::AsValue>::try_from_value(value)?);
                        } else if name == "_second" {
                            _second = Some(<String as ::tank::AsValue>::try_from_value(value)?);
                        } else if name == "_third" {
                            _third = Some(<Vec<f64> as ::tank::AsValue>::try_from_value(value)?);
                        }
                        Ok::<_, ::tank::Error>(())
                    });
                Ok(MyEntity {
                    _first: _first.ok_or(::tank::Error::msg(format!(
                        "Field `{}` does not exist in the row provided",
                        "_first"
                    )))?,
                    _second: _second.ok_or(::tank::Error::msg(format!(
                        "Field `{}` does not exist in the row provided",
                        "_second"
                    )))?,
                    _third: _third.ok_or(::tank::Error::msg(format!(
                        "Field `{}` does not exist in the row provided",
                        "_third"
                    )))?,
                })
            }
        }
        trait MyEntityColumnTrait {
            #[allow(non_upper_case_globals)]
            const _first: ::tank::ColumnRef;
            #[allow(non_upper_case_globals)]
            const _second: ::tank::ColumnRef;
            #[allow(non_upper_case_globals)]
            const _third: ::tank::ColumnRef;
        }
        impl MyEntityColumnTrait for MyEntity {
            const _first: ::tank::ColumnRef = ::tank::ColumnRef {
                name: "first",
                table: "the_table",
                schema: "",
            };
            const _second: ::tank::ColumnRef = ::tank::ColumnRef {
                name: "second",
                table: "the_table",
                schema: "",
            };
            const _third: ::tank::ColumnRef = ::tank::ColumnRef {
                name: "third",
                table: "the_table",
                schema: "",
            };
        }
        impl ::tank::Entity for MyEntity {
            type PrimaryKey<'a> = ();
            fn table_name() -> &'static str {
                "the_table"
            }
            fn schema_name() -> &'static str {
                ""
            }
            fn table_ref() -> &'static ::tank::TableRef {
                static TABLE_REF: ::tank::TableRef = ::tank::TableRef {
                    name: "the_table",
                    schema: "",
                    alias: ::std::borrow::Cow::Borrowed(""),
                };
                &TABLE_REF
            }
            fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<Self> {
                MyEntityFromRowFactory::<Self>::from_row(row)
            }
            fn columns_def() -> &'static [::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock<Box<[::tank::ColumnDef]>> =
                    ::std::sync::LazyLock::new(|| {
                        vec![
                            ::tank::ColumnDef {
                                reference: MyEntity::_first,
                                column_type: "",
                                value: ::tank::Value::Int128(None),
                                nullable: false,
                                default: None,
                                primary_key: ::tank::PrimaryKeyType::None,
                                unique: false,
                                auto_increment: false,
                                passive: false,
                            },
                            ::tank::ColumnDef {
                                reference: MyEntity::_second,
                                column_type: "",
                                value: ::tank::Value::Varchar(None),
                                nullable: false,
                                default: None,
                                primary_key: ::tank::PrimaryKeyType::None,
                                unique: false,
                                auto_increment: false,
                                passive: false,
                            },
                            ::tank::ColumnDef {
                                reference: MyEntity::_third,
                                column_type: "",
                                value: ::tank::Value::List(
                                    None,
                                    Box::new(::tank::Value::Float64(None)),
                                ),
                                nullable: false,
                                default: None,
                                primary_key: ::tank::PrimaryKeyType::None,
                                unique: false,
                                auto_increment: false,
                                passive: false,
                            },
                        ]
                        .into_boxed_slice()
                    });
                &RESULT
            }
            fn primary_key_def() -> &'static [&'static ::tank::ColumnDef] {
                static RESULT: ::std::sync::LazyLock<Box<[&::tank::ColumnDef]>> =
                    ::std::sync::LazyLock::new(|| {
                        let columns = MyEntity::columns_def();
                        vec![].into_boxed_slice()
                    });
                &RESULT
            }
            async fn create_table<E: ::tank::Executor>(
                executor: &mut E,
                if_not_exists: bool,
            ) -> ::tank::Result<()> {
                let mut query = String::with_capacity(512);
                ::tank::SqlWriter::sql_create_table::<MyEntity>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    if_not_exists,
                );
                executor.execute(query.into()).await.map(|_| ())
            }
            async fn drop_table<E: ::tank::Executor>(
                executor: &mut E,
                if_exists: bool,
            ) -> ::tank::Result<()> {
                let mut query = String::with_capacity(64);
                ::tank::SqlWriter::sql_drop_table::<MyEntity>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut query,
                    if_exists,
                );
                executor
                    .execute(::tank::Query::Raw(query.into()))
                    .await
                    .map(|_| ())
            }
            fn find_one<E: ::tank::Executor>(
                executor: &mut E,
                primary_key: &Self::PrimaryKey<'_>,
            ) -> impl ::std::future::Future<Output = ::tank::Result<Option<Self>>> {
                let mut query = String::with_capacity(256);
                ::tank::SqlWriter::sql_select::<MyEntity, _, _>(
                    &::tank::Driver::sql_writer(executor.driver()),
                    &mut String::with_capacity(256),
                    Self::table_ref(),
                    &::tank::expr!(),
                    Some(1),
                );
                let stream = executor.fetch(query.into());
                async move {
                    let mut stream = ::std::pin::pin!(stream);
                    ::tank::stream::StreamExt::next(&mut stream)
                        .await
                        .transpose()?
                        .map(Self::from_row)
                        .transpose()
                }
            }
            fn find_many<E: ::tank::Executor, Expr: ::tank::Expression>(
                executor: &mut E,
                condition: Expr,
            ) -> impl ::tank::stream::Stream<Item = ::tank::Result<Self>> {
                ::tank::stream::empty()
            }
            fn row_labeled(&self) -> ::tank::RowLabeled {
                ::tank::RowLabeled {
                    labels: [
                        ("first".into(), true),
                        ("second".into(), true),
                        ("third".into(), true),
                    ]
                    .into_iter()
                    .filter_map(|(v, f)| if f { Some(v) } else { None })
                    .collect::<::tank::RowNames>(),
                    values: self.row(),
                }
            }
            fn row(&self) -> ::tank::Row {
                [
                    (self._first.clone().into(), true),
                    (self._second.clone().into(), true),
                    (self._third.clone().into(), true),
                ]
                .into_iter()
                .filter_map(|(v, f)| if f { Some(v) } else { None })
                .collect::<::tank::Row>()
            }
            fn primary_key<'a>(&'a self) -> Self::PrimaryKey<'a> {
                ()
            }
            async fn save<E: ::tank::Executor>(&self, executor: &mut E) -> ::tank::Result<()> {
                Ok(())
            }
        }

        let expr = expr!(MyEntity::_first + 2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Addition,
                lhs: Operand::Column(..),
                rhs: Operand::LitInt(2),
            }
        ));

        let Operand::Column(ref col) = expr.lhs else {
            panic!("Unexpected error")
        };
        assert_eq!(col.name, "first");
        assert_eq!(col.table, "the_table");
        assert_eq!(col.schema, "");
        {
            let mut out = String::new();
            expr.sql_write(&WRITER, &mut out, false);
            assert_eq!(out, "first + 2");
        }
        {
            let mut out = String::new();
            expr.sql_write(&WRITER, &mut out, true);
            assert_eq!(out, "the_table.first + 2");
        }

        let expr = expr!(MyEntity::_first != NULL);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::IsNot,
                lhs: Operand::Column(..),
                rhs: Operand::Null,
            }
        ));
    }
}
