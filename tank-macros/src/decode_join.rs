use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    custom_keyword, parenthesized,
    parse::{discouraged::Speculative, Parse, ParseBuffer, ParseStream},
    parse2,
    token::Paren,
    Expr, Ident, Path, Result,
};
use tank_core::JoinType;

pub(crate) struct JoinParsed(pub(crate) TokenStream);
struct JoinTypeParsed(pub(crate) TokenStream, JoinType);
struct JoinMemberParsed(pub(crate) TokenStream);

/// Accumulates the tokens until a parser matches.
///
/// It returns the accumulated `TokenStream` as well as the result of the match (if any).
macro_rules! accumulate_until {
    ($original:expr, $($parser:expr),+ $(,)?) => {{
        let input = $original.fork();
        let mut result = (TokenStream::new(), None);
        loop {
            if input.is_empty() {
                break;
            }
            $(
                let attempt = input.fork();
                if let Ok(parsed) = ($parser)(&attempt) {
                    input.advance_to(&attempt);
                    result.1 = Some(parsed);
                    break;
                }
            )+
            result.0.append(input.parse::<TokenTree>()?);
        }
        $original.advance_to(&input);
        result
    }};
}

fn parse_join_rhs(original: ParseStream, join: JoinType, lhs: TokenStream) -> Result<TokenStream> {
    let input = original.fork();
    let (rhs, on, expr_type, chained_join) = if join.has_on_clause() {
        custom_keyword!(ON);
        let rhs = accumulate_until!(input, |p| ParseBuffer::parse::<ON>(p).map(|_| None), |p| {
            ParseBuffer::parse::<JoinType>(p).map(|v| Some(v))
        },);
        let (rhs, chained_join) = (parse2::<JoinMemberParsed>(rhs.0)?, rhs.1.flatten());
        let (expr, chained_join) = {
            let mut accumulated = TokenStream::new();
            loop {
                if input.is_empty() {
                    break (parse2::<Expr>(accumulated)?, None);
                }
                let attempt = input.fork();
                if let Ok(join) = attempt.parse::<JoinType>() {
                    input.advance_to(&attempt);
                    break (parse2::<Expr>(accumulated)?, Some(join));
                }
                accumulated.append(input.parse::<TokenTree>()?);
            }
        };
        (
            rhs,
            quote! { Some(::tank::expr!(#expr)) },
            quote! { _ },
            chained_join,
        )
    } else {
        let (rhs, chained_join) = accumulate_until!(input, ParseBuffer::parse::<JoinType>);
        let rhs = parse2::<JoinMemberParsed>(rhs)?;
        (rhs, quote! { None }, quote! { () }, chained_join)
    };
    let rhs = rhs.0;
    let result = quote! {
        ::tank::Join::<_, _, #expr_type> {
            join: #join,
            lhs: #lhs,
            rhs: #rhs,
            on: #on,
        }
    };
    original.advance_to(&input);
    if let Some(chained) = chained_join {
        parse_join_rhs(original, chained, result)
    } else {
        Ok(result)
    }
}

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
        let (join, lhs) = {
            let mut accumulated = TokenStream::new();
            let join: JoinType = loop {
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
            (join, lhs)
        };

        original.advance_to(&input);
        let parsed = parse_join_rhs(original, join, lhs.0)?;
        Ok(Self(parsed))
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
