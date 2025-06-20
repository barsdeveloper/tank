use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Ident, ItemStruct, Type};

pub(crate) fn from_row_trait(item: &ItemStruct) -> (Ident, TokenStream) {
    let struct_name = &item.ident;
    let trait_name = Ident::new(&format!("{}FromRowTrait", item.ident), item.span());
    let factory_name = Ident::new(&format!("{}FromRowFactory", item.ident), item.span());
    let fields = item.fields.iter().map(|f| {
        (
            f.ident.clone().expect("Field identifier is expected"),
            f.ty.clone(),
        )
    });
    let fields_holder_declarations = fields.clone().map(|(ident, ty)| {
        quote! {
            let mut #ident: Option<#ty> = None;
        }
    });
    type AssignmentFn = dyn Fn(&Ident, &Type) -> TokenStream;
    type ProducerFn = Box<dyn Fn(&AssignmentFn) -> TokenStream>;
    let field_assignment = fields
        .clone()
        .map(|(ident, ty)| {
            Box::new(move |assign: &AssignmentFn| {
                let assign = assign(&ident, &ty);
                let name = ident.to_string();
                quote! {
                    if name == #name {
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
    let create_result = fields.map(|(ident, _ty)| {
        let field_name = ident.to_string();
        quote! {
            #ident: #ident.ok_or(::tank::Error::msg(format!(
                "Field `{}` does not exist in the row provided",
                #field_name
            )))?
        }
    });
    let create_result = quote! {
        #struct_name {
            #(#create_result),*
        }
    };
    let field_assignment_default = field_assignment(
        &|field, ty| quote!(result.#field = <#ty as ::tank::AsValue>::try_from_value(value)?),
    );
    let field_assignment_holder = field_assignment(
        &|field, ty| quote!(#field = Some(<#ty as ::tank::AsValue>::try_from_value(value)?)),
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
                    // TODO: Remove into_vec when consuming iterator will be possible on boxed slices
                    ::std::iter::zip(row.labels.iter(), row.values.into_vec().into_iter()).try_for_each(|(name, value)| {
                        #field_assignment_default
                        Ok::<_, ::tank::Error>(())
                    });
                    Ok(result)
                }
            }
            impl<T> #trait_name for #factory_name<T> {
                // Called when T doesn't have default trait
                fn from_row(row: ::tank::RowLabeled) -> ::tank::Result<#struct_name> {
                    #(#fields_holder_declarations)*
                    // TODO: Remove into_vec when consuming iterator will be possible on boxed slices
                    ::std::iter::zip(row.labels.iter(), row.values.into_vec().into_iter()).try_for_each(|(name, value)| {
                        #field_assignment_holder
                        Ok::<_, ::tank::Error>(())
                    });
                    Ok(#create_result)
                }
            }
        },
    )
}
