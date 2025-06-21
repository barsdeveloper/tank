mod aggregate;
mod resource;
mod trade;

use crate::trade::{trade_multiple, trade_simple};
pub use resource::*;
use tank::Connection;

pub async fn execute_tests<C: Connection>(connection: &mut C) {
    trade_simple(connection).await;
    trade_multiple(connection).await;
}
