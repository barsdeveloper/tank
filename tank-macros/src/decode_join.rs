use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, TokenStreamExt};
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
macro_rules! take_until {
    ($original:expr, $($parser:expr),+ $(,)?) => {{
        let macro_local_input = $original.fork();
        let mut macro_local_result = (
            TokenStream::new(),
            ($({
                let _ = $parser;
                None
            }),+),
        );
        loop {
            if macro_local_input.is_empty() {
                break;
            }
            let mut parsed = false;
            let produced = ($({
                let attempt = macro_local_input.fork();
                if let Ok(content) = ($parser)(&attempt) {
                    macro_local_input.advance_to(&attempt);
                    parsed = true;
                    Some(content)
                } else {
                    None
                }
            }),+);
            if parsed {
                macro_local_result.1 = produced;
                break;
            }
            macro_local_result.0.append(macro_local_input.parse::<TokenTree>()?);
        }
        $original.advance_to(&macro_local_input);
        macro_local_result
    }};
}

fn parse_join_rhs(original: ParseStream, join: JoinType, lhs: TokenStream) -> Result<TokenStream> {
    let input = original.fork();
    let (rhs, on, expr_type, chained_join) = {
        custom_keyword!(ON);
        let (rhs, (on, chained_join)) = take_until!(
            input,
            ParseBuffer::parse::<ON>,
            ParseBuffer::parse::<JoinType>,
        );
        let Ok(rhs) = parse2::<JoinMemberParsed>(rhs) else {
            return Err(input.error(
                "Expected to find a right hand side after the join, a table or a nested join",
            ));
        };
        let (expr, expr_type, chained_join) = if on.is_some() {
            let (expr, chained_join) = take_until!(input, ParseBuffer::parse::<JoinType>);
            let Ok(expr) = parse2::<Expr>(expr) else {
                return Err(input.error("Expected to find a valid expression after `ON`"));
            };
            (
                quote! { Some(::tank::expr!(#expr)) },
                quote! { _ },
                chained_join,
            )
        } else {
            (quote! { None }, quote! { () }, chained_join)
        };
        (rhs, expr, expr_type, chained_join)
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
        if original.peek(Paren) {
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
        let (lhs, join) = take_until!(input, ParseBuffer::parse::<JoinType>);
        let Some(join) = join else {
            return Err(input.error("Expected to find join keywords in the input"));
        };
        let lhs = parse2::<JoinMemberParsed>(lhs)?.0;
        let parsed = parse_join_rhs(&input, join, lhs)?;

        original.advance_to(&input);
        Ok(Self(parsed))
    }
}

impl Parse for JoinMemberParsed {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(join) = input.parse::<JoinParsed>() {
            Ok(Self(join.0))
        } else if let Ok(table) = input.parse::<Path>() {
            Ok(Self(quote! { #table::table_ref() }))
        } else if let Ok(table) = input.parse::<Ident>() {
            Ok(Self(quote! { #table::table_ref() }))
        } else {
            Err(input.error("Expected a table or some nested join"))
        }
    }
}
