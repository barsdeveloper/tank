use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Cow;

pub fn quote_cow<T: ToOwned + ToTokens + ?Sized>(value: &Cow<T>) -> TokenStream
where
    <T as ToOwned>::Owned: ToTokens,
{
    match value {
        Cow::Borrowed(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
        Cow::Owned(v) => quote! { ::std::borrow::Cow::Borrowed(#v) },
    }
}
