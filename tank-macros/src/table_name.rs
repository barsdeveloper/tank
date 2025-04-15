use convert_case::{Case, Casing};
use syn::{ItemStruct, LitStr};

pub(crate) fn table_name(item: &ItemStruct) -> String {
    let default_table_name = item.ident.to_string().to_case(Case::Snake);
    item.attrs
        .iter()
        .find_map(|attr| {
            if attr.meta.path().is_ident("table_name") {
                let Ok(v) = attr
                    .meta
                    .require_list()
                    .and_then(|v| v.parse_args::<LitStr>())
                else {
                    panic!(
                        "Error while parsing `table_name`, use it like #[table_name(\"{}_table\")]",
                        &default_table_name
                    );
                };
                return Some(v.value());
            }
            None
        })
        .unwrap_or(default_table_name)
}
