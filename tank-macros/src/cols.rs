use syn::{
    Expr, Ident, Token, custom_keyword,
    parse::{Parse, ParseStream},
};
use tank_core::Order;

pub(crate) struct ColItem {
    pub(crate) expr: Expr,
    pub(crate) order: Option<Order>,
}

impl Parse for ColItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<Expr>()?;
        let order = if input.peek(Ident) {
            custom_keyword!(ASC);
            custom_keyword!(DESC);
            if let Ok(..) = input.parse::<ASC>() {
                Some(Order::ASC)
            } else if let Ok(..) = input.parse::<DESC>() {
                Some(Order::DESC)
            } else {
                panic!(
                    "Unexpected keyword `{}` after column, use either ASC or DESC",
                    input
                );
            }
        } else {
            None
        };
        Ok(ColItem { expr, order })
    }
}

pub(crate) struct ColList {
    pub(crate) cols: Vec<ColItem>,
}

impl Parse for ColList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ColList {
            cols: input
                .parse_terminated(ColItem::parse, Token![,])?
                .into_iter()
                .collect(),
        })
    }
}
