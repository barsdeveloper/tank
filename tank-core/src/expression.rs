pub trait Expression {}

#[derive(Debug, PartialEq)]
pub enum Operand {
    // Function(String, Vec<String>),
    LitBool(bool),
    LitFloat(f64),
    LitIdent(&'static str),
    LitInt(i128),
    LitStr(String),
    Column(String),
}

impl Expression for Operand {}

#[derive(Debug, PartialEq)]
pub enum UnaryOpType {
    Negative,
    Not,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOpType {
    ArrayIndexing,
    Cast,
    Multiplication,
    Division,
    Remainder,
    Addition,
    Subtraction,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseOr,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

pub struct UnaryOp<V: Expression> {
    pub op: UnaryOpType,
    pub v: V,
}
impl<V: Expression> Expression for UnaryOp<V> {}

pub struct BinaryOp<L: Expression, R: Expression> {
    pub op: BinaryOpType,
    pub lhs: L,
    pub rhs: R,
}
impl<L: Expression, R: Expression> Expression for BinaryOp<L, R> {}
