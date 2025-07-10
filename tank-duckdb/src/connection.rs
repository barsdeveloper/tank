use crate::{DuckDBPrepared, cbox::CBox, driver::DuckDBDriver, extract_value::extract_value};
use flume::Sender;
use futures::Stream;
use libduckdb_sys::*;
use std::{
    collections::BTreeMap,
    ffi::{CStr, CString, c_char, c_void},
    fmt::{self, Debug, Formatter},
    mem, ptr,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicPtr, Ordering},
    },
};
use tank_core::{
    Connection, Context, Error, Executor, Query, QueryResult, Result, RowLabeled, RowsAffected,
};
use tokio::{sync::RwLock, task::spawn_blocking};
use urlencoding::decode;

pub struct DuckDBConnection {
    pub(crate) connection: CBox<duckdb_connection>,
    pub(crate) transaction: bool,
    prepared_cache: RwLock<BTreeMap<String, DuckDBPrepared>>,
}

impl DuckDBConnection {
    pub(crate) fn database_cache() -> &'static AtomicPtr<_duckdb_instance_cache> {
        static DATABASE_CACHE: LazyLock<CBox<AtomicPtr<_duckdb_instance_cache>>> =
            LazyLock::new(|| {
                CBox::new(
                    AtomicPtr::new(unsafe { duckdb_create_instance_cache() }),
                    |ptr| unsafe {
                        duckdb_destroy_instance_cache(&mut ptr.load(Ordering::Relaxed))
                    },
                )
            });
        &**DATABASE_CACHE
    }

    pub(crate) fn run_unprepared(
        connection: duckdb_connection,
        query: &str,
        tx: Sender<Result<QueryResult>>,
    ) {
        unsafe {
            let result: duckdb_result = mem::zeroed();
            let mut result = CBox::new(result, |mut r| duckdb_destroy_result(&mut r));
            let rc = duckdb_query(connection, as_c_string(query).as_ptr(), &mut *result);
            if rc != duckdb_state_DuckDBSuccess {
                let message = CStr::from_ptr(duckdb_result_error(&mut *result))
                    .to_str()
                    .expect("Error message from prepare is expected to be a valid C string");
                let _ = tx.send(Err(Error::msg(format!(
                    "Error while executing the unprepared query: {}",
                    message
                ))));
                return;
            }
            Self::extract_result(result, tx);
        }
    }

    pub(crate) fn run_prepared(query: DuckDBPrepared, tx: Sender<Result<QueryResult>>) {
        unsafe {
            let result: duckdb_result = mem::zeroed();
            let mut result = CBox::new(result, |mut r| duckdb_destroy_result(&mut r));
            let rc = duckdb_execute_prepared_streaming(**query.prepared, &mut *result);
            if rc != duckdb_state_DuckDBSuccess {
                let message = CStr::from_ptr(duckdb_result_error(&mut *result))
                    .to_str()
                    .expect("Error message from prepare is expected to be a valid C string");
                let _ = tx.send(Err(Error::msg(format!(
                    "Error while executing the prepared query: {}",
                    message
                ))));
                return;
            }
            let statement_type = duckdb_result_statement_type(*result);
            #[allow(non_upper_case_globals)]
            if matches!(
                statement_type,
                duckdb_statement_type_DUCKDB_STATEMENT_TYPE_INSERT
                    | duckdb_statement_type_DUCKDB_STATEMENT_TYPE_UPDATE
                    | duckdb_statement_type_DUCKDB_STATEMENT_TYPE_DELETE
            ) {
                let rows_affected = duckdb_rows_changed(&mut *result);
                let _ = tx.send(Ok(QueryResult::Affected(RowsAffected {
                    rows_affected,
                    ..Default::default()
                })));
                return;
            }
            Self::extract_result(result, tx);
        }
    }

    pub(crate) fn extract_result(mut result: CBox<duckdb_result>, tx: Sender<Result<QueryResult>>) {
        unsafe {
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
                        let logical_type =
                            CBox::new(duckdb_vector_get_column_type(vector), |mut l| {
                                duckdb_destroy_logical_type(&mut l)
                            });
                        let type_id = duckdb_get_type_id(*logical_type);
                        let data = duckdb_vector_get_data(vector);
                        let validity = duckdb_vector_get_validity(vector);
                        let name = CStr::from_ptr(duckdb_column_name(&mut *result, i))
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
                        let info = &info[col];
                        Ok(extract_value(
                            info.0,
                            row as usize,
                            *info.1,
                            info.2,
                            info.3,
                            info.4,
                        )?)
                    });
                    let row =
                        RowLabeled::new(names.clone(), columns.collect::<Result<_>>().unwrap());
                    let _ = tx.send(Ok(QueryResult::RowLabeled(row)));
                });
            }
        }
    }
}

impl Debug for DuckDBConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DuckDBConnection")
            .field("connection", &self.connection)
            .field("transaction", &self.transaction)
            .finish()
    }
}

impl Executor for DuckDBConnection {
    type Driver = DuckDBDriver;

    fn driver(&self) -> &Self::Driver {
        &DuckDBDriver {}
    }

    async fn prepare(&mut self, query: String) -> Result<Query<DuckDBPrepared>> {
        if let Some(prepared) = self.prepared_cache.read().await.get(&query) {
            return Ok(prepared.clone().into());
        }
        log::debug!("Preparing query: `{}`", query);
        let connection = AtomicPtr::new(*self.connection);
        let source = query.clone();
        let prepared = spawn_blocking(move || unsafe {
            let mut prepared = CBox::new(ptr::null_mut(), |mut p| duckdb_destroy_prepare(&mut p));
            let source = match CString::new(source.as_bytes()) {
                Ok(query) => query,
                Err(e) => {
                    return Err(
                        Error::new(e).context("Could not create a CString from the query String")
                    );
                }
            };
            let rc = duckdb_prepare(
                connection.load(Ordering::Relaxed),
                source.as_ptr(),
                &mut *prepared,
            );
            if rc != duckdb_state_DuckDBSuccess {
                return Err(Error::msg(
                    CStr::from_ptr(duckdb_prepare_error(*prepared))
                        .to_str()
                        .expect("Errore message from prepare is expected to be a valid C string"),
                ));
            }
            Ok(prepared)
        })
        .await?;
        let prepared = DuckDBPrepared::new(prepared?);
        self.prepared_cache
            .write()
            .await
            .insert(query, prepared.clone());
        Ok(prepared.into())
    }

    fn run(&mut self, query: Query<DuckDBPrepared>) -> impl Stream<Item = Result<QueryResult>> {
        let (tx, rx) = flume::unbounded::<Result<QueryResult>>();
        let connection = AtomicPtr::new(*self.connection);
        spawn_blocking(move || match query {
            Query::Raw(query) => {
                Self::run_unprepared(connection.load(Ordering::Relaxed), &query, tx)
            }
            Query::Prepared(query) => Self::run_prepared(query, tx),
        });
        rx.into_stream()
    }
}

fn as_c_string<S: Into<Vec<u8>>>(str: S) -> CString {
    CString::new(str.into()).expect("Expected a valid C string")
}

impl Connection for DuckDBConnection {
    const PREFIX: &'static str = "duckdb";

    #[allow(refining_impl_trait)]
    async fn connect(url: &str) -> Result<DuckDBConnection> {
        let prefix = format!("{}://", Self::PREFIX);
        if !url.starts_with(&prefix) {
            return Err(Error::msg(format!(
                "Expected duckdb connection url to start with `{}`",
                &prefix,
            )));
        }
        let mut parts = url.trim_start_matches(&prefix).splitn(2, '?');
        let path = parts.next().ok_or(Error::msg(format!(
            "Expected database file path or `:memory:` in the connection URL: `{}`",
            url,
        )))?;
        let params = parts.next().unwrap_or_default();
        let context = || format!("Error while decoding connection URL: `{}`", url);
        let path = decode(path)
            .with_context(context)
            .and_then(|v| CString::new(&*v).with_context(context))?;
        let mut config: CBox<duckdb_config> = CBox::new(ptr::null_mut(), |mut p| unsafe {
            duckdb_destroy_config(&mut p)
        });
        unsafe {
            let rc = duckdb_create_config(&mut *config);
            if rc != duckdb_state_DuckDBSuccess {
                return Err(Error::msg("Error while creating the duckdb_config object"));
            }
        };
        for (key, value) in url::form_urlencoded::parse(params.as_bytes()) {
            let rc = unsafe {
                match &*key {
                    "mode" => duckdb_set_config(
                        *config,
                        c"access_mode".as_ptr(),
                        match &*value {
                            "ro" => c"READ_ONLY",
                            "rw" => c"READ_WRITE",
                            _ => {
                                return Err(Error::msg("Unknown value {value:?} for `mode`, expected one of: `ro`, `rw`"));
                            }
                        }
                        .as_ptr(),
                    ),
                    _ => duckdb_set_config(
                        *config,
                        as_c_string(&*key).as_ptr(),
                        as_c_string(&*value).as_ptr(),
                    ),
                }
            };
            if rc != duckdb_state_DuckDBSuccess {
                return Err(Error::msg(format!(
                    "Error while setting config `{}={}`",
                    key, value
                )));
            }
        }
        let mut database: duckdb_database = ptr::null_mut();
        let mut connection: CBox<duckdb_connection>;
        let mut error: CBox<*mut c_char> = CBox::new(ptr::null_mut(), |p| unsafe {
            duckdb_free(p as *mut c_void)
        });
        let db_cache = Self::database_cache().load(Ordering::Relaxed);
        unsafe {
            let rc = duckdb_get_or_create_from_cache(
                db_cache,
                path.as_ptr(),
                &mut database,
                *config,
                &mut *error,
            );
            if rc != duckdb_state_DuckDBSuccess {
                let error = CStr::from_ptr(error.ptr)
                    .to_str()
                    .context("While reading the error from `duckdb_get_or_create_from_cache`")?
                    .to_owned();
                return Err(Error::msg(error));
            };
            connection = CBox::new(ptr::null_mut(), |mut p| duckdb_disconnect(&mut p));
            let rc = duckdb_connect(database, &mut *connection);
            if rc != duckdb_state_DuckDBSuccess {
                return Err(Error::msg("Could not connect to the database"));
            };
        };
        Ok(DuckDBConnection {
            connection,
            transaction: false,
            prepared_cache: Default::default(),
        })
    }
}
