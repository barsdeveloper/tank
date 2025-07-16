#![feature(assert_matches)]
#![feature(box_patterns)]
mod aggregate_functions;
mod trade;

use crate::trade::{trade_multiple, trade_simple};
use aggregate_functions::aggregate_functions;
use tank::Connection;

pub async fn execute_tests<C: Connection>(connection: &mut C) {
    trade_simple(connection).await;
    trade_multiple(connection).await;
    aggregate_functions(connection).await;
}
