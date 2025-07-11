use crate::{DataSet, Expression, SqlWriter};
use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt, quote};
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

#[derive(Debug)]
pub struct Join<L: DataSet, R: DataSet, E: Expression> {
    pub join: JoinType,
    pub lhs: L,
    pub rhs: R,
    pub on: Option<E>,
}

#[derive(Default, Clone, Copy, Debug)]
pub enum JoinType {
    #[default]
    Inner,
    Outer,
    Left,
    Right,
    Cross,
    Natural,
}

impl<L: DataSet, R: DataSet, E: Expression> DataSet for Join<L, R, E> {
    fn qualified_columns() -> bool
    where
        Self: Sized,
    {
        true
    }
    fn write_query(&self, writer: &dyn SqlWriter, out: &mut String) {
        writer.write_join(
            out,
            &Join {
                join: self.join,
                lhs: &self.lhs,
                rhs: &self.rhs,
                on: self.on.as_ref().map(|v| v as &dyn Expression),
            },
        );
    }
}

impl Parse for JoinType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tokens = input.cursor().token_stream().into_iter().map(|t| match t {
            TokenTree::Ident(ident) => ident.to_string(),
            _ => "".to_string(),
        });
        let patterns: &[(&[&str], JoinType)] = &[
            (&["INNER", "JOIN"], JoinType::Inner),
            (&["JOIN"], JoinType::Inner),
            (&["FULL", "OUTER", "JOIN"], JoinType::Outer),
            (&["OUTER", "JOIN"], JoinType::Outer),
            (&["LEFT", "OUTER", "JOIN"], JoinType::Left),
            (&["LEFT", "JOIN"], JoinType::Left),
            (&["RIGHT", "OUTER", "JOIN"], JoinType::Right),
            (&["RIGHT", "JOIN"], JoinType::Right),
            (&["CROSS", "JOIN"], JoinType::Cross),
            (&["CROSS"], JoinType::Cross),
            (&["NATURAL", "JOIN"], JoinType::Natural),
        ];
        for (keywords, join_type) in patterns {
            let it = tokens.clone().take(keywords.len());
            if it.eq(keywords.iter().copied()) {
                for _ in 0..keywords.len() {
                    input.parse::<Ident>().expect(&format!(
                        "Unexpected error, the input should contain {:?} as next Ident tokens at this point",
                        keywords
                    ));
                }
                return Ok(*join_type);
            }
        }
        Err(syn::Error::new(input.span(), "Not a join keyword"))
    }
}

impl ToTokens for JoinType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(match self {
            JoinType::Inner => quote! { JoinType::Inner },
            JoinType::Outer => quote! { JoinType::Outer },
            JoinType::Left => quote! { JoinType::Left },
            JoinType::Right => quote! { JoinType::Right },
            JoinType::Cross => quote! { JoinType::Cross },
            JoinType::Natural => quote! { JoinType::Natural },
        });
    }
}
