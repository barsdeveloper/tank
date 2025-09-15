use crate::{CBox, error_message_from_ptr};
use libsqlite3_sys::{
    SQLITE_OK, SQLITE_TRANSIENT, sqlite3_bind_blob, sqlite3_bind_double, sqlite3_bind_int,
    sqlite3_bind_int64, sqlite3_bind_null, sqlite3_bind_text, sqlite3_clear_bindings,
    sqlite3_db_handle, sqlite3_errmsg, sqlite3_int64, sqlite3_stmt,
};
use rust_decimal::prelude::ToPrimitive;
use std::{
    ffi::c_int,
    fmt::{self, Display},
    os::raw::{c_char, c_void},
};
use tank_core::{AsValue, Error, Prepared, Result, Value};

pub struct SqlitePrepared {
    pub(crate) statement: CBox<*mut sqlite3_stmt>,
    pub(crate) bind_index: c_int,
}

impl SqlitePrepared {
    pub(crate) fn new(prepared: CBox<*mut sqlite3_stmt>) -> Self {
        unsafe {
            sqlite3_clear_bindings(*prepared);
        }
        Self {
            statement: prepared,
            bind_index: 0,
        }
    }
}

impl Prepared for SqlitePrepared {
    fn bind<V: AsValue>(&mut self, v: V) -> Result<&mut Self> {
        unsafe {
            let value = v.as_value();
            let rc = match value {
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
                | Value::Struct(None, ..) => sqlite3_bind_null(*self.statement, self.bind_index),
                Value::Boolean(Some(v)) => {
                    sqlite3_bind_int(*self.statement, self.bind_index, v as c_int)
                }
                Value::Int8(Some(v)) => {
                    sqlite3_bind_int(*self.statement, self.bind_index, v as c_int)
                }
                Value::Int16(Some(v)) => {
                    sqlite3_bind_int(*self.statement, self.bind_index, v as c_int)
                }
                Value::Int32(Some(v)) => {
                    sqlite3_bind_int(*self.statement, self.bind_index, v as c_int)
                }
                Value::Int64(Some(v)) => sqlite3_bind_int64(*self.statement, self.bind_index, v),
                Value::Int128(Some(v)) => {
                    if v as sqlite3_int64 as i128 != v {
                        return Err(Error::msg(
                            "Cannot bind i128 value `{}` into sqlite integer because it's out of bounds",
                        ));
                    }
                    sqlite3_bind_int64(*self.statement, self.bind_index, v as sqlite3_int64)
                }
                Value::UInt8(Some(v)) => {
                    sqlite3_bind_int(*self.statement, self.bind_index, v as c_int)
                }
                Value::UInt16(Some(v)) => {
                    sqlite3_bind_int(*self.statement, self.bind_index, v as c_int)
                }
                Value::UInt32(Some(v)) => {
                    sqlite3_bind_int(*self.statement, self.bind_index, v as c_int)
                }
                Value::UInt64(Some(v)) => {
                    if v as sqlite3_int64 as u64 != v {
                        return Err(Error::msg(
                            "Cannot bind i128 value `{}` into sqlite integer because it's out of bounds",
                        ));
                    }
                    sqlite3_bind_int64(*self.statement, self.bind_index, v as sqlite3_int64)
                }
                Value::UInt128(Some(v)) => {
                    if v as sqlite3_int64 as u128 != v {
                        return Err(Error::msg(
                            "Cannot bind i128 value `{}` into sqlite integer because it's out of bounds",
                        ));
                    }
                    sqlite3_bind_int64(*self.statement, self.bind_index, v as sqlite3_int64)
                }
                Value::Float32(Some(v)) => {
                    sqlite3_bind_double(*self.statement, self.bind_index, v as f64)
                }
                Value::Float64(Some(v)) => sqlite3_bind_double(*self.statement, self.bind_index, v),
                Value::Decimal(Some(v), ..) => sqlite3_bind_double(
                    *self.statement,
                    self.bind_index,
                    v.to_f64().ok_or_else(|| {
                        Error::msg(format!("Cannot convert the Decimal value `{}` to f64", v))
                    })?,
                ),
                Value::Char(Some(v)) => {
                    let v = v.to_string();
                    sqlite3_bind_text(
                        *self.statement,
                        self.bind_index,
                        v.as_ptr() as *const c_char,
                        v.len() as c_int,
                        SQLITE_TRANSIENT(),
                    )
                }
                Value::Varchar(Some(v)) => sqlite3_bind_text(
                    *self.statement,
                    self.bind_index,
                    v.as_ptr() as *const c_char,
                    v.len() as c_int,
                    SQLITE_TRANSIENT(),
                ),
                Value::Blob(Some(v)) => sqlite3_bind_blob(
                    *self.statement,
                    self.bind_index,
                    v.as_ptr() as *const c_void,
                    v.len() as c_int,
                    SQLITE_TRANSIENT(),
                ),
                Value::Date(Some(v)) => {
                    let v = v.to_string();
                    sqlite3_bind_text(
                        *self.statement,
                        self.bind_index,
                        v.as_ptr() as *const c_char,
                        v.len() as c_int,
                        SQLITE_TRANSIENT(),
                    )
                }
                Value::Time(Some(v)) => {
                    let v = v.to_string();
                    sqlite3_bind_text(
                        *self.statement,
                        self.bind_index,
                        v.as_ptr() as *const c_char,
                        v.len() as c_int,
                        SQLITE_TRANSIENT(),
                    )
                }
                Value::Timestamp(Some(v)) => {
                    let v = v.to_string();
                    sqlite3_bind_text(
                        *self.statement,
                        self.bind_index,
                        v.as_ptr() as *const c_char,
                        v.len() as c_int,
                        SQLITE_TRANSIENT(),
                    )
                }
                Value::TimestampWithTimezone(Some(v)) => {
                    let v = v.to_string();
                    sqlite3_bind_text(
                        *self.statement,
                        self.bind_index,
                        v.as_ptr() as *const c_char,
                        v.len() as c_int,
                        SQLITE_TRANSIENT(),
                    )
                }
                Value::Uuid(Some(v)) => {
                    let v = v.to_string();
                    sqlite3_bind_text(
                        *self.statement,
                        self.bind_index,
                        v.as_ptr() as *const c_char,
                        v.len() as c_int,
                        SQLITE_TRANSIENT(),
                    )
                }
                _ => {
                    let error =
                        Error::msg(format!("Cannot use a {:?} as a query parameter", value));
                    log::error!("{}", error);
                    return Err(error);
                }
            };
            if rc != SQLITE_OK {
                let db = sqlite3_db_handle(*self.statement);
                let error = Error::msg(error_message_from_ptr(&sqlite3_errmsg(db)).to_string());
                log::error!("{}", error);
                return Err(error);
            }
            Ok(self)
        }
    }
}

impl Display for SqlitePrepared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", *self.statement)
    }
}
