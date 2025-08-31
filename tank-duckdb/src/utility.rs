use std::{ffi::CStr, ptr};

use libduckdb_sys::{duckdb_hugeint, duckdb_uhugeint};

// pub(crate) fn u128_to_duckdb_uhugeint(v: u128) -> duckdb_uhugeint {
//     duckdb_uhugeint {
//         lower: v as u64,
//         upper: (v >> 64) as u64,
//     }
// }

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

pub(crate) fn extract_duckdb_error_from_ptr(ptr: &'_ *const i8) -> &'_ str {
    unsafe {
        if *ptr != ptr::null() {
            CStr::from_ptr(*ptr)
                .to_str()
                .unwrap_or("Unknown error (the error message was not a valid C string)")
        } else {
            "Unknown error (could not extract it from DuckDB)"
        }
    }
}
