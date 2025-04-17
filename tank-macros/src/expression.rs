use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, BinOp, Expr, ExprLit, LitStr};
use tank_metadata::{Operand, Operator};

pub fn decode_expression(condition: &Expr) -> TokenStream {
    match condition {
        Expr::Binary(v) => {
            let op = match v.op {
                BinOp::Add(..) => quote! { ::tank::Operator::Addition },
                BinOp::Sub(..) => quote! { ::tank::Operator::Subtraction},
                BinOp::Mul(..) => quote! { ::tank::Operator::Multiplication},
                BinOp::Div(..) => quote! { ::tank::Operator::Division},
                BinOp::Rem(..) => quote! { ::tank::Operator::Remainder},
                BinOp::And(..) => quote! { ::tank::Operator::And},
                BinOp::Or(..) => quote! { ::tank::Operator::Or },
                BinOp::BitAnd(..) => quote! { ::tank::Operator::BitwiseAnd },
                BinOp::BitOr(..) => quote! { ::tank::Operator::BitwiseOr },
                BinOp::Shl(..) => quote! { ::tank::Operator::ShiftLeft },
                BinOp::Shr(..) => quote! { ::tank::Operator::ShiftRight },
                BinOp::Eq(..) => quote! { ::tank::Operator::Equal },
                BinOp::Lt(..) => quote! { ::tank::Operator::Less },
                BinOp::Le(..) => quote! { ::tank::Operator::LessEqual },
                BinOp::Ne(..) => quote! { ::tank::Operator::NotEqual },
                BinOp::Ge(..) => quote! { ::tank::Operator::GreaterEqual },
                BinOp::Gt(..) => quote! { ::tank::Operator::Greater },
                _ => todo!(),
            };
            let lhs = decode_expression(&v.left);
            let rhs = decode_expression(&v.right);
            quote! {
                ::tank::BinaryOp {
                    op: #op,
                    lhs: #lhs,
                    rhs: #rhs,
                }
            }
        }
        Expr::Unary(v) => {
            let op = match v.op {
                syn::UnOp::Not(..) => quote! { ::tank::Operator::Not },
                syn::UnOp::Neg(..) => quote! { ::tank::Operator::Negative },
                _ => panic!("Unsupported operator: dereference"),
            };
            let v = decode_expression(v.expr.as_ref());
            quote! {
                ::tank::UnaryOp {
                    op: #op,
                    v: #v,
                }
            }
        }
        Expr::Call(v) => todo!(),
        Expr::Lit(ExprLit { lit: v, .. }) => {
            let v = match v {
                syn::Lit::Str(v) => quote! { ::tank::Operand::LitStr(#v) },
                syn::Lit::Char(v) => quote! { ::tank::Operand::LitStr(#v) },
                syn::Lit::Int(v) => quote! { ::tank::Operand::LitInt(#v) },
                syn::Lit::Float(v) => quote! { ::tank::Operand::LitFloat(#v) },
                syn::Lit::Bool(v) => quote! { ::tank::Operand::LitBool(#v) },
                // syn::Lit::Verbatim(v) => quote! { ::tank::Operand::LitIdent(#v) },
                _ => panic!(
                    "Unexpected value {:?} in a sql expression",
                    v.into_token_stream()
                ),
            };
            quote! { #v }
        }
        Expr::MethodCall(expr_method_call) => todo!(),
        Expr::Paren(v) => decode_expression(&v.expr),
        Expr::Path(v) => {
            let v = LitStr::new(&v.path.to_token_stream().to_string(), v.path.span());
            quote! { ::tank::Operand::LitIdent(#v) }
        }
        _ => todo!("UNKNOWN"),
    }
}

macro_rules! condition {
    ($v:tt) => {
        let tokens = ::quote::quote! { $v };
        let expr: Expr = ::syn::parse_macro_input!(input as ::syn::Expr);
    };
}
