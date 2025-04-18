#[cfg(test)]
mod tests {
    use tank::{BinaryOp, BinaryOpType, ColumnTrait, Operand, UnaryOp, UnaryOpType};
    use tank_macros::{sql, Entity};

    #[test]
    fn simple() {
        let expr = sql!(1 + 2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Addition,
                lhs: Operand::LitInt(1),
                rhs: Operand::LitInt(2)
            }
        ));

        let expr = sql!(5 * 1.2);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::Multiplication,
                lhs: Operand::LitInt(5),
                rhs: Operand::LitFloat(1.2)
            }
        ));

        let expr = sql!(true && false);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::And,
                lhs: Operand::LitBool(true),
                rhs: Operand::LitBool(false)
            }
        ));

        let expr = sql!(45 | -90);
        assert!(matches!(
            expr,
            BinaryOp {
                op: BinaryOpType::BitwiseOr,
                lhs: Operand::LitInt(45),
                rhs: UnaryOp {
                    op: UnaryOpType::Negative,
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
                        rhs: Operand::LitInt(2)
                    }
                },
                rhs: BinaryOp {
                    op: BinaryOpType::Division,
                    lhs: Operand::LitInt(7),
                    rhs: Operand::LitInt(2)
                }
            }
        ));

        let expr = sql!((2 + 3) * (4 - 1) >> 1 & (8 | 3));
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
                        }
                    },
                    rhs: Operand::LitInt(1)
                },
                rhs: BinaryOp {
                    op: BinaryOpType::BitwiseOr,
                    lhs: Operand::LitInt(8),
                    rhs: Operand::LitInt(3),
                }
            }
        ));

        let expr = sql!(-(-PI) + 2 * (5 % (2 + 1)) == 7 && !(4 < 2));
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
                            }
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
                                    rhs: Operand::LitInt(1)
                                }
                            }
                        }
                    },
                    rhs: Operand::LitInt(7)
                },
                rhs: UnaryOp {
                    op: UnaryOpType::Not,
                    v: BinaryOp {
                        op: BinaryOpType::Less,
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

        let expr = sql!(MyEntityColumn::first + 2);
        // assert!(matches!(
        //     expr,
        //     BinaryOp {
        //         op: BinaryOpType::BitwiseOr,
        //         lhs: Operand::LitInt(45),
        //         rhs: UnaryOp {
        //             op: UnaryOpType::Negative,
        //             v: Operand::LitInt(90),
        //         }
        //     }
        // ));
    }
}
