use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::{borrow::Cow, cmp::min, ffi::CString};
use syn::Path;

#[derive(Clone)]
pub enum EitherIterator<A, B>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
{
    Left(A),
    Right(B),
}
impl<A, B> Iterator for EitherIterator<A, B>
where
    A: Iterator,
    B: Iterator<Item = A::Item>,
{
    type Item = A::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIterator::Left(a) => a.next(),
            EitherIterator::Right(b) => b.next(),
        }
    }
}

pub fn quote_cow<T: ToOwned + ToTokens + ?Sized>(value: &Cow<T>) -> TokenStream
where
    <T as ToOwned>::Owned: ToTokens,
{
    match value {
        Cow::Borrowed(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
        Cow::Owned(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
    }
}

pub fn quote_option<T: ToTokens>(value: &Option<T>) -> TokenStream {
    match value {
        None => quote! { None },
        Some(v) => quote! { Some(#v) },
    }
}

pub fn matches_path(path: &Path, expect: &[&str]) -> bool {
    let len = min(path.segments.len(), expect.len());
    path.segments
        .iter()
        .rev()
        .take(len)
        .map(|v| &v.ident)
        .eq(expect.iter().rev().take(len))
}

pub fn separated_by<T, F>(
    out: &mut String,
    values: impl IntoIterator<Item = T>,
    mut f: F,
    separator: &str,
) where
    F: FnMut(&mut String, T),
{
    let mut len = out.len();
    for v in values {
        if out.len() > len {
            out.push_str(separator);
        }
        len = out.len();
        f(out, v);
    }
}

pub fn as_c_string<S: Into<Vec<u8>>>(str: S) -> CString {
    CString::new(str.into()).expect("Expected a valid C string")
}

pub fn consume_while<'s>(input: &mut &'s str, predicate: impl FnMut(&char) -> bool) -> &'s str {
    let len = input.chars().take_while(predicate).count();
    if len == 0 {
        return "";
    }
    let result = &input[..len];
    *input = &input[len..];
    result
}

#[macro_export]
macro_rules! possibly_parenthesized {
    ($buff:ident, $cond:expr, $v:expr) => {
        if $cond {
            $buff.push('(');
            $v;
            $buff.push(')');
        } else {
            $v;
        }
    };
}

#[macro_export]
macro_rules! truncate_long {
    ($query:expr) => {
        format_args!(
            "{}{}\n",
            &$query[..::std::cmp::min($query.len(), 497)].trim_end(),
            if $query.len() > 497 { "..." } else { "" },
        )
    };
}

/// Sends the value through the channel and logs in case of error.
#[macro_export]
macro_rules! send_value {
    ($tx:ident, $value:expr) => {{
        if let Err(e) = $tx.send($value) {
            log::error!("{:#}", e);
        }
    }};
}

/// Accumulates the tokens until a parser matches.
///
/// It returns the accumulated `TokenStream` as well as the result of the match (if any).
#[macro_export]
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
