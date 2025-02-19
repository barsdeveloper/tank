use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Fragment)]
pub fn derive_fragment(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ref name = input.ident;
    if !name.to_string().ends_with("Fragment") {
        panic!("Fragment structure names must end with the 'Fragment' word")
    }
    let actual_name = name
        .to_string()
        .strip_suffix("Fragment")
        .unwrap()
        .to_string();
    let allow_trait_name = proc_macro2::Ident::new(
        &format!("Allow{}", actual_name),
        proc_macro2::Span::call_site(),
    );
    quote! {
        impl tank::Fragment for #name {
            fn as_any_ref(&self) -> &dyn std::any::Any {
                self
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
        pub trait #allow_trait_name {}
    }
    .into()
}
