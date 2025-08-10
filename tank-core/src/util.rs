use crate::{
    Result,
    stream::{Stream, TryStreamExt},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::{borrow::Cow, cmp::min, fmt::Display};
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

pub fn add_error_context<T, S: Stream<Item = Result<T>>, Q: Display>(
    stream: S,
    query: &Q,
) -> impl Stream<Item = Result<T>> + use<T, S, Q> {
    let query = format!("{}", query).chars().take(500).collect::<String>();
    stream.map_err(move |e| e.context(format!("While executing the query:\n{}", query)))
}

#[macro_export]
macro_rules! possibly_parenthesized {
    ($out:ident, $cond:expr, $v:expr) => {
        if $cond {
            $out.push('(');
            $v;
            $out.push(')');
        } else {
            $v;
        }
    };
}
