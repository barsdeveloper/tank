use crate::{
    DuckDBPrepared, DuckDBTransaction, cbox::CBox, date_to_duckdb_date, decimal_to_duckdb_decimal,
    driver::DuckDBDriver, error_message_from_ptr, extract::extract_value, i128_to_duckdb_hugeint,
    interval_to_duckdb_interval, offsetdatetime_to_duckdb_timestamp,
    primitive_date_time_to_duckdb_timestamp, tank_value_to_duckdb_logical_type,
    tank_value_to_duckdb_value, time_to_duckdb_time, u128_to_duckdb_uhugeint,
};
use flume::Sender;
use libduckdb_sys::*;
use std::{
    borrow::Cow,
    ffi::{CStr, CString, c_char, c_void},
    fmt::{self, Debug, Formatter},
    mem, ptr,
    sync::{
        Arc, LazyLock,
        atomic::{AtomicPtr, Ordering},
    },
};
use tank_core::{
    Connection, Driver, Entity, Error, ErrorContext, Executor, Query, QueryResult, Result,
    RowLabeled, RowsAffected, Value, as_c_string, printable_query, send_error,
    stream::{Stream, TryStreamExt},
};
use tokio::task::spawn_blocking;
use url::form_urlencoded;
use urlencoding::decode;

pub struct DuckDBConnection {
    pub(crate) connection: CBox<duckdb_connection>,
    pub(crate) transaction: bool,
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

    pub(crate) fn run<F>(execute: F, tx: Sender<Result<QueryResult>>)
    where
        F: FnOnce(*mut duckdb_result) -> u32,
    {
        unsafe {
            let result: duckdb_result = mem::zeroed();
            let mut result = CBox::new(result, |mut r| duckdb_destroy_result(&mut r));
            let rc = execute(&mut *result);
            if rc != duckdb_state_DuckDBSuccess {
                send_error!(
                    tx,
                    Error::msg(
                        error_message_from_ptr(&duckdb_result_error(&mut *result)).to_string(),
                    )
                );
                return;
            }
            let statement_type = duckdb_result_statement_type(*result);
            #[allow(non_upper_case_globals)]
            if !matches!(
                statement_type,
                duckdb_statement_type_DUCKDB_STATEMENT_TYPE_SELECT
            ) {
                let rows_affected = duckdb_rows_changed(&mut *result);
                let _ = tx.send(Ok(QueryResult::Affected(RowsAffected {
                    rows_affected,
                    ..Default::default()
                })));
                return;
            }
            Self::extract_result(&mut *result, tx);
        }
    }

    pub(crate) fn run_unprepared(
        connection: duckdb_connection,
        sql: CString,
        tx: Sender<Result<QueryResult>>,
    ) {
        unsafe {
            let mut statements =
                CBox::new(ptr::null_mut(), |mut p| duckdb_destroy_extracted(&mut p));
            let count = duckdb_extract_statements(connection, sql.as_ptr(), &mut *statements);
            if count == 0 {
                let error = Error::msg(
                    error_message_from_ptr(&duckdb_extract_statements_error(*statements))
                        .to_string(),
                );
                send_error!(tx, error);
                return;
            }
            for i in 0..count {
                let mut statement =
                    CBox::new(ptr::null_mut(), |mut p| duckdb_destroy_prepare(&mut p));
                let rc =
                    duckdb_prepare_extracted_statement(connection, *statements, i, &mut *statement);
                if rc != duckdb_state_DuckDBSuccess {
                    let error = Error::msg(
                        error_message_from_ptr(&duckdb_prepare_error(*statement)).to_string(),
                    );
                    send_error!(tx, error);
                    return;
                }
                Self::run_prepared(statement.into(), tx.clone());
            }
        }
    }

    pub(crate) fn run_prepared(prepared: DuckDBPrepared, tx: Sender<Result<QueryResult>>) {
        let tx2 = tx.clone();
        Self::run(
            |result| unsafe {
                let rc = duckdb_execute_prepared_streaming(*prepared.statement, result);
                if rc != duckdb_state_DuckDBSuccess {
                    let error = Error::msg(
                        error_message_from_ptr(&duckdb_prepare_error(*prepared.statement))
                            .to_string(),
                    )
                    .context("While preparing a query");
                    send_error!(tx2, error);
                }
                rc
            },
            tx,
        );
        unsafe {
            duckdb_clear_bindings(*prepared.statement);
        }
    }

    pub(crate) fn extract_result(result: *mut duckdb_result, tx: Sender<Result<QueryResult>>) {
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
                for row in 0..rows {
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
                    if let Err(e) = tx.send(Ok(QueryResult::RowLabeled(row))) {
                        log::error!("{:#}", e);
                    }
                }
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

    async fn prepare(&mut self, query: String) -> Result<Query<DuckDBDriver>> {
        let connection = AtomicPtr::new(*self.connection);
        let source = query.clone();
        let context = format!(
            "While preparing the query {}",
            printable_query!(query.as_str())
        );
        let prepared = spawn_blocking(move || unsafe {
            let mut prepared = CBox::new(ptr::null_mut(), |mut p| duckdb_destroy_prepare(&mut p));
            let source = match CString::new(source.as_bytes()) {
                Ok(query) => query,
                Err(e) => {
                    let error =
                        Error::new(e).context("Could not create a CString from the query String");
                    log::error!("{:#}", error);
                    return Err(error);
                }
            };
            let rc = duckdb_prepare(
                connection.load(Ordering::Relaxed),
                source.as_ptr(),
                &mut *prepared,
            );
            if rc != duckdb_state_DuckDBSuccess {
                let error = Error::msg(
                    error_message_from_ptr(&duckdb_prepare_error(*prepared)).to_string(),
                )
                .context(context);
                log::error!("{:#}", error);
                return Err(error);
            }
            Ok(prepared)
        })
        .await?;
        let prepared = DuckDBPrepared::new(prepared?);
        Ok(prepared.into())
    }

    fn run(&mut self, query: Query<DuckDBDriver>) -> impl Stream<Item = Result<QueryResult>> {
        let (tx, rx) = flume::unbounded::<Result<QueryResult>>();
        let connection = AtomicPtr::new(*self.connection);
        let context = Arc::new(format!("While executing the query:\n{}", query));
        let stream = rx
            .into_stream()
            .map_err(move |e| e.context(context.clone()));
        spawn_blocking(move || match query {
            Query::Raw(query) => {
                let query = unsafe { CString::from_vec_unchecked(query.into_bytes()) };
                Self::run_unprepared(connection.load(Ordering::Relaxed), query, tx)
            }
            Query::Prepared(query) => Self::run_prepared(query, tx),
        });
        stream
    }

    async fn append<'a, E, It>(&mut self, rows: It) -> Result<RowsAffected>
    where
        E: Entity + 'a,
        It: IntoIterator<Item = &'a E> + Send,
    {
        let connection = AtomicPtr::new(*self.connection);
        let rows = rows.into_iter().map(Entity::row_full).collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(Default::default());
        }
        spawn_blocking(move || unsafe {
            let table_ref = E::table_ref();
            let mut appender = CBox::new(ptr::null_mut(), |mut p| {
                duckdb_appender_destroy(&mut p);
            });
            let connection = connection.load(Ordering::Relaxed);
            let rc = if let Some((catalog, schema)) = table_ref.schema.rsplit_once('.') {
                duckdb_appender_create_ext(
                    connection,
                    as_c_string(catalog).as_ptr(),
                    as_c_string(schema).as_ptr(),
                    as_c_string(table_ref.name).as_ptr(),
                    &mut *appender,
                )
            } else {
                duckdb_appender_create(
                    connection,
                    as_c_string(table_ref.schema).as_ptr(),
                    as_c_string(table_ref.name).as_ptr(),
                    &mut *appender,
                )
            };
            if rc != duckdb_state_DuckDBSuccess {
                return Err(Error::msg(
                    error_message_from_ptr(&duckdb_appender_error(*appender)).to_string(),
                )
                .context("While creating the `duckdb_appender` object"));
            }
            for column in E::columns() {
                duckdb_appender_add_column(*appender, as_c_string(column.name()).as_ptr());
            }
            let rows_affected = rows.len() as u64;
            for row in rows {
                for value in row {
                    let rc = match value {
                        Value::Boolean(Some(v), ..) => duckdb_append_bool(*appender, v),
                        Value::Int8(Some(v), ..) => duckdb_append_int8(*appender, v),
                        Value::Int16(Some(v), ..) => duckdb_append_int16(*appender, v),
                        Value::Int32(Some(v), ..) => duckdb_append_int32(*appender, v),
                        Value::Int64(Some(v), ..) => duckdb_append_int64(*appender, v),
                        Value::Int128(Some(v), ..) => {
                            duckdb_append_hugeint(*appender, i128_to_duckdb_hugeint(v))
                        }
                        Value::UInt8(Some(v), ..) => duckdb_append_uint8(*appender, v),
                        Value::UInt16(Some(v), ..) => duckdb_append_uint16(*appender, v),
                        Value::UInt32(Some(v), ..) => duckdb_append_uint32(*appender, v),
                        Value::UInt64(Some(v), ..) => duckdb_append_uint64(*appender, v),
                        Value::UInt128(Some(v), ..) => {
                            duckdb_append_uhugeint(*appender, u128_to_duckdb_uhugeint(v))
                        }
                        Value::Float32(Some(v), ..) => duckdb_append_float(*appender, v),
                        Value::Float64(Some(v), ..) => duckdb_append_double(*appender, v),
                        Value::Decimal(Some(v), width, scale) => {
                            let value = CBox::new(
                                duckdb_create_decimal(decimal_to_duckdb_decimal(&v, width, scale)),
                                |mut p| duckdb_destroy_value(&mut p),
                            );
                            duckdb_append_value(*appender, *value)
                        }
                        Value::Char(Some(v), ..) => {
                            duckdb_append_varchar(*appender, as_c_string(v.to_string()).as_ptr())
                        }
                        Value::Varchar(Some(v), ..) => {
                            duckdb_append_varchar(*appender, as_c_string(v).as_ptr())
                        }
                        Value::Blob(Some(v), ..) => duckdb_append_blob(
                            *appender,
                            v.as_ptr() as *const c_void,
                            v.len() as u64,
                        ),
                        Value::Date(Some(v), ..) => {
                            duckdb_append_date(*appender, date_to_duckdb_date(&v))
                        }
                        Value::Time(Some(v), ..) => {
                            duckdb_append_time(*appender, time_to_duckdb_time(&v))
                        }
                        Value::Timestamp(Some(v), ..) => duckdb_append_timestamp(
                            *appender,
                            primitive_date_time_to_duckdb_timestamp(&v),
                        ),
                        Value::TimestampWithTimezone(Some(v), ..) => duckdb_append_timestamp(
                            *appender,
                            offsetdatetime_to_duckdb_timestamp(&v),
                        ),
                        Value::Interval(Some(ref v), ..) => {
                            duckdb_append_interval(*appender, interval_to_duckdb_interval(&v))
                        }
                        Value::Uuid(Some(ref v), ..) => duckdb_append_value(
                            *appender,
                            duckdb_create_uuid(u128_to_duckdb_uhugeint(v.as_u128())),
                        ),
                        Value::List(Some(ref v), ty) => {
                            let logical_type = tank_value_to_duckdb_logical_type(&ty);
                            let values = v.iter().map(|v| tank_value_to_duckdb_value(v));
                            let value = CBox::new(
                                duckdb_create_list_value(
                                    *logical_type,
                                    values.map(|v| *v).collect::<Vec<_>>().as_mut_ptr(),
                                    v.len() as u64,
                                ),
                                |mut p| duckdb_destroy_value(&mut p),
                            );
                            duckdb_append_value(*appender, *value)
                        }
                        Value::Array(Some(ref v), ty, len) => {
                            let logical_type = tank_value_to_duckdb_logical_type(&*ty);
                            let values = v
                                .iter()
                                .map(|v| tank_value_to_duckdb_value(v))
                                .collect::<Vec<_>>();
                            let value = CBox::new(
                                duckdb_create_array_value(
                                    *logical_type,
                                    values.iter().map(|v| **v).collect::<Vec<_>>().as_mut_ptr(),
                                    len as u64,
                                ),
                                |mut p| duckdb_destroy_value(&mut p),
                            );
                            duckdb_append_value(*appender, *value)
                        }
                        Value::Map(Some(ref v), ..) => {
                            let logical_type = tank_value_to_duckdb_logical_type(&value);
                            let keys = v
                                .keys()
                                .map(|v| tank_value_to_duckdb_value(v))
                                .collect::<Vec<_>>();
                            let values = v
                                .values()
                                .map(|v| tank_value_to_duckdb_value(v))
                                .collect::<Vec<_>>();
                            let value = CBox::new(
                                duckdb_create_map_value(
                                    *logical_type,
                                    keys.iter().map(|v| **v).collect::<Vec<_>>().as_mut_ptr(),
                                    values.iter().map(|v| **v).collect::<Vec<_>>().as_mut_ptr(),
                                    v.len() as u64,
                                ),
                                |mut p| duckdb_destroy_value(&mut p),
                            );
                            duckdb_append_value(*appender, *value)
                        }
                        Value::Struct(Some(ref v), ..) => {
                            let values = v
                                .iter()
                                .map(|v| tank_value_to_duckdb_value(&v.1))
                                .collect::<Vec<_>>();
                            let value = CBox::new(
                                duckdb_create_struct_value(
                                    *tank_value_to_duckdb_logical_type(&value),
                                    values.iter().map(|v| **v).collect::<Vec<_>>().as_mut_ptr(),
                                ),
                                |mut p| duckdb_destroy_value(&mut p),
                            );
                            duckdb_append_value(*appender, *value)
                        }
                        _ => duckdb_append_null(*appender),
                    };
                    if rc != duckdb_state_DuckDBSuccess {
                        let error = Error::msg(
                            error_message_from_ptr(&duckdb_appender_error(*appender)).to_string(),
                        );
                        log::error!("{}", error);
                        return Err(error);
                    }
                }
                duckdb_appender_end_row(*appender);
            }
            Ok(RowsAffected {
                last_affected_id: None,
                rows_affected,
            })
        })
        .await?
    }
}

impl Connection for DuckDBConnection {
    #[allow(refining_impl_trait)]
    async fn connect(url: Cow<'static, str>) -> Result<DuckDBConnection> {
        let prefix = format!("{}://", <Self::Driver as Driver>::NAME);
        if !url.starts_with(&prefix) {
            let error = Error::msg(format!(
                "DuckDB connection url must start with `{}`",
                &prefix,
            ));
            log::error!("{:#}", error);
            return Err(error);
        }
        let mut parts = url.trim_start_matches(&prefix).splitn(2, '?');
        let path = parts
            .next()
            .ok_or(Error::msg(format!("Invalid database url `{}`", url,)))?;
        let params = parts.next().unwrap_or_default();
        let context = || format!("Invalid database url: `{}`", url);
        let path = decode(path)
            .with_context(context)
            .and_then(|v| CString::new(&*v).with_context(context))?;
        let mut config: CBox<duckdb_config> = CBox::new(ptr::null_mut(), |mut p| unsafe {
            duckdb_destroy_config(&mut p)
        });
        unsafe {
            let rc = duckdb_create_config(&mut *config);
            if rc != duckdb_state_DuckDBSuccess {
                let error = Error::msg("Cannot allocate the duckdb_config object")
                    .context(format!("Failed to connect to database url `{}`", url));
                log::error!("{:#}", error);
                return Err(error);
            }
        };
        for (key, value) in form_urlencoded::parse(params.as_bytes()) {
            let rc = unsafe {
                match &*key {
                    "mode" => duckdb_set_config(
                        *config,
                        c"access_mode".as_ptr(),
                        match &*value {
                            "ro" => c"READ_ONLY",
                            "rw" => c"READ_WRITE",
                            _ => {
                                let error = Error::msg("Unknown value {value:?} for `mode`, expected one of: `ro`, `rw`");
                                log::warn!("{:#}", error);
                                return Err(error);
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
                let error = Error::msg(format!("Error while setting config `{}={}`", key, value));
                log::warn!("{:#}", error);
                return Err(error);
            }
        }
        let mut database: duckdb_database = ptr::null_mut();
        let mut connection;
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
                let error = CStr::from_ptr(*error)
                    .to_str()
                    .context("While reading the error from `duckdb_get_or_create_from_cache`")?
                    .to_owned();
                return Err(Error::msg(error));
            };
            connection = CBox::new(ptr::null_mut(), |mut p| duckdb_disconnect(&mut p));
            let rc = duckdb_connect(database, &mut *connection);
            if rc != duckdb_state_DuckDBSuccess {
                let error = Error::msg(format!("Failed to connect to database url `{}`", url));
                log::error!("{:#}", error);
                return Err(error);
            };
        };
        Ok(DuckDBConnection {
            connection,
            transaction: false,
        })
    }

    #[allow(refining_impl_trait)]
    fn begin(&mut self) -> impl Future<Output = Result<DuckDBTransaction<'_>>> {
        DuckDBTransaction::new(self)
    }
}
