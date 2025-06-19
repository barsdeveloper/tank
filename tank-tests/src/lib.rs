mod resource;
mod trade_test;

use crate::trade_test::trade_test_setup;
pub use resource::*;
use tank::Connection;

pub async fn execute_tests<C: Connection>(connection: &mut C) {
    trade_test_setup(connection).await;
}
