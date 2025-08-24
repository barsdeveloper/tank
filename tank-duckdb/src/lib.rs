mod cbox;
mod connection;
mod driver;
mod extract_value;
mod query;
mod sql_writer;

pub use connection::*;
pub use driver::*;
use libduckdb_sys::{duckdb_hugeint, duckdb_uhugeint};
pub use query::*;
pub use sql_writer::*;

pub(crate) fn u128_to_duckdb_uhugeint(v: u128) -> duckdb_uhugeint {
    duckdb_uhugeint {
        lower: v as u64,
        upper: (v >> 64) as u64,
    }
}

pub(crate) fn i128_to_duckdb_hugeint(v: i128) -> duckdb_hugeint {
    duckdb_hugeint {
        lower: v as u64,
        upper: (v >> 64) as i64,
    }
}

pub(crate) fn duckdb_uhugeint_to_u128(v: &duckdb_uhugeint) -> u128 {
    (v.upper as u128) << 64 | v.lower as u128
}

pub(crate) fn duckdb_hugeint_to_i128(v: &duckdb_hugeint) -> i128 {
    (v.upper as i128) << 64 | v.lower as i128
}
