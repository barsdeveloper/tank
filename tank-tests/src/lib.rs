#![feature(assert_matches)]
#![feature(box_patterns)]
mod aggregates;
mod books;
mod complex;
mod insane;
mod simple;
mod trade;
mod user;

use crate::{
    books::books,
    complex::complex,
    insane::insane,
    simple::simple,
    trade::{trade_multiple, trade_simple},
    user::users,
};
use aggregates::aggregates;
use tank::Connection;

pub async fn execute_tests<C: Connection>(connection: &mut C) {
    simple(connection).await;
    trade_simple(connection).await;
    trade_multiple(connection).await;
    aggregates(connection).await;
    users(connection).await;
    books(connection).await;
    complex(connection).await;
    insane(connection).await;
}
