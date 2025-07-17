#![feature(assert_matches)]
#![feature(box_patterns)]
mod aggregates;
mod trade;

use crate::trade::{trade_multiple, trade_simple};
use aggregates::aggregates;
use tank::Connection;

pub async fn execute_tests<C: Connection>(connection: &mut C) {
    trade_simple(connection).await;
    trade_multiple(connection).await;
    aggregates(connection).await;
}
