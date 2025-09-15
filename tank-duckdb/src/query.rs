use crate::{
    cbox::CBox, date_to_duckdb_date, decimal_to_duckdb_decimal, error_message_from_ptr,
    i128_to_duckdb_hugeint, interval_to_duckdb_interval, offsetdatetime_to_duckdb_timestamp,
    primitive_date_time_to_duckdb_timestamp, time_to_duckdb_time, u128_to_duckdb_uhugeint,
};
use libduckdb_sys::*;
use std::{
    ffi::c_void,
    fmt::{self, Display},
    sync::Arc,
};
use tank_core::{AsValue, Error, Prepared, Result, Value};

pub struct DuckDBPrepared {
    pub(crate) prepared: Arc<CBox<duckdb_prepared_statement>>,
    pub(crate) index: u64,
}
impl DuckDBPrepared {
    pub(crate) fn new(prepared: CBox<duckdb_prepared_statement>) -> Self {
        unsafe {
            duckdb_clear_bindings(*prepared);
        }
        Self {
            prepared: Arc::new(prepared),
            index: 1,
        }
    }
}

impl Display for DuckDBPrepared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.prepared)
    }
}

impl Prepared for DuckDBPrepared {
    fn bind<V: AsValue>(&mut self, v: V) -> Result<&mut Self> {
        unsafe {
            let prepared = **self.prepared;
            let value = v.as_value();
            let state = match value {
                Value::Null
                | Value::Boolean(None, ..)
                | Value::Int8(None, ..)
                | Value::Int16(None, ..)
                | Value::Int32(None, ..)
                | Value::Int64(None, ..)
                | Value::Int128(None, ..)
                | Value::UInt8(None, ..)
                | Value::UInt16(None, ..)
                | Value::UInt32(None, ..)
                | Value::UInt64(None, ..)
                | Value::UInt128(None, ..)
                | Value::Float32(None, ..)
                | Value::Float64(None, ..)
                | Value::Decimal(None, ..)
                | Value::Char(None, ..)
                | Value::Varchar(None, ..)
                | Value::Blob(None, ..)
                | Value::Date(None, ..)
                | Value::Time(None, ..)
                | Value::Timestamp(None, ..)
                | Value::TimestampWithTimezone(None, ..)
                | Value::Interval(None, ..)
                | Value::Uuid(None, ..)
                | Value::Array(None, ..)
                | Value::List(None, ..)
                | Value::Map(None, ..)
                | Value::Struct(None, ..) => duckdb_bind_null(prepared, self.index),
                Value::Boolean(Some(v)) => duckdb_bind_boolean(prepared, self.index, v),
                Value::Int8(Some(v)) => duckdb_bind_int8(prepared, self.index, v),
                Value::Int16(Some(v)) => duckdb_bind_int16(prepared, self.index, v),
                Value::Int32(Some(v)) => duckdb_bind_int32(prepared, self.index, v),
                Value::Int64(Some(v)) => duckdb_bind_int64(prepared, self.index, v),
                Value::Int128(Some(v)) => {
                    duckdb_bind_hugeint(prepared, self.index, i128_to_duckdb_hugeint(v))
                }
                Value::UInt8(Some(v)) => duckdb_bind_uint8(prepared, self.index, v),
                Value::UInt16(Some(v)) => duckdb_bind_uint16(prepared, self.index, v),
                Value::UInt32(Some(v)) => duckdb_bind_uint32(prepared, self.index, v),
                Value::UInt64(Some(v)) => duckdb_bind_uint64(prepared, self.index, v),
                Value::UInt128(Some(v)) => {
                    duckdb_bind_uhugeint(prepared, self.index, u128_to_duckdb_uhugeint(v))
                }
                Value::Float32(Some(v)) => duckdb_bind_float(prepared, self.index, v),
                Value::Float64(Some(v)) => duckdb_bind_double(prepared, self.index, v),
                Value::Decimal(Some(v), w, s) => {
                    duckdb_bind_decimal(prepared, self.index, decimal_to_duckdb_decimal(&v, w, s))
                }
                Value::Char(Some(v)) => {
                    let v = v.to_string();
                    let status = duckdb_bind_varchar_length(
                        prepared,
                        self.index,
                        v.as_ptr() as *const i8,
                        1,
                    );
                    status
                }
                Value::Varchar(Some(v)) => {
                    let status = duckdb_bind_varchar_length(
                        prepared,
                        self.index,
                        v.as_ptr() as *const i8,
                        v.len() as u64,
                    );
                    status
                }
                Value::Blob(Some(v)) => {
                    let status = duckdb_bind_blob(
                        prepared,
                        self.index,
                        v.as_ptr() as *const c_void,
                        v.len() as u64,
                    );
                    status
                }
                Value::Date(Some(v)) => {
                    duckdb_bind_date(prepared, self.index, date_to_duckdb_date(&v))
                }
                Value::Time(Some(v)) => {
                    duckdb_bind_time(prepared, self.index, time_to_duckdb_time(&v))
                }
                Value::Timestamp(Some(v)) => duckdb_bind_timestamp(
                    prepared,
                    self.index,
                    primitive_date_time_to_duckdb_timestamp(&v),
                ),
                Value::TimestampWithTimezone(Some(v)) => duckdb_bind_timestamp_tz(
                    prepared,
                    self.index,
                    offsetdatetime_to_duckdb_timestamp(&v),
                ),
                Value::Interval(Some(v)) => {
                    duckdb_bind_interval(prepared, self.index, interval_to_duckdb_interval(&v))
                }
                Value::Uuid(Some(_v)) => todo!(),
                _ => {
                    let error =
                        Error::msg(format!("Cannot use a {:?} as a query parameter", value));
                    log::error!("{}", error);
                    return Err(error);
                }
            };
            if state != duckdb_state_DuckDBSuccess {
                let error =
                    Error::msg(error_message_from_ptr(&duckdb_prepare_error(prepared)).to_string())
                        .context(format!("While trying to bind the parameter {}", self.index));
                log::error!("{}", error);
                return Err(error);
            }
            self.index += 1;
            Ok(self)
        }
    }
}
