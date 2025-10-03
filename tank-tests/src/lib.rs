#![feature(assert_matches)]
#![feature(box_patterns)]
mod aggregates;
mod books;
mod complex;
mod documentation;
mod insane;
mod limits;
mod multiple;
mod readme;
mod simple;
mod trade;
mod transaction1;
mod user;

use crate::{
    books::books,
    complex::complex,
    documentation::documentation,
    insane::insane,
    limits::limits,
    multiple::multiple,
    simple::simple,
    trade::{trade_multiple, trade_simple},
    transaction1::transaction1,
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
        .format_file(true)
        .format_line_number(true)
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
    limits(&mut connection).await;
    multiple(&mut connection).await;
    #[cfg(not(feature = "disable-transactions"))]
    transaction1(&mut connection).await;
    drop(readme(&mut connection).await);
    documentation(&mut connection).await;
}
