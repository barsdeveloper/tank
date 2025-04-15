pub trait Expression {}

pub enum Operand {
    Literal(String),
    Function(String, Vec<String>),
}

impl Expression for Operand {}

pub enum NumericOperator {
    Exponentiation = 0,
    Multiplication,
    Division,
    Remainder,
    Addition,
    Subtraction,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseOr,
}

pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

pub enum LogicOperator {
    And,
    Or,
}

pub struct NumericOperation<L: Expression, R: Expression> {
    pub precedence: u32,
    pub lhs: L,
    pub rhs: R,
}
