use crate::decode_column;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemStruct, Type, spanned::Spanned};

pub(crate) fn from_row_trait(item: &ItemStruct) -> (Ident, TokenStream) {
    let struct_name = &item.ident;
    let trait_name = Ident::new(&format!("{}FromRowTrait", item.ident), item.span());
    let factory_name = Ident::new(&format!("{}FromRowFactory", item.ident), item.span());
    let fields = item.fields.iter().map(|f| {
        (
            f.ident.clone().expect("Field identifier is expected"),
            f.ty.clone(),
            decode_column(f),
        )
    });
    let fields_holder_declarations = fields.clone().map(|(ident, ty, _)| {
        quote! {
            let mut #ident: Option<#ty> = None;
        }
    });
    type AssignmentFn = dyn Fn(&Ident, &Type) -> TokenStream;
    type ProducerFn = Box<dyn Fn(&AssignmentFn) -> TokenStream>;
    let field_assignment = fields
        .clone()
        .map(|(ident, ty, col)| {
            Box::new(move |assign: &AssignmentFn| {
                let assign = assign(&ident, &ty);
                let name = col.name.to_string();
                quote! {
                    if __n__ == #name {
                        #assign;
                    }
                }
            }) as ProducerFn
        })
        .reduce(|acc, cur| {
            Box::new(move |assign: &AssignmentFn| {
                let acc = acc(assign);
                let cur = cur(assign);
                quote!(#acc else #cur)
            }) as ProducerFn
        })
        .unwrap_or(Box::new(|_| TokenStream::new()));
    let create_result = fields.clone().map(|(ident, _ty, col)| {
        let column = &col.name;
        quote! {
            #ident: #ident.ok_or(__make_error__(#column))?
        }
    });
    let create_result = quote! {
        #struct_name {
            #(#create_result),*
        }
    };
    let field_assignment_default = field_assignment(
        &|field, ty| quote!(result.#field = <#ty as ::tank::AsValue>::try_from_value(__v__)?),
    );
    let field_assignment_holder = field_assignment(
        &|field, ty| quote!(#field = Some(<#ty as ::tank::AsValue>::try_from_value(__v__)?)),
    );
    (
        factory_name.clone(),
        quote! {
            trait #trait_name {
                fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<#struct_name>;
            }
            struct #factory_name<T>(std::marker::PhantomData<T>);
            impl<T: Default + Into<#struct_name>> #factory_name<T> {
                // Called when T has Default Trait
                fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<#struct_name> {
                    let mut result = T::default().into();
                    for (__n__, __v__) in ::std::iter::zip(row.labels.iter(), row.values.into_iter())
                    {
                        #field_assignment_default
                    }
                    Ok(result)
                }
            }
            impl<T> #trait_name for #factory_name<T> {
                // Called when T doesn't have default trait
                fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<#struct_name> {
                    #(#fields_holder_declarations)*
                    for (__n__, __v__) in ::std::iter::zip(row.labels.iter(), row.values.into_iter())
                    {
                        #field_assignment_holder
                    }
                    let __make_error__ = |name: &str| ::tank::Error::msg(format!(
                        "Column `{}` does not exist in the row provided",
                        name
                    ));
                    Ok(#create_result)
                }
            }
        },
    )
}
