use proc_macro2::{Delimiter, Group, Spacing, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::{borrow::Cow, cmp::min};
use syn::Path;

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

pub fn flag_evaluated(input: TokenStream) -> TokenStream {
    fn do_flagging(input: TokenStream) -> TokenStream {
        let mut iter = input.into_iter().peekable();
        std::iter::from_fn(move || {
            while let Some(token) = iter.next() {
                let next = iter.peek().cloned();
                match (&token, next) {
                    (TokenTree::Punct(p), Some(tt))
                        if p.as_char() == '#' && p.spacing() == Spacing::Alone =>
                    {
                        iter.next();
                        let wrapped: TokenStream = quote!(::tank::evaluated!(#tt)).into();
                        return Some(TokenTree::Group(Group::new(
                            Delimiter::None,
                            wrapped.into(),
                        )));
                    }
                    (TokenTree::Group(group), ..) => {
                        let content = do_flagging(group.stream());
                        return Some(TokenTree::Group(Group::new(group.delimiter(), content)));
                    }
                    _ => {}
                }
                return Some(token);
            }
            None
        })
        .collect()
    }
    do_flagging(input)
}
