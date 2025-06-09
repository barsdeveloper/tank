use crate::{decode_column::ColumnMetadata, encode_column_ref::encode_column_ref};
use proc_macro2::TokenStream;
use quote::quote;

pub fn encode_column_def(metadata: &ColumnMetadata) -> TokenStream {
    let reference = encode_column_ref(metadata);
    let column_type = &metadata.column_type;
    let value = &metadata.value;
    let nullable = &metadata.nullable;
    let default = &metadata.default;
    let primary_key = &metadata.primary_key;
    let unique = &metadata.unique;
    let auto_increment = &metadata.auto_increment;
    let passive = &metadata.passive;
    quote! {
        ::tank::ColumnDef {
            reference: #reference,
            column_type: #column_type,
            value: #value,
            nullable: #nullable,
            default: None,
            primary_key: #primary_key,
            unique: #unique,
            auto_increment: #auto_increment,
            passive: #passive,
        }
    }
}
