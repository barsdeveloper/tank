use crate::{data_type, DataType};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    // /// `DEFAULT <restricted-expr>`
    // pub default: Option<Expr>,
    // /// `{ PRIMARY KEY | UNIQUE }`
    // pub unique: Option<ColumnUniqueOption>,
    // pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnRef {
    pub name: String,
}

impl ColumnRef {
    pub fn new(name: String) -> ColumnRef {
        ColumnRef { name }
    }
}

impl From<&str> for ColumnRef {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for ColumnRef {
    fn from(value: String) -> Self {
        ColumnRef::new(value)
    }
}

// impl ToTokens for ColumnDef {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         let name = Ident::new(&self.name, Span::call_site());
//         let data_type = tokens.append_all(quote! {
//             tank::ColumnDef {
//                 name: #name,
//             }
//         });
//     }
// }
