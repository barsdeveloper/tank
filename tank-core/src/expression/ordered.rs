use crate::{
    Expression, OpPrecedence,
    writer::{Context, SqlWriter},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};
use std::fmt::Write;

#[derive(Debug, Clone, Copy)]
pub enum Order {
    ASC,
    DESC,
}

impl ToTokens for Order {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(match self {
            Order::ASC => quote!(::tank::Order::ASC),
            Order::DESC => quote!(::tank::Order::DESC),
        });
    }
}

#[derive(Debug)]
pub struct Ordered<E: Expression> {
    pub expression: E,
    pub order: Order,
}

impl<E: Expression> OpPrecedence for Ordered<E> {
    fn precedence(&self, writer: &dyn SqlWriter) -> i32 {
        self.expression.precedence(writer)
    }
}

impl<E: Expression> Expression for Ordered<E> {
    fn write_query(&self, writer: &dyn SqlWriter, context: Context, buff: &mut dyn Write) {
        writer.write_expression_ordered(context, buff, &self.expression, self.order);
    }
    fn is_ordered(&self) -> bool {
        true
    }
}
