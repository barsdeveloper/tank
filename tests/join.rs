#![feature(assert_matches)]
#[cfg(test)]
mod tests {
    use std::{assert_matches::assert_matches, borrow::Cow};
    use tank::{
        BinaryOp, BinaryOpType, ColumnRef, Entity, Join, JoinType, Operand, TableRef, join,
    };

    #[derive(Entity)]
    #[tank(schema = "my_data")]
    struct Alpha {
        _a: u32,
        _b: String,
    }
    #[derive(Entity)]
    struct Bravo {
        _first: u32,
        _second: String,
    }

    #[test]
    fn join_simple() {
        let join = join!(Alpha AA JOIN crate::tests::Bravo BB ON AA.a == BB.first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Inner,
                lhs: TableRef {
                    name: "alpha",
                    schema: "my_data",
                    alias: Cow::Borrowed("AA"),
                    ..
                },
                rhs: TableRef {
                    name: "bravo",
                    schema: "",
                    alias: Cow::Borrowed("BB"),
                    ..
                },
                on: Some(BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: Operand::LitField(["AA", "a"]),
                    rhs: Operand::LitField(["BB", "first"]),
                    ..
                }),
                ..
            }
        );

        let join = join!(Alpha INNER JOIN Bravo ON Alpha::_a == Bravo::_first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Inner,
                ..
            }
        );

        let join = join!(Alpha FULL OUTER JOIN Bravo ON Alpha::_a == Bravo::_first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Outer,
                ..
            }
        );

        let join = join!(Alpha OUTER JOIN Bravo ON Alpha::_a == Bravo::_first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Outer,
                ..
            }
        );

        let join = join!(Alpha LEFT OUTER JOIN Bravo ON Alpha::_a == Bravo::_first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Left,
                ..
            }
        );

        let join = join!(Alpha LEFT JOIN Bravo ON Alpha::_a == Bravo::_first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Left,
                ..
            }
        );

        let join = join!(Alpha RIGHT OUTER JOIN Bravo ON Alpha::_a == Bravo::_first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Right,
                ..
            }
        );

        let join = join!(Alpha RIGHT JOIN Bravo ON Alpha::_a == Bravo::_first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Right,
                ..
            }
        );

        let join = join!(Alpha CROSS Bravo);
        assert_matches!(
            join,
            Join {
                join: JoinType::Cross,
                on: None,
                ..
            },
        );

        let join = join!(Alpha NATURAL JOIN Bravo);
        assert_matches!(
            join,
            Join {
                join: JoinType::Natural,
                on: None,
                ..
            },
        );
    }

    #[test]
    fn join_left_nested() {
        #[derive(Entity)]
        #[tank(name = "another_table")]
        struct Charlie {
            _column: u128,
        }

        let join = join!((Charlie JOIN Alpha ON Charlie::_column < Alpha::_b) JOIN Bravo ON Alpha::_a == Bravo::_second);
        assert_matches!(
            join,
            Join {
                join: JoinType::Inner,
                lhs: Join {
                    join: JoinType::Inner,
                    lhs: TableRef {
                        name: "another_table",
                        ..
                    },
                    rhs: TableRef { name: "alpha", .. },
                    on: Some(BinaryOp {
                        op: BinaryOpType::Less,
                        lhs: ColumnRef {
                            name: "column",
                            table: "another_table",
                            ..
                        },
                        rhs: ColumnRef {
                            name: "b",
                            table: "alpha",
                            ..
                        },
                        ..
                    }),
                    ..
                },
                rhs: TableRef { name: "bravo", .. },
                on: Some(BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: ColumnRef { name: "a", .. },
                    rhs: ColumnRef { name: "second", .. },
                    ..
                }),
                ..
            }
        );
    }

    #[test]
    fn join_right_nested() {
        #[derive(Entity)]
        #[tank(schema = "delta_dataset", name = "delta_table")]
        struct Delta {
            _time_column: time::Date,
            #[tank(name = "the_string")]
            _string_column: Option<String>,
        }

        let join = join!(
            Bravo OUTER JOIN (
                Delta LEFT JOIN Alpha ON Delta::_string_column < Alpha::_b
            ) ON Bravo::_second == Delta::_string_column
        );
        assert_matches!(
            join,
            Join {
                join: JoinType::Outer,
                lhs: TableRef {
                    name: "bravo",
                    schema: "",
                    ..
                },
                rhs: Join {
                    join: JoinType::Left,
                    lhs: TableRef {
                        name: "delta_table",
                        schema: "delta_dataset",
                        ..
                    },
                    rhs: TableRef {
                        name: "alpha",
                        schema: "my_data",
                        ..
                    },
                    on: Some(BinaryOp {
                        op: BinaryOpType::Less,
                        lhs: ColumnRef {
                            name: "the_string",
                            table: "delta_table",
                            schema: "delta_dataset",
                            ..
                        },
                        rhs: ColumnRef {
                            name: "b",
                            table: "alpha",
                            schema: "my_data",
                            ..
                        },
                        ..
                    }),
                    ..
                },
                on: Some(BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: ColumnRef {
                        name: "second",
                        table: "bravo",
                        ..
                    },
                    rhs: ColumnRef {
                        name: "the_string",
                        table: "delta_table",
                        ..
                    },
                    ..
                }),
                ..
            }
        );
    }

    #[test]
    fn join_chained() {
        #[derive(Entity)]
        struct Some {
            col: Box<i64>,
        }
        let join = join!(Alpha A FULL OUTER JOIN Bravo ON Alpha::_b >= Bravo::_second RIGHT JOIN Some ON Some::col == Bravo::_first);
        assert_matches!(
            join,
            Join {
                join: JoinType::Right,
                lhs: Join {
                    join: JoinType::Outer,
                    lhs: TableRef {
                        name: "alpha",
                        schema: "my_data",
                        alias: Cow::Borrowed("A"),
                        ..
                    },
                    rhs: TableRef {
                        name: "bravo",
                        // alias: Cow::Borrowed(""),
                        ..
                    },
                    on: Some(BinaryOp {
                        op: BinaryOpType::GreaterEqual,
                        lhs: ColumnRef {
                            name: "b",
                            table: "alpha",
                            schema: "my_data",
                            ..
                        },
                        rhs: ColumnRef {
                            name: "second",
                            table: "bravo",
                            ..
                        },
                        ..
                    }),
                    ..
                },
                rhs: TableRef { name: "some", .. },
                on: Some(BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: ColumnRef {
                        name: "col",
                        table: "some",
                        ..
                    },
                    rhs: ColumnRef {
                        name: "first",
                        table: "bravo",
                        ..
                    },
                    ..
                }),
                ..
            }
        );
    }

    #[test]
    fn join_multi_chained() {
        #[derive(Entity)]
        #[tank(name = "ccc")]
        struct Charlie;

        #[derive(Entity)]
        struct Delta;

        let join = join!(
            Alpha NATURAL JOIN Charlie
                CROSS Bravo
                    LEFT JOIN Bravo ON Bravo::_second == Alpha::_b
                        CROSS Delta
        );
        assert_matches!(
            join,
            Join {
                join: JoinType::Cross,
                lhs: Join {
                    join: JoinType::Left,
                    lhs: Join {
                        join: JoinType::Cross,
                        lhs: Join {
                            join: JoinType::Natural,
                            lhs: TableRef { name: "alpha", .. },
                            rhs: TableRef { name: "ccc", .. },
                            on: None,
                            ..
                        },
                        ..
                    },
                    rhs: TableRef { name: "bravo", .. },
                    on: Some(BinaryOp {
                        op: BinaryOpType::Equal,
                        lhs: ColumnRef {
                            name: "second",
                            table: "bravo",
                            ..
                        },
                        rhs: ColumnRef {
                            name: "b",
                            table: "alpha",
                            schema: "my_data",
                            ..
                        },
                        ..
                    }),
                    ..
                },
                rhs: TableRef { name: "delta", .. },
                on: None,
                ..
            }
        );
    }

    #[test]
    fn join_with_many_parentheses() {
        let join = join!(
            ((((((Alpha RIGHT JOIN Bravo ON (((((((((((((Alpha::_a)) <= (((((((Bravo::_first))))))))))))))))))))))))
        );
        assert_matches!(
            join,
            Join {
                join: JoinType::Right,
                lhs: TableRef {
                    name: "alpha",
                    schema: "my_data",
                    ..
                },
                rhs: TableRef {
                    name: "bravo",
                    schema: "",
                    ..
                },
                on: Some(BinaryOp {
                    op: BinaryOpType::LessEqual,
                    lhs: ColumnRef {
                        name: "a",
                        table: "alpha",
                        schema: "my_data",
                        ..
                    },
                    rhs: ColumnRef {
                        name: "first",
                        table: "bravo",
                        schema: "",
                        ..
                    },
                    ..
                }),
                ..
            }
        );
    }
}
