use crate::{DataSet, Expression};
use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use std::fmt::{self, Display, Formatter};
use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

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

impl JoinType {
    pub fn has_on_clause(&self) -> bool {
        match self {
            JoinType::Inner | JoinType::Outer | JoinType::Left | JoinType::Right => true,
            JoinType::Cross | JoinType::Natural => false,
        }
    }
}

impl<L: DataSet, R: DataSet, E: Expression> DataSet for Join<L, R, E> {}

impl Display for JoinType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Ok(f.write_str(match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Outer => "OUTER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Cross => "CROSS",
            JoinType::Natural => "NATURAL JOIN",
        })?)
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
