use crate::{cbox::CBox, i128_to_duckdb_hugeint};
use libduckdb_sys::*;
use std::{
    ffi::{CStr, c_void},
    ptr,
    sync::Arc,
};
use tank_core::{AsValue, Error, Prepared, Result, Value};
use time::{Time, macros::date};

#[derive(Clone)]
pub struct DuckDBPrepared {
    pub(crate) prepared: Arc<CBox<duckdb_prepared_statement>>,
    pub(crate) index: u64,
}
impl DuckDBPrepared {
    pub(crate) fn new(prepared: CBox<duckdb_prepared_statement>) -> Self {
        Self {
            prepared: Arc::new(prepared),
            index: 0,
        }
    }
}

impl Default for DuckDBPrepared {
    fn default() -> Self {
        Self {
            prepared: Arc::new(CBox::new(ptr::null_mut(), |_| {})),
            index: 0,
        }
    }
}

impl Prepared for DuckDBPrepared {
    // fn bind<V: AsValue>(mut self, v: V) -> Result<Self> {
    //     unsafe {
    //         let prepared = **self.prepared;
    //         let state = match v.as_value() {
    //             Value::Null
    //             | Value::Boolean(None)
    //             | Value::Int8(None, ..)
    //             | Value::Int16(None, ..)
    //             | Value::Int32(None, ..)
    //             | Value::Int64(None, ..)
    //             | Value::Int128(None, ..)
    //             | Value::UInt8(None, ..)
    //             | Value::UInt16(None, ..)
    //             | Value::UInt32(None, ..)
    //             | Value::UInt64(None, ..)
    //             | Value::UInt128(None, ..)
    //             | Value::Float32(None, ..)
    //             | Value::Float64(None, ..)
    //             | Value::Decimal(None, ..)
    //             | Value::Char(None, ..)
    //             | Value::Varchar(None, ..)
    //             | Value::Blob(None, ..)
    //             | Value::Date(None, ..)
    //             | Value::Time(None, ..)
    //             | Value::Timestamp(None, ..)
    //             | Value::TimestampWithTimezone(None, ..)
    //             | Value::Interval(None, ..)
    //             | Value::Uuid(None, ..)
    //             | Value::Array(None, ..)
    //             | Value::List(None, ..)
    //             | Value::Map(None, ..)
    //             | Value::Struct(None, ..) => duckdb_bind_null(prepared, self.index),
    //             Value::Boolean(Some(v)) => duckdb_bind_boolean(prepared, self.index, v),
    //             Value::Int8(Some(v)) => duckdb_bind_int8(prepared, self.index, v),
    //             Value::Int16(Some(v)) => duckdb_bind_int16(prepared, self.index, v),
    //             Value::Int32(Some(v)) => duckdb_bind_int32(prepared, self.index, v),
    //             Value::Int64(Some(v)) => duckdb_bind_int64(prepared, self.index, v),
    //             Value::Int128(Some(v)) => duckdb_bind_hugeint(
    //                 prepared,
    //                 self.index,
    //                 duckdb_hugeint {
    //                     lower: v as u64,
    //                     upper: (v >> 64) as i64,
    //                 },
    //             ),
    //             Value::UInt8(Some(v)) => duckdb_bind_uint8(prepared, self.index, v),
    //             Value::UInt16(Some(v)) => duckdb_bind_uint16(prepared, self.index, v),
    //             Value::UInt32(Some(v)) => duckdb_bind_uint32(prepared, self.index, v),
    //             Value::UInt64(Some(v)) => duckdb_bind_uint64(prepared, self.index, v),
    //             Value::UInt128(Some(v)) => duckdb_bind_uhugeint(
    //                 prepared,
    //                 self.index,
    //                 duckdb_uhugeint {
    //                     lower: v as u64,
    //                     upper: (v >> 64) as u64,
    //                 },
    //             ),
    //             Value::Float32(Some(v)) => duckdb_bind_float(prepared, self.index, v),
    //             Value::Float64(Some(v)) => duckdb_bind_double(prepared, self.index, v),
    //             Value::Decimal(Some(v), w, s) => duckdb_bind_decimal(
    //                 prepared,
    //                 self.index,
    //                 duckdb_decimal {
    //                     width: w,
    //                     scale: s,
    //                     value: i128_to_duckdb_hugeint(v.mantissa()),
    //                 },
    //             ),
    //             Value::Char(Some(v)) => {
    //                 let v = v.to_string();
    //                 let status = duckdb_bind_varchar_length(
    //                     prepared,
    //                     self.index,
    //                     v.as_ptr() as *const i8,
    //                     1,
    //                 );
    //                 self.storage.push(StoredValue::Varchar(v));
    //                 status
    //             }
    //             Value::Varchar(Some(v)) => {
    //                 let status = duckdb_bind_varchar_length(
    //                     prepared,
    //                     self.index,
    //                     v.as_ptr() as *const i8,
    //                     v.len() as u64,
    //                 );
    //                 self.storage.push(StoredValue::Varchar(v));
    //                 status
    //             }
    //             Value::Blob(Some(v)) => {
    //                 let status = duckdb_bind_blob(
    //                     prepared,
    //                     self.index,
    //                     v.as_ptr() as *const c_void,
    //                     v.len() as u64,
    //                 );
    //                 self.storage.push(StoredValue::Blob(v));
    //                 status
    //             }
    //             Value::Date(Some(v)) => duckdb_bind_date(
    //                 prepared,
    //                 self.index,
    //                 duckdb_date {
    //                     days: (v - date!(1970 - 01 - 01)).whole_days() as i32,
    //                 },
    //             ),
    //             Value::Time(Some(v)) => duckdb_bind_time(
    //                 prepared,
    //                 self.index,
    //                 duckdb_time {
    //                     micros: (v - Time::MIDNIGHT).whole_microseconds() as i64,
    //                 },
    //             ),
    //             Value::Timestamp(Some(v)) => duckdb_bind_timestamp(
    //                 prepared,
    //                 self.index,
    //                 duckdb_timestamp {
    //                     micros: (v.assume_utc().unix_timestamp_nanos() / 1000) as i64,
    //                 },
    //             ),
    //             Value::TimestampWithTimezone(Some(v)) => duckdb_bind_timestamp_tz(
    //                 prepared,
    //                 self.index,
    //                 duckdb_timestamp {
    //                     micros: (v.to_utc().unix_timestamp_nanos() / 1000) as i64,
    //                 },
    //             ),
    //             Value::Interval(Some(v)) => duckdb_bind_interval(
    //                 prepared,
    //                 self.index,
    //                 duckdb_interval {
    //                     months: v.months as i32,
    //                     days: v.days as i32,
    //                     micros: (v.nanos / 1000) as i64,
    //                 },
    //             ),
    //             Value::Uuid(Some(_v)) => todo!(),
    //             Value::Array(Some(_v), ..) => {
    //                 unreachable!("Cannot use a array as a query parameter")
    //             }
    //             Value::List(Some(_v), ..) => {
    //                 unreachable!("Cannot use a list as a query parameter")
    //             }
    //             Value::Map(Some(_v), ..) => {
    //                 unreachable!("Cannot use a map as a query parameter")
    //             }
    //             Value::Struct(Some(_v), ..) => {
    //                 unreachable!("Cannot use a struct as a query parameter")
    //             }
    //         };
    //         self.index += 1;
    //         if state != duckdb_state_DuckDBSuccess {
    //             return Err(Error::msg(
    //                 CStr::from_ptr(duckdb_prepare_error(prepared))
    //                     .to_str()
    //                     .expect("Errore message from binding is expected to be a valid C string"),
    //             )
    //             .context(format!("While trying to bind the parameter {}", self.index)));
    //         }
    //         Ok(self)
    //     }
    // }
}

pub struct DuckDBPreparedPtr(pub(crate) duckdb_prepared_statement);
unsafe impl Send for DuckDBPreparedPtr {}

impl From<duckdb_prepared_statement> for DuckDBPreparedPtr {
    fn from(value: duckdb_prepared_statement) -> Self {
        Self(value)
    }
}
