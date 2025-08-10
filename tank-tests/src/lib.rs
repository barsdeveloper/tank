#![feature(assert_matches)]
#![feature(box_patterns)]
mod aggregates;
mod books;
mod nullability;
mod trade;
mod user;

use crate::{
    books::books,
    nullability::single_null_fields,
    trade::{trade_multiple, trade_simple},
    user::users,
};
use aggregates::aggregates;
use tank::Connection;

pub async fn execute_tests<C: Connection>(connection: &mut C) {
    single_null_fields(connection).await;
    trade_simple(connection).await;
    trade_multiple(connection).await;
    aggregates(connection).await;
    users(connection).await;
    books(connection).await;
}
