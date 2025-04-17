#[cfg(test)]
mod tests {
    use tank::{BinaryOp, Entity, Operand, Operator, UnaryOp};
    use tank_macros::sql;

    #[test]
    fn simple() {
        let expr = sql!(1 + 2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: Operator::Addition,
                lhs: Operand::LitInt(1),
                rhs: Operand::LitInt(2)
            }
        ));

        let expr = sql!(5 * 1.2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: Operator::Multiplication,
                lhs: Operand::LitInt(5),
                rhs: Operand::LitFloat(1.2)
            }
        ));

        let expr = sql!(true && false);
        assert!(matches!(
            expr,
            BinaryOp {
                op: Operator::And,
                lhs: Operand::LitBool(true),
                rhs: Operand::LitBool(false)
            }
        ));

        let expr = sql!(45 | -90);
        assert!(matches!(
            expr,
            BinaryOp {
                op: Operator::BitwiseOr,
                lhs: Operand::LitInt(45),
                rhs: UnaryOp {
                    op: Operator::Negative,
                    v: Operand::LitInt(90),
                }
            }
        ));
    }

    #[test]
    fn complex() {
        let expr = sql!(90.5 - -0.54 * 2 < 7 / 2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: Operator::Less,
                lhs: BinaryOp {
                    op: Operator::Subtraction,
                    lhs: Operand::LitFloat(90.5),
                    rhs: BinaryOp {
                        op: Operator::Multiplication,
                        lhs: UnaryOp {
                            op: Operator::Negative,
                            v: Operand::LitFloat(0.54),
                        },
                        rhs: Operand::LitInt(2)
                    }
                },
                rhs: BinaryOp {
                    op: Operator::Division,
                    lhs: Operand::LitInt(7),
                    rhs: Operand::LitInt(2)
                }
            }
        ));

        let expr = sql!((2 + 3) * (4 - 1) >> 1 & (8 | 3));
        assert!(matches!(
            expr,
            BinaryOp {
                op: Operator::BitwiseAnd,
                lhs: BinaryOp {
                    op: Operator::ShiftRight,
                    lhs: BinaryOp {
                        op: Operator::Multiplication,
                        lhs: BinaryOp {
                            op: Operator::Addition,
                            lhs: Operand::LitInt(2),
                            rhs: Operand::LitInt(3),
                        },
                        rhs: BinaryOp {
                            op: Operator::Subtraction,
                            lhs: Operand::LitInt(4),
                            rhs: Operand::LitInt(1),
                        }
                    },
                    rhs: Operand::LitInt(1)
                },
                rhs: BinaryOp {
                    op: Operator::BitwiseOr,
                    lhs: Operand::LitInt(8),
                    rhs: Operand::LitInt(3),
                }
            }
        ));

        let expr = sql!(-(-PI) + 2 * (5 % (2 + 1)) == 7 && !(4 < 2));
        assert!(matches!(
            expr,
            BinaryOp {
                op: Operator::And,
                lhs: BinaryOp {
                    op: Operator::Equal,
                    lhs: BinaryOp {
                        op: Operator::Addition,
                        lhs: UnaryOp {
                            op: Operator::Negative,
                            v: UnaryOp {
                                op: Operator::Negative,
                                v: Operand::LitIdent("PI"),
                            }
                        },
                        rhs: BinaryOp {
                            op: Operator::Multiplication,
                            lhs: Operand::LitInt(2),
                            rhs: BinaryOp {
                                op: Operator::Remainder,
                                lhs: Operand::LitInt(5),
                                rhs: BinaryOp {
                                    op: Operator::Addition,
                                    lhs: Operand::LitInt(2),
                                    rhs: Operand::LitInt(1)
                                }
                            }
                        }
                    },
                    rhs: Operand::LitInt(7)
                },
                rhs: UnaryOp {
                    op: Operator::Not,
                    v: BinaryOp {
                        op: Operator::Less,
                        lhs: Operand::LitInt(4),
                        rhs: Operand::LitInt(2)
                    }
                }
            }
        ));
    }

    #[test]
    fn columns() {
        #[derive(Entity)]
        struct MyEntity {
            first: i128,
            second: String,
            third: Vec<f64>,
        }
        let c = Column::first;

        let expr = sql!(1 + 2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: Operator::BitwiseOr,
                lhs: Operand::LitInt(45),
                rhs: UnaryOp {
                    op: Operator::Negative,
                    v: Operand::LitInt(90),
                }
            }
        ));
    }
}
