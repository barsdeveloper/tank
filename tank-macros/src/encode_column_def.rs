use crate::decode_column::ColumnMetadata;
use proc_macro2::TokenStream;
use quote::quote;

pub fn encode_column_def(metadata: &ColumnMetadata, reference: TokenStream) -> TokenStream {
    let column_type = &metadata.column_type;
    let value = &metadata.value;
    let nullable = &metadata.nullable;
    let default = metadata
        .default
        .as_ref()
        .map_or(quote!(None), |v| quote!(Some(Box::new(#v))));
    let primary_key = &metadata.primary_key;
    let unique = &metadata.unique;
    let auto_increment = &metadata.auto_increment;
    let passive = &metadata.passive;
    let comment = &metadata.comment;
    quote! {
        ::tank::ColumnDef {
            reference: #reference,
            column_type: #column_type,
            value: #value,
            nullable: #nullable,
            default: #default,
            primary_key: #primary_key,
            unique: #unique,
            auto_increment: #auto_increment,
            passive: #passive,
            comment: #comment,
        }
    }
}
