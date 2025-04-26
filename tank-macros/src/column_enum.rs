use crate::decode_fields::decode_field;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, spanned::Spanned, token::Comma, Ident, ItemStruct};

pub(crate) fn column_enum(item: &ItemStruct) -> TokenStream {
    let enum_name = Ident::new(&format!("{}Column", item.ident), item.span());
    let it = item.fields.iter();
    let columns = it
        .clone()
        .map(|field| {
            let column_def = decode_field(&field, &item);
            let column_name = Ident::new(column_def.name(), item.span());
            (column_name, column_def)
        })
        .collect::<Vec<_>>();
    let match_variants = columns
        .iter()
        .map(|(column_name, column_def)| quote! { #enum_name::#column_name => #column_def });
    let columns_enum = columns
        .iter()
        .map(|(column_name, _)| column_name)
        .collect::<Punctuated<_, Comma>>();
    quote! {
        #[allow(non_camel_case_types)]
        pub enum #enum_name {
            #columns_enum
        }
        impl ::tank::ColumnTrait for #enum_name {
            fn column_def(&self) -> ::tank::ColumnDef {
                match &self {
                    #(#match_variants,)*
                    _ => panic!("Unexpected column type"),
                }
            }
            fn column_ref(&self) -> ::tank::ColumnRef {
                self.column_def().into()
            }
        }
    }
}
