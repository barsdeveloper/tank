use crate::decode_fields::decode_field;
use syn::{
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    token::Comma,
    ItemStruct, LitStr,
};
use tank_core::ColumnDef;

pub(crate) fn table_primary_key(item: &ItemStruct) -> Vec<ColumnDef> {
    item
    .attrs
    .iter()
    .find_map(|attr| {
        if attr.meta.path().is_ident("primary_key") {
            let parser =
                |input: ParseStream| Punctuated::<LitStr, Comma>::parse_terminated(input);
            let Ok(primary_keys) = attr.meta.require_list().and_then(|v| {
                Ok(parser
                    .parse2(v.tokens.clone())?
                    .into_iter()
                    .map(|v| v.value())
                    .collect::<Vec<_>>())
            }) else {
                panic!("Error while parsing `primary_key`, use it like `#[primary_key(\"first\", \"second\")]`");
            };
            let columns = item.fields.iter().map(|f| decode_field(f, item).0);
            let primary_keys = primary_keys
            .iter().map(|pk| columns.clone().find(|col| **col.name() == *pk).expect(&format!("Primary key `{}` is not a field of the entity", pk)));
            return Some(primary_keys.collect());
        }
        None
    })
    .unwrap_or(Default::default())
}
