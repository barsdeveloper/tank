use crate::decode_column::ColumnMetadata;
use proc_macro2::TokenStream;
use quote::quote;

pub fn encode_column_ref(metadata: &ColumnMetadata) -> TokenStream {
    let name = &metadata.name;
    let table = &metadata.table;
    let schema = &metadata.schema;
    quote! {
        ::tank::ColumnRef {
            name: #name,
            table: #table,
            schema: #schema,
        }
    }
}
