use crate::decode_fields::decode_field;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, ItemStruct};

pub(crate) fn column_trait(item: &ItemStruct) -> TokenStream {
    let struct_name = &item.ident;
    let trait_name = Ident::new(&format!("{}ColumnTrait", item.ident), item.span());
    let columns = item.fields.iter().map(|field| {
        (
            field.ident.as_ref().expect("The field must have a name"),
            decode_field(&field, &item).0,
        )
    });
    let columns_fields_declarations = columns.clone().map(|(name, _)| {
        quote! {
            #[allow(non_upper_case_globals)]
            const #name: &::tank::ColumnRef;
        }
    });
    let columns_fields_definitions = columns.clone().map(|(name, def)| {
        let reference = def.reference;
        quote! { const #name: &::tank::ColumnRef = &#reference; }
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
