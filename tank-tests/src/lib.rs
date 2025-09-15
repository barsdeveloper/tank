#![feature(assert_matches)]
#![feature(box_patterns)]
mod aggregates;
mod books;
mod complex;
mod insane;
mod readme;
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
use log::LevelFilter;
use readme::readme;
use tank::Connection;

pub fn init_logs() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::Warn)
        .try_init();
}

pub async fn execute_tests<C: Connection>(mut connection: C) {
    simple(&mut connection).await;
    trade_simple(&mut connection).await;
    trade_multiple(&mut connection).await;
    users(&mut connection).await;
    aggregates(&mut connection).await;
    books(&mut connection).await;
    complex(&mut connection).await;
    insane(&mut connection).await;
    let _ = readme(&mut connection).await;

    // let mut connection = connection.as_cached_connection();
    // simple(&mut connection).await;
    // trade_simple(&mut connection).await;
    // trade_multiple(&mut connection).await;
    // users(&mut connection).await;
    // aggregates(&mut connection).await;
    // books(&mut connection).await;
    // complex(&mut connection).await;
    // insane(&mut connection).await;
}
