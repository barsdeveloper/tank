#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use tank::{
        join, BinaryOp, BinaryOpType, ColumnRef, Entity, Join, JoinType, Operand, TableRef,
    };

    #[derive(Entity)]
    #[schema_name("my_data")]
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
        let join = join!(Alpha JOIN crate::tests::Bravo ON AlphaColumn::a == BravoColumn::first);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Inner,
                lhs: TableRef {
                    name: Cow::Borrowed("alpha"),
                    schema: Cow::Borrowed("my_data"),
                    ..
                },
                rhs: TableRef {
                    name: Cow::Borrowed("bravo"),
                    schema: Cow::Borrowed(""),
                    ..
                },
                on: Some(BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: Operand::Column(ColumnRef {
                        name: Cow::Borrowed("a"),
                        table: Cow::Borrowed("alpha"),
                        schema: Cow::Borrowed("my_data"),
                        ..
                    }),
                    rhs: Operand::Column(ColumnRef {
                        name: Cow::Borrowed("first"),
                        table: Cow::Borrowed("bravo"),
                        schema: Cow::Borrowed(""),
                        ..
                    }),
                    ..
                }),
                ..
            }
        ));

        let join = join!(Alpha INNER JOIN Bravo ON AlphaColumn::a == BravoColumn::first);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Inner,
                ..
            }
        ));

        let join = join!(Alpha FULL OUTER JOIN Bravo ON AlphaColumn::a == BravoColumn::first);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Outer,
                ..
            }
        ));

        let join = join!(Alpha OUTER JOIN Bravo ON AlphaColumn::a == BravoColumn::first);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Outer,
                ..
            }
        ));

        let join = join!(Alpha LEFT OUTER JOIN Bravo ON AlphaColumn::a == BravoColumn::first);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Left,
                ..
            }
        ));

        let join = join!(Alpha LEFT JOIN Bravo ON AlphaColumn::a == BravoColumn::first);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Left,
                ..
            }
        ));

        let join = join!(Alpha RIGHT OUTER JOIN Bravo ON AlphaColumn::a == BravoColumn::first);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Right,
                ..
            }
        ));

        let join = join!(Alpha RIGHT JOIN Bravo ON AlphaColumn::a == BravoColumn::first);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Right,
                ..
            }
        ));

        let join = join!(Alpha CROSS Bravo);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Cross,
                on: None,
                ..
            },
        ));

        let join = join!(Alpha NATURAL JOIN Bravo);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Natural,
                on: None,
                ..
            },
        ));
    }

    #[test]
    fn join_left_nested() {
        #[derive(Entity)]
        #[table_name("another_table")]
        struct Charlie {
            _column: u128,
        }

        let join = join!((Charlie JOIN Alpha ON CharlieColumn::column < AlphaColumn::b) JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Inner,
                lhs: Join {
                    join: JoinType::Inner,
                    lhs: TableRef {
                        name: Cow::Borrowed("another_table"),
                        ..
                    },
                    rhs: TableRef {
                        name: Cow::Borrowed("alpha"),
                        ..
                    },
                    on: Some(BinaryOp {
                        op: BinaryOpType::Less,
                        lhs: Operand::Column(ColumnRef {
                            name: Cow::Borrowed("column"),
                            table: Cow::Borrowed("another_table"),
                            ..
                        }),
                        rhs: Operand::Column(ColumnRef {
                            name: Cow::Borrowed("b"),
                            table: Cow::Borrowed("alpha"),
                            ..
                        }),
                        ..
                    }),
                    ..
                },
                rhs: TableRef {
                    name: Cow::Borrowed("bravo"),
                    ..
                },
                on: Some(BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: Operand::Column(ColumnRef {
                        name: Cow::Borrowed("a"),
                        ..
                    }),
                    rhs: Operand::Column(ColumnRef {
                        name: Cow::Borrowed("second"),
                        ..
                    }),
                    ..
                }),
                ..
            }
        ));
    }

    #[test]
    fn join_right_nested() {
        #[derive(Entity)]
        #[schema_name("delta_dataset")]
        #[table_name("delta_table")]
        struct Delta {
            _time_column: time::Date,
            #[column_name("the_string")]
            _string_column: Option<String>,
        }

        let join = join!(
            Bravo OUTER JOIN (
                Delta LEFT JOIN Alpha ON DeltaColumn::the_string < AlphaColumn::b
            ) ON BravoColumn::second == DeltaColumn::the_string
        );
        assert!(matches!(
            join,
            Join {
                join: JoinType::Outer,
                lhs: TableRef {
                    name: Cow::Borrowed("bravo"),
                    schema: Cow::Borrowed(""),
                    ..
                },
                rhs: Join {
                    join: JoinType::Left,
                    lhs: TableRef {
                        name: Cow::Borrowed("delta_table"),
                        schema: Cow::Borrowed("delta_dataset"),
                        ..
                    },
                    rhs: TableRef {
                        name: Cow::Borrowed("alpha"),
                        schema: Cow::Borrowed("my_data"),
                        ..
                    },
                    on: Some(BinaryOp {
                        op: BinaryOpType::Less,
                        lhs: Operand::Column(ColumnRef {
                            name: Cow::Borrowed("the_string"),
                            table: Cow::Borrowed("delta_table"),
                            schema: Cow::Borrowed("delta_dataset"),
                            ..
                        }),
                        rhs: Operand::Column(ColumnRef {
                            name: Cow::Borrowed("b"),
                            table: Cow::Borrowed("alpha"),
                            schema: Cow::Borrowed("my_data"),
                            ..
                        }),
                        ..
                    }),
                    ..
                },
                on: Some(BinaryOp {
                    op: BinaryOpType::Equal,
                    lhs: Operand::Column(ColumnRef {
                        name: Cow::Borrowed("second"),
                        table: Cow::Borrowed("bravo"),
                        ..
                    }),
                    rhs: Operand::Column(ColumnRef {
                        name: Cow::Borrowed("the_string"),
                        table: Cow::Borrowed("delta_table"),
                        ..
                    }),
                    ..
                }),
                ..
            }
        ));
    }

    #[test]
    fn join_chained() {
        #[derive(Entity)]
        struct Some {
            col: Box<i64>,
        }
        let join = join!(Alpha FULL OUTER JOIN Bravo ON AlphaColumn::b < BravoColumn::second RIGHT JOIN Some ON SomeColumn::col == BravoColumn::first);
    }
}
