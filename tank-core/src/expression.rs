use crate::ColumnDef;

pub trait Expression {}

#[derive(Debug)]
pub enum Operand {
    // Function(String, Vec<String>),
    LitBool(bool),
    LitFloat(f64),
    LitIdent(&'static str),
    LitInt(i128),
    LitStr(String),
    Column(ColumnDef),
}
impl Expression for Operand {}
impl PartialEq for Operand {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LitBool(l), Self::LitBool(r)) => l == r,
            (Self::LitFloat(l), Self::LitFloat(r)) => l == r,
            (Self::LitIdent(l), Self::LitIdent(r)) => l == r,
            (Self::LitInt(l), Self::LitInt(r)) => l == r,
            (Self::LitStr(l), Self::LitStr(r)) => l == r,
            (Self::Column(l), Self::Column(r)) => l.name == r.name && l.value.same_type(&r.value),
            _ => false,
        }
    }
}

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
