mod decode_fields;

use convert_case::{Case, Casing};
use decode_fields::decode_field;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, ItemStruct, LitStr, Meta};

#[proc_macro_derive(Entity, attributes(table_name, default_value, unique))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    let input: ItemStruct = parse_macro_input!(input as ItemStruct);
    let ref name = input.ident;
    let default_table_name = name.to_string().to_case(Case::Snake);
    let table_name = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.meta.path().is_ident("table_name") {
                if let Meta::List(v) = &attr.meta {
                    let table_name = match v.parse_args::<LitStr>() {
                        Ok(lit_str) => lit_str.value(),
                        Err(e) => {
                            panic!(
                                "Error while parsing `table_name`: {}, use it like #[table_name(\"{}_table\")]",
                                e,
                                &default_table_name
                            );
                        }
                    };
                    return Some(table_name);
                }
            }
            return None;
        })
        .unwrap_or(default_table_name);
    let iter = input.fields.iter();
    let columns_defs = iter
        .clone()
        .map(|f| decode_field(&f).to_token_stream())
        .collect::<Punctuated<_, Comma>>();
    let columns_enum = iter
        .clone()
        .map(|f| f.ident.as_ref().unwrap())
        .collect::<Punctuated<_, Comma>>();
    let match_variants = iter.clone().map(|field| {
        let column_name = field.ident.as_ref().unwrap();
        let column_def = decode_field(&field);
        quote! { Column::#column_name => #column_def }
    });
    //let primary_keys = sql_list_primary_keys(&input).join(", ");
    //fields.iter().map(result, |col|)
    quote! {
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
        impl ::tank::Entity for #name {
            type Column = Column;

            fn table_name() -> &'static str {
                #table_name
            }

            fn columns() -> &'static [::tank::ColumnDef] {
                static columns: &[::tank::ColumnDef] = &[#columns_defs];
                &columns
            }

            fn sql_create_table<D: ::tank::Driver>(driver: &D, if_not_exists: bool) -> String {
                let mut query = String::with_capacity(512);
                driver.sql_writer().sql_create_table::<#name>(&mut query, if_not_exists);
                query
            }

            // fn sql_drop_table(if_exists: bool) -> gluesql::core::ast_builder::DropTableNode {
            //     let result = gluesql::core::ast_builder::table(Self::name());
            //     let result = if if_exists {
            //         result.drop_table_if_exists()
            //     } else {
            //         result.drop_table()
            //     };
            //     result
            // }

            // fn primary_key(&self) -> Vec<gluesql::core::ast::ColumnDef> {
            //     vec![]
            // }
        }

    }
    .into()
}

// #[proc_macro_derive(Transition, attributes(method))]
// pub fn derive_transition(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident.to_string();
//     let fragment_struct = syn::Ident::new(name.as_str(), Span::call_site());
//     let transition_trait = syn::Ident::new(
//         format!("Transition{}", name.strip_suffix("Fragment").unwrap()).as_str(),
//         Span::mixed_site(),
//     );
//     let allow_trait = syn::Ident::new(
//         format!("Allow{}", name.strip_suffix("Fragment").unwrap()).as_str(),
//         Span::mixed_site(),
//     );
//     let method = syn::Ident::new(
//         input
//             .attrs
//             .iter()
//             .find_map(|attr| {
//                 if attr.meta.path().is_ident("method") {
//                     if let Meta::List(v) = &attr.meta {
//                         if let Ok(lit_str) = v.parse_args::<LitStr>() {
//                             return Some(lit_str.value());
//                         }
//                     }
//                     panic!("Error while parsing `method`. Example: like #[method(\"select\")]");
//                 }
//                 return None;
//             })
//             .unwrap_or_else(|| panic!("Expected `method` attribute` like #[method(\"select\")]"))
//             .as_str(),
//         Span::call_site(),
//     );
//     quote! {
//         pub trait #transition_trait {
//             fn #method(self) -> tank::Statement<#fragment_struct>;
//         }
//         impl<S: tank::Fragment> #transition_trait for tank::Statement<S>
//         where
//             S: tank::#allow_trait,
//         {
//             fn #method(self) -> tank::Statement<#fragment_struct> {
//                 self.add_status(Default::default())
//             }
//         }
//     }
//     .into()
// }

// #[proc_macro_derive(ToSql, attributes(keyword))]
// pub fn derive_to_sql(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident;
//     let keyword = input
//         .attrs
//         .iter()
//         .find_map(|attr| {
//             if attr.meta.path().is_ident("keyword") {
//                 if let Meta::List(v) = &attr.meta {
//                     if let Ok(lit_str) = v.parse_args::<LitStr>() {
//                         return Some(lit_str.value());
//                     }
//                 }
//                 panic!(
//                     "Error while parsing `keyword`. Example of correct use: #[keyword(\"SELECT\")]"
//                 );
//             }
//             return None;
//         })
//         .unwrap_or_else(|| {
//             panic!("Expected `keyword` attribute. Example of correct use: #[keyword(\"SELECT\")]")
//         })
//         .to_uppercase();
//     quote! {
//         impl tank::ToSql for #name {
//             fn to_sql(&self) -> String {
//                 #keyword
//             }
//         }
//     }
//     .into()
// }
