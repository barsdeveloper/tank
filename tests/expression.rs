#![feature(assert_matches)]
#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;
    use tank::{
        BinaryOp, BinaryOpType, ColumnRef, Expression, OpPrecedence, Operand, SqlWriter, UnaryOp,
        UnaryOpType, Value,
    };
    use tank_core::Entity;
    use tank_macros::{Entity, expr};

    struct Writer;
    impl SqlWriter for Writer {
        fn as_dyn(&self) -> &dyn SqlWriter {
            self
        }
    }

    const WRITER: Writer = Writer {};

    #[test]
    fn test_simple_expressions() {
        let expr = expr!();
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "false");
        assert_matches!(expr, Operand::LitBool(false));

        let expr = expr!(1 + 2);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "1 + 2");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Addition,
                lhs: Operand::LitInt(1),
                rhs: Operand::LitInt(2),
            }
        );

        let expr = expr!(5 * 1.2);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "5 * 1.2");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Multiplication,
                lhs: Operand::LitInt(5),
                rhs: Operand::LitFloat(1.2),
            }
        );

        let expr = expr!(true && false);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "true AND false");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::And,
                lhs: Operand::LitBool(true),
                rhs: Operand::LitBool(false),
            }
        );

        let expr = expr!(45 | -90);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "45 | -90");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::BitwiseOr,
                lhs: Operand::LitInt(45),
                rhs: UnaryOp {
                    op: UnaryOpType::Negative,
                    v: Operand::LitInt(90),
                },
            }
        );

        let expr = expr!(CAST(true as i32));
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "CAST(true AS INTEGER)");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Cast,
                lhs: Operand::LitBool(true),
                rhs: Operand::Type(Value::Int32(..)),
            }
        );

        let expr = expr!(CAST("1.5" as f64));
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "CAST('1.5' AS DOUBLE)");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Cast,
                lhs: Operand::LitStr("1.5"),
                rhs: Operand::Type(Value::Float64(..)),
            }
        );

        let expr = expr!(["a", "b", "c"]);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "['a', 'b', 'c']");
        assert_matches!(
            expr,
            Operand::LitArray([
                Operand::LitStr("a"),
                Operand::LitStr("b"),
                Operand::LitStr("c"),
            ])
        );

        let expr = expr!([11, 22, 33][1]);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "[11, 22, 33][1]");
        assert_matches!(
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
        );

        let expr = expr!("hello" == "hell_" as LIKE);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "'hello' LIKE 'hell_'");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Like,
                lhs: Operand::LitStr("hello"),
                rhs: Operand::LitStr("hell_"),
            }
        );

        let expr = expr!("abc" != "A%" as LIKE);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "'abc' NOT LIKE 'A%'");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::NotLike,
                lhs: Operand::LitStr("abc"),
                rhs: Operand::LitStr("A%"),
            }
        );

        let expr = expr!("log.txt" != "src/**/log.{txt,csv}" as GLOB);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "'log.txt' NOT GLOB 'src/**/log.{txt,csv}'");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::NotGlob,
                lhs: Operand::LitStr("log.txt"),
                rhs: Operand::LitStr("src/**/log.{txt,csv}"),
            }
        );

        let expr = expr!(CAST(true as i32));
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "CAST(true AS INTEGER)");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Cast,
                lhs: Operand::LitBool(true),
                rhs: Operand::Type(Value::Int32(..))
            }
        );

        let expr = expr!("value" != NULL);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "'value' IS NOT NULL");
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::IsNot,
                lhs: Operand::LitStr("value"),
                rhs: Operand::Null,
            }
        );
    }

    #[test]
    fn test_asterisk_expressions() {
        let expr = expr!(COUNT(*));
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "COUNT(*)");
        assert_matches!(expr, Operand::Call("COUNT", _));

        #[derive(Entity)]
        struct ATable {
            #[tank(name = "my_column")]
            a_column: u8,
        }
        let expr = expr!(SUM(ATable::a_column));
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "SUM(my_column)");
        assert_matches!(expr, Operand::Call("SUM", _));
    }

    #[test]
    fn test_complex_expressions() {
        let expr = expr!(90.5 - -0.54 * 2 < 7 / 2);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "90.5 - -0.54 * 2 < 7 / 2");
        assert_matches!(
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
        );

        let expr = expr!((2 + 3) * (4 - 1) >> 1 & (8 | 3));
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "(2 + 3) * (4 - 1) >> 1 & (8 | 3)");
        assert_matches!(
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
        );

        let expr = expr!(-(-PI) + 2 * (5 % (2 + 1)) == 7 && !(4 < 2));
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "-(-PI) + 2 * (5 % (2 + 1)) = 7 AND NOT 4 < 2");
        assert_matches!(
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
        );
    }

    #[test]
    fn test_variables_expressions() {
        let one = 1;
        let three = 3;
        let expr = expr!(#one + 2 == #three);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "1 + 2 = 3");
        assert_matches!(
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
        );

        let vec = vec![-1, -2, -3, -4];
        let index = 2;
        let expr = expr!(#vec[#index + 1] + 60);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, false);
        assert_eq!(out, "[-1,-2,-3,-4][2 + 1] + 60");
        assert_matches!(
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
            } if vec.as_slice() == &[
                Value::Int32(Some(-1)),
                Value::Int32(Some(-2)),
                Value::Int32(Some(-3)),
                Value::Int32(Some(-4)),
            ]
        );
    }

    #[test]
    fn test_columns_expressions() {
        #[derive(Entity)]
        #[tank(name = "the_table")]
        struct MyEntity {
            _first: i128,
            _second: String,
            _third: Vec<f64>,
        }
        assert!(MyEntity::columns()[0].precedence(&WRITER) > 0); // For coverage purpose

        let expr = expr!(MyEntity::_first + 2);
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Addition,
                lhs: ColumnRef {
                    name: "first",
                    table: "the_table",
                    schema: ""
                },
                rhs: Operand::LitInt(2),
            }
        );

        {
            let mut out = String::new();
            expr.write_query(&WRITER, &mut out, false);
            assert_eq!(out, "first + 2");
        }
        {
            let mut out = String::new();
            expr.write_query(&WRITER, &mut out, true);
            assert_eq!(out, "the_table.first + 2");
        }

        let expr = expr!(MyEntity::_first != NULL);
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::IsNot,
                lhs: ColumnRef {
                    name: "first",
                    table: "the_table",
                    schema: ""
                },
                rhs: Operand::Null,
            }
        );

        let expr =
            expr!(CAST(MyEntity::_first as String) == MyEntity::_second && MyEntity::_first > 0);
        let mut out = String::new();
        expr.write_query(&WRITER, &mut out, true);
        assert_eq!(
            out,
            "CAST(the_table.first AS VARCHAR) = the_table.second AND the_table.first > 0"
        );
        assert_matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::And,
                lhs: BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: BinaryOp {
                        op: BinaryOpType::Cast,
                        lhs: ColumnRef {
                            name: "first",
                            table: "the_table",
                            schema: ""
                        },
                        rhs: Operand::Type(Value::Varchar(None)),
                    },
                    rhs: ColumnRef {
                        name: "second",
                        table: "the_table",
                        schema: ""
                    },
                },
                rhs: BinaryOp {
                    op: BinaryOpType::Greater,
                    lhs: ColumnRef {
                        name: "first",
                        table: "the_table",
                        schema: ""
                    },
                    rhs: Operand::LitInt(0),
                },
            }
        );
    }
}
