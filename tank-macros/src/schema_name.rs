use convert_case::{Case, Casing};
use syn::{ItemStruct, LitStr};

pub(crate) fn schema_name(item: &ItemStruct) -> String {
    let default_table_name = item.ident.to_string().to_case(Case::Snake);
    item.attrs
        .iter()
        .find_map(|attr| {
            if attr.meta.path().is_ident("schema_name") {
                let Ok(v) = attr
                    .meta
                    .require_list()
                    .and_then(|v| v.parse_args::<LitStr>())
                else {
                    panic!(
                        "Error while parsing `schema_name`, use it like #[schema_name(\"{}\")]",
                        &default_table_name
                    );
                };
                return Some(v.value());
            }
            None
        })
        .unwrap_or(default_table_name)
}
