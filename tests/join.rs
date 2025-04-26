#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use tank::{BinaryOp, BinaryOpType, ColumnRef, Entity, Join, JoinType, Operand, TableRef};
    use tank_core::join;

    #[test]
    fn join() {
        #[derive(Entity)]
        struct Alpha {
            a: u32,
            b: String,
        }

        #[derive(Entity)]
        struct Bravo {
            first: f64,
            second: String,
        }

        let join = join!(Alpha JOIN Bravo ON AlphaColumn::a == BravoColumn::second);

        assert!(matches!(
            join,
            Join {
                join: JoinType::Inner,
                lhs: TableRef {
                    name: Cow::Borrowed("alpha"),
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
                }),
                ..
            }
        ));
    }
}
