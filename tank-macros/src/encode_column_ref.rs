use crate::decode_column::ColumnMetadata;
use proc_macro2::TokenStream;
use quote::quote;

pub fn encode_column_ref(metadata: &ColumnMetadata, table: String, schema: String) -> TokenStream {
    let name = &metadata.name;
    quote! {
        ::tank::ColumnRef {
            name: #name,
            table: #table,
            schema: #schema,
        }
    }
}
