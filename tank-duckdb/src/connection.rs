use crate::{cbox::CBox, driver::DuckDBDriver, extract_value::extract_value, DuckDBPrepared};
use anyhow::{anyhow, Error, Result};
use futures::{stream::BoxStream, StreamExt};
use libduckdb_sys::*;
use std::{
    ffi::{CStr, CString},
    mem,
    ops::DerefMut,
    ptr,
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc,
    },
};
use tank_core::{Connection, Count, Executor, Query, QueryResult, Row};
use tokio::task::spawn_blocking;

#[derive(Debug)]
pub struct DuckDBConnection {
    pub(crate) connection: CBox<duckdb_connection>,
    pub(crate) transaction: bool,
}

impl Executor for DuckDBConnection {
    type Driver = DuckDBDriver;

    fn driver(&self) -> &Self::Driver {
        &DuckDBDriver {}
    }

    fn run<'a>(&mut self, mut query: Query<DuckDBPrepared>) -> BoxStream<'a, Result<QueryResult>> {
        let (tx, rx) = flume::unbounded::<Result<QueryResult>>();
        let connection = AtomicPtr::new(*self.connection);
        spawn_blocking(move || unsafe {
            if let Query::Raw(v) = query {
                query = Query::Prepared({
                    let mut prepared: duckdb_prepared_statement = ptr::null_mut();
                    let query = match CString::new(v.as_bytes()) {
                        Ok(query) => query,
                        Err(e) => {
                            let _ = tx.send(Err(Error::new(e)
                                .context("Could not create a CString from the query String")));
                            return;
                        }
                    };
                    let rc = duckdb_prepare(
                        connection.load(Ordering::Relaxed),
                        query.as_ptr(),
                        &mut prepared,
                    );
                    if rc != duckdb_state_DuckDBSuccess {
                        let message = CStr::from_ptr(duckdb_prepare_error(prepared))
                            .to_str()
                            .unwrap()
                            .to_string();
                        let _ = tx.send(Err(anyhow!(message)));
                    }
                    DuckDBPrepared::new(prepared)
                });
            }
            let Query::Prepared(query) = query else {
                unreachable!("The query is prepared by this point");
            };
            let mut result: duckdb_result = mem::zeroed();
            let rc = duckdb_execute_prepared_streaming(**query.prepared, &mut result);
            let mut result = CBox::new(result, |mut r| duckdb_destroy_result(&mut r));
            if rc != duckdb_state_DuckDBSuccess {
                let _ = tx.send(Err(anyhow!("Error while executing the query")));
            }
            let statement_type = duckdb_result_statement_type(*result);
            if matches!(
                statement_type,
                duckdb_statement_type_DUCKDB_STATEMENT_TYPE_INSERT
                    | duckdb_statement_type_DUCKDB_STATEMENT_TYPE_DELETE
            ) {
                let rows_affected = duckdb_rows_changed(&mut *result);
                let _ = tx.send(Ok(QueryResult::Count(Count {
                    rows_affected,
                    ..Default::default()
                })));
                return;
            }
            // duckdb_execute_prepared_streaming can also produce non streaming result, must check separately
            let is_streaming = duckdb_result_is_streaming(*result);
            loop {
                let chunk = CBox::new(
                    if is_streaming {
                        duckdb_stream_fetch_chunk(*result)
                    } else {
                        duckdb_fetch_chunk(*result)
                    },
                    |mut v| duckdb_destroy_data_chunk(&mut v),
                );
                if chunk.is_null() {
                    return;
                }
                let rows = duckdb_data_chunk_get_size(*chunk);
                let cols = duckdb_data_chunk_get_column_count(*chunk);
                let info = (0..cols)
                    .map(|i| {
                        let vector = duckdb_data_chunk_get_vector(*chunk, i);
                        let logical_type = duckdb_vector_get_column_type(vector);
                        let type_id = duckdb_get_type_id(logical_type);
                        let data = duckdb_vector_get_data(vector);
                        let validity = duckdb_vector_get_validity(vector);
                        let name = CStr::from_ptr(duckdb_column_name(result.deref_mut(), i))
                            .to_str()
                            .unwrap();
                        (vector, logical_type, type_id, data, validity, name)
                    })
                    .collect::<Box<[_]>>();
                let names = info
                    .iter()
                    .map(|v| v.5.to_string())
                    .collect::<Arc<[String]>>();
                (0..rows).for_each(|row| {
                    let columns = (0..cols).map(|col| {
                        let col = col as usize;
                        let info = info[col];
                        Ok(extract_value(
                            info.0,
                            row as usize,
                            info.1,
                            info.2,
                            info.3,
                            info.4,
                        )?)
                    });
                    let row = Row::new(names.clone(), columns.collect::<Result<_>>().unwrap());
                    let _ = tx.send(Ok(QueryResult::Row(row)));
                });
            }
        });
        rx.into_stream().boxed()
    }
}

impl Connection for DuckDBConnection {}
