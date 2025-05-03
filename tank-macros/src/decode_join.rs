use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    custom_keyword, parenthesized,
    parse::{discouraged::Speculative, Parse, ParseStream},
    parse2,
    token::Paren,
    Expr, Ident, Path, Result,
};
use tank_core::JoinType;

pub(crate) struct JoinParsed(pub(crate) TokenStream);
struct JoinTypeParsed(pub(crate) TokenStream, JoinType);
struct JoinMemberParsed(pub(crate) TokenStream);

impl Parse for JoinParsed {
    fn parse(original: ParseStream) -> Result<Self> {
        let input = original.fork();
        // Unwrap eventual parentheses
        if input.peek(Paren) {
            let attempt = input.fork();
            let content;
            parenthesized!(content in attempt);
            if attempt.is_empty() {
                // The whole join can be parenthesized
                original.advance_to(&attempt);
                return Self::parse(&content);
            }
        }

        // Base case
        let mut accumulated = TokenStream::new();
        let join = loop {
            if input.is_empty() {
                return Err(input.error("Expected to find join keywords in the input"));
            }
            let attempt = input.fork();
            if let Ok(join) = attempt.parse::<JoinType>() {
                input.advance_to(&attempt);
                break join;
            }
            accumulated.append(input.parse::<TokenTree>()?);
        };
        let lhs = parse2::<JoinMemberParsed>(accumulated)?;
        let (rhs, on, expr_type) = if join.has_on_clause() {
            custom_keyword!(ON);
            let mut accumulated = TokenStream::new();
            let rhs = loop {
                if input.is_empty() {
                    return Err(input.error("Expected ON clause"));
                }
                let attempt = input.fork();
                if let Ok(..) = attempt.parse::<ON>() {
                    input.advance_to(&attempt);
                    break parse2::<JoinMemberParsed>(accumulated)?;
                }
                accumulated.append(input.parse::<TokenTree>()?);
            };
            let expr = input.parse::<Expr>()?;
            (rhs, quote! { Some(::tank::expr!(#expr)) }, quote! { _ })
        } else {
            (
                input.parse::<JoinMemberParsed>()?,
                quote! { None },
                quote! { () },
            )
        };
        let lhs = lhs.0;
        let rhs = rhs.0;
        original.advance_to(&input);
        Ok(Self(quote! {
            ::tank::Join::<_, _, #expr_type> {
                join: #join,
                lhs: #lhs,
                rhs: #rhs,
                on: #on,
            }
        }))
    }
}

impl Parse for JoinMemberParsed {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(join) = input.parse::<JoinParsed>() {
            return Ok(Self(join.0));
        }
        if let Ok(table) = input.parse::<Path>() {
            return Ok(Self(quote! { #table::table_ref() }));
        }
        if let Ok(table) = input.parse::<Ident>() {
            return Ok(Self(quote! { #table::table_ref() }));
        }
        Err(input.error(""))
    }
}
