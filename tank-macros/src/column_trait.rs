use crate::{decode_column::decode_column, encode_column_ref::encode_column_ref};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, ItemStruct};

pub(crate) fn column_trait(item: &ItemStruct) -> TokenStream {
    let struct_name = &item.ident;
    let trait_name = Ident::new(&format!("{}ColumnTrait", item.ident), item.span());
    let columns: Vec<_> = item
        .fields
        .iter()
        .map(|field| {
            (
                field.ident.as_ref().expect("The field must have a name"),
                encode_column_ref(&decode_column(field, item)),
            )
        })
        .collect();
    let columns_fields_declarations = columns.iter().map(|(name, _)| {
        quote! {
            #[allow(non_upper_case_globals)]
            const #name: ::tank::ColumnRef;
        }
    });
    let columns_fields_definitions = columns.iter().map(|(name, column_ref)| {
        quote! {
            const #name: ::tank::ColumnRef = #column_ref;
        }
    });
    quote! {
        trait #trait_name {
            #(#columns_fields_declarations)*
        }
        impl #trait_name for #struct_name {
            #(#columns_fields_definitions)*
        }
    }
}
