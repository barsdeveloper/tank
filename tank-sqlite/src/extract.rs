use libsqlite3_sys::*;
use std::ffi::{CStr, c_int};
use tank_core::{Error, Result, Value};

pub(crate) fn extract_value(statement: *mut sqlite3_stmt, index: c_int) -> Result<Value> {
    unsafe {
        let column_type = sqlite3_column_type(statement, index);
        Ok(match column_type {
            SQLITE_NULL => Value::Null,
            SQLITE_INTEGER => sqlite3_column_int64(statement, index).into(),
            SQLITE_FLOAT => sqlite3_column_double(statement, index).into(),
            SQLITE_BLOB => {
                let ptr = sqlite3_column_blob(statement, index) as *const u8;
                let len = sqlite3_column_bytes(statement, index) as usize;
                Value::Blob(Some((0..len).map(|i| *ptr.add(i)).collect()))
            }
            SQLITE_TEXT => {
                let ptr = sqlite3_column_text(statement, index);
                let len = sqlite3_column_bytes(statement, index) as usize;
                String::from_utf8_unchecked((0..len).map(|i| *ptr.add(i)).collect()).into()
            }
            _ => {
                return Err(Error::msg(format!(
                    "Unexpected column type {}",
                    column_type
                )));
            }
        })
    }
}

pub(crate) fn extract_name(statement: *mut sqlite3_stmt, index: c_int) -> Result<String> {
    unsafe {
        Ok(CStr::from_ptr(sqlite3_column_name(statement, index))
            .to_str()?
            .into())
    }
}
