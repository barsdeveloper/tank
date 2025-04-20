use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, BinOp, Expr, ExprLit, LitStr};
use tank_core::decode_type;

pub fn decode_expression(condition: &Expr) -> TokenStream {
    match condition {
        Expr::Binary(v) => {
            let op = match v.op {
                BinOp::Add(..) => quote! { ::tank::BinaryOpType::Addition },
                BinOp::Sub(..) => quote! { ::tank::BinaryOpType::Subtraction},
                BinOp::Mul(..) => quote! { ::tank::BinaryOpType::Multiplication},
                BinOp::Div(..) => quote! { ::tank::BinaryOpType::Division},
                BinOp::Rem(..) => quote! { ::tank::BinaryOpType::Remainder},
                BinOp::And(..) => quote! { ::tank::BinaryOpType::And},
                BinOp::Or(..) => quote! { ::tank::BinaryOpType::Or },
                BinOp::BitAnd(..) => quote! { ::tank::BinaryOpType::BitwiseAnd },
                BinOp::BitOr(..) => quote! { ::tank::BinaryOpType::BitwiseOr },
                BinOp::Shl(..) => quote! { ::tank::BinaryOpType::ShiftLeft },
                BinOp::Shr(..) => quote! { ::tank::BinaryOpType::ShiftRight },
                BinOp::Eq(..) => quote! { ::tank::BinaryOpType::Equal },
                BinOp::Lt(..) => quote! { ::tank::BinaryOpType::Less },
                BinOp::Le(..) => quote! { ::tank::BinaryOpType::LessEqual },
                BinOp::Ne(..) => quote! { ::tank::BinaryOpType::NotEqual },
                BinOp::Ge(..) => quote! { ::tank::BinaryOpType::GreaterEqual },
                BinOp::Gt(..) => quote! { ::tank::BinaryOpType::Greater },
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
        Expr::Index(v) => {
            let lhs = decode_expression(&v.expr);
            let rhs = decode_expression(&v.index);
            quote! {
                ::tank::BinaryOp {
                    op: ::tank::BinaryOpType::Indexing,
                    lhs: #lhs,
                    rhs: #rhs,
                }
            }
        }
        Expr::Cast(v) => {
            let lhs = decode_expression(&v.expr);
            let rhs = decode_type(match v.ty.as_ref() {
                syn::Type::Array(type_array) => todo!(),
                syn::Type::BareFn(type_bare_fn) => todo!(),
                syn::Type::Group(type_group) => todo!(),
                syn::Type::ImplTrait(type_impl_trait) => todo!(),
                syn::Type::Infer(type_infer) => todo!(),
                syn::Type::Macro(type_macro) => todo!(),
                syn::Type::Never(type_never) => todo!(),
                syn::Type::Paren(type_paren) => todo!(),
                syn::Type::Path(v) => &v.path,
                syn::Type::Ptr(type_ptr) => todo!(),
                syn::Type::Reference(type_reference) => todo!(),
                syn::Type::Slice(type_slice) => todo!(),
                syn::Type::TraitObject(type_trait_object) => todo!(),
                syn::Type::Tuple(type_tuple) => todo!(),
                syn::Type::Verbatim(token_stream) => todo!(),
                _ => todo!(),
            })
            .0;
            quote! {
                ::tank::BinaryOp {
                    op: ::tank::BinaryOpType::Cast,
                    lhs: #lhs,
                    rhs: ::tank::Operand::Type(#rhs),
                }
            }
        }
        Expr::Unary(v) => {
            let op = match v.op {
                syn::UnOp::Not(..) => quote! { ::tank::UnaryOpType::Not },
                syn::UnOp::Neg(..) => quote! { ::tank::UnaryOpType::Negative },
                _ => panic!("Unsupported operator `{}`", v.op.to_token_stream()),
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
            if v.path
                .segments
                .iter()
                .rev()
                .nth(1)
                .map_or(false, |v| v.ident.to_string().ends_with("Column"))
            {
                let path = &v.path;
                quote! { ::tank::Operand::Column(::tank::ColumnTrait::def(&#path)) }
            } else {
                let v = LitStr::new(&v.path.to_token_stream().to_string(), v.path.span());
                quote! { ::tank::Operand::LitIdent(#v) }
            }
        }
        Expr::Array(v) => {
            let v = v
                .elems
                .iter()
                .map(|v| decode_expression(v))
                .collect::<Punctuated<_, Comma>>();
            quote! { ::tank::Operand::LitArray(&[#v]) }
        }
        _ => panic!(
            "Unexpected expression `{}`",
            condition.to_token_stream().to_string()
        ),
    }
}
