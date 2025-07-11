#![feature(assert_matches)]
mod average;
mod trade;

use crate::trade::{trade_multiple, trade_simple};
use tank::Connection;

pub async fn execute_tests<C: Connection>(connection: &mut C) {
    trade_simple(connection).await;
    trade_multiple(connection).await;
}
