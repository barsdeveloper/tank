use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    custom_keyword, parenthesized,
    parse::{discouraged::Speculative, Parse, ParseBuffer, ParseStream},
    parse2,
    token::Paren,
    Expr, ExprParen, Ident, Path, Result,
};
use tank_core::JoinType;

pub(crate) struct JoinParsed(pub(crate) TokenStream);
struct JoinTypeParsed(pub(crate) TokenStream, JoinType);
struct JoinMemberParsed(pub(crate) TokenStream);

trait UnwrapParentheses {
    fn unwrap_parentheses(self) -> Self;
}

impl UnwrapParentheses for ParseBuffer<'_> {
    fn unwrap_parentheses(self) -> Self {
        if self.peek(Paren) {
            let attempt = self.fork();
            let content = (|| {
                let content;
                parenthesized!(content in attempt);
                Ok(content)
            })()
            .expect("Must be parentheses by this point");
            if attempt.is_empty() {
                self.advance_to(&attempt);
                return content.unwrap_parentheses();
            }
        }
        self
    }
}

impl UnwrapParentheses for TokenStream {
    fn unwrap_parentheses(self) -> Self {
        let original = self.into_iter();
        let mut it = original.clone();
        if let [Some(TokenTree::Group(content)), None] = [it.next(), it.next()] {
            if matches!(content.delimiter(), Delimiter::Parenthesis) {
                return content.stream().unwrap_parentheses();
            }
        }
        original.collect::<TokenStream>()
    }
}

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
        let rhs = parse2::<JoinMemberParsed>(rhs /*.unwrap_parentheses()*/)?;
        let (expr, expr_type, chained_join) = if on.is_some() {
            let (expr, chained_join) = take_until!(input, ParseBuffer::parse::<JoinType>);
            let expr = parse2::<Expr>(expr)?;
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
        let input = original.fork()/* .unwrap_parentheses()*/;

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
