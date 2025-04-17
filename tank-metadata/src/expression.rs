use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{punctuated::Punctuated, token::Comma, BinOp, Expr, ExprLit};

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
pub enum Operator {
    Negative,
    Exponentiation,
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
    Not,
    And,
    Or,
}

pub struct BinaryOp<L: Expression, R: Expression> {
    pub op: Operator,
    pub lhs: L,
    pub rhs: R,
}
impl<L: Expression, R: Expression> Expression for BinaryOp<L, R> {}

pub struct UnaryOp<V: Expression> {
    pub op: Operator,
    pub v: V,
}
impl<V: Expression> Expression for UnaryOp<V> {}

// impl ToTokens for Operand {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         tokens.append_all(match self {
//             Operand::LitBool(v) => quote! { ::tank::Operand::LitBool(#v) },
//             Operand::LitFloat(v) => quote! { ::tank::Operand::LitFloat(#v) },
//             Operand::LitIdent(v) => quote! { ::tank::Operand::LitIdent(#v) },
//             Operand::LitInt(v) => quote! { ::tank::Operand::LitInt(#v) },
//             Operand::LitStr(v) => quote! { ::tank::Operand::LitStr(#v) },
//             Operand::Column(v) => quote! { ::tank::ColumnDef{ name: #v, ..Default::default()} },
//         });
//     }
// }

// impl ToTokens for Operator {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         tokens.append_all(match self {
//             Operator::Negative => quote! { ::tank::Operator::Negative },
//             Operator::Exponentiation => quote! { ::tank::Operator::Exponentiation },
//             Operator::Multiplication => quote! { ::tank::Operator::Multiplication },
//             Operator::Division => quote! { ::tank::Operator::Division },
//             Operator::Remainder => quote! { ::tank::Operator::Remainder },
//             Operator::Addition => quote! { ::tank::Operator::Addition },
//             Operator::Subtraction => quote! { ::tank::Operator::Subtraction },
//             Operator::ShiftLeft => quote! { ::tank::Operator::ShiftLeft },
//             Operator::ShiftRight => quote! { ::tank::Operator::ShiftRight },
//             Operator::BitwiseAnd => quote! { ::tank::Operator::BitwiseAnd },
//             Operator::BitwiseOr => quote! { ::tank::Operator::BitwiseOr },
//             Operator::Equal => quote! { ::tank::Operator::Equal },
//             Operator::NotEqual => quote! { ::tank::Operator::NotEqual },
//             Operator::Less => quote! { ::tank::Operator::Less },
//             Operator::Greater => quote! { ::tank::Operator::Greater },
//             Operator::LessEqual => quote! { ::tank::Operator::LessEqual },
//             Operator::GreaterEqual => quote! { ::tank::Operator::GreaterEqual },
//             Operator::Not => quote! { ::tank::Operator::Not },
//             Operator::And => quote! { ::tank::Operator::And },
//             Operator::Or => quote! { ::tank::Operator::Or },
//         });
//     }
// }
