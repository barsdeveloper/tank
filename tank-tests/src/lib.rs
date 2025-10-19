mod aggregates;
mod books;
mod complex;
mod documentation;
mod full1;
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
use std::env;
use tank::Connection;

#[cfg(not(feature = "disable-intervals"))]
mod interval;
#[cfg(not(feature = "disable-intervals"))]
use interval::interval;

#[cfg(not(feature = "disable-arrays"))]
mod array;
#[cfg(not(feature = "disable-arrays"))]
use array::array;

pub fn init_logs() {
    let mut logger = env_logger::builder();
    logger
        .is_test(true)
        .format_file(true)
        .format_line_number(true);
    if env::var("RUST_LOG").is_err() {
        logger.filter_level(LevelFilter::Warn);
    }
    let _ = logger.try_init();
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
    #[cfg(not(feature = "disable-intervals"))]
    interval(&mut connection).await;
    #[cfg(not(feature = "disable-arrays"))]
    array(&mut connection).await;
    drop(readme(&mut connection).await);
    documentation(&mut connection).await;
}

#[macro_export]
macro_rules! silent_logs {
    ($($code:tt)+) => {{
        let level = log::max_level();
        log::set_max_level(log::LevelFilter::Off);
        $($code)+
        log::set_max_level(level);
    }};
}
