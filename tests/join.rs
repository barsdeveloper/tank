#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use tank::{BinaryOp, BinaryOpType, ColumnRef, Entity, Join, JoinType, Operand, TableRef};
    use tank_core::join;

    #[derive(Entity)]
    #[schema_name("my_data")]
    struct Alpha {
        a: u32,
        b: String,
    }

    #[derive(Entity)]
    struct Bravo {
        first: f64,
        second: String,
    }

    #[test]
    fn join_simple() {
        let join = join!(Alpha JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
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
                        name: Cow::Borrowed("second"),
                        table: Cow::Borrowed("bravo"),
                        schema: Cow::Borrowed(""),
                        ..
                    }),
                    ..
                }),
                ..
            }
        ));

        let join = join!(Alpha INNER JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Inner,
                ..
            }
        ));

        let join = join!(Alpha FULL OUTER JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Outer,
                ..
            }
        ));

        let join = join!(Alpha OUTER JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Outer,
                ..
            }
        ));

        let join = join!(Alpha LEFT OUTER JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Left,
                ..
            }
        ));

        let join = join!(Alpha LEFT JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Left,
                ..
            }
        ));

        let join = join!(Alpha RIGHT OUTER JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
        assert!(matches!(
            join,
            Join {
                join: JoinType::Right,
                ..
            }
        ));

        let join = join!(Alpha RIGHT JOIN Bravo ON AlphaColumn::a == BravoColumn::second);
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
            column: u128,
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
}
