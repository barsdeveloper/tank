use syn::{ItemStruct, LitStr};

pub(crate) fn schema_name(item: &ItemStruct) -> String {
    item.attrs
        .iter()
        .find_map(|attr| {
            if attr.meta.path().is_ident("schema_name") {
                let Ok(v) = attr
                    .meta
                    .require_list()
                    .and_then(|v| v.parse_args::<LitStr>())
                else {
                    panic!("Error while parsing `schema_name`, use it like `#[schema_name(\"the_name\")]`");
                };
                return Some(v.value());
            }
            None
        })
        .unwrap_or("".into())
}
