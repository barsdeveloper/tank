use syn::{
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    token::Comma,
    ItemStruct, LitStr,
};

pub(crate) fn table_primary_key(item: &ItemStruct) -> Vec<String> {
    item
    .attrs
    .iter()
    .find_map(|attr| {
        if attr.meta.path().is_ident("primary_key") {
            let parser =
                |input: ParseStream| Punctuated::<LitStr, Comma>::parse_terminated(input);
            let Ok(v) = attr.meta.require_list().and_then(|v| {
                Ok(parser
                    .parse2(v.tokens.clone())?
                    .into_iter()
                    .map(|v| v.value())
                    .collect::<Vec<_>>())
            }) else {
                panic!("Error while parsing `primary_key`, use it like `#[primary_key(\"first\", \"second\")]`");
            };
            return Some(v);
        }
        None
    })
    .unwrap_or(Default::default())
}
