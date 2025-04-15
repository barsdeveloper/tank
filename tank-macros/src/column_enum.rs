use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, ItemStruct};

use crate::decode_fields::decode_field;

pub(crate) fn column_enum(item: &ItemStruct) -> TokenStream {
    let it = item.fields.iter();
    let columns_enum = it
        .clone()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Punctuated<_, Comma>>();
    let match_variants = it.clone().map(|field| {
        let column_name = field.ident.as_ref().unwrap();
        let column_def = decode_field(&field);
        quote! { Column::#column_name => #column_def }
    });
    quote! {
        #[allow(non_camel_case_types)]
        pub enum Column {
            #columns_enum
        }
        impl ::tank::ColumnTrait for Column {
            fn def(&self) -> ::tank::ColumnDef {
                match &self {
                    #(#match_variants,)*
                    _ => panic!("Unexpected column type"),
                }
            }
        }
    }
}
