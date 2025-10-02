use crate::{
    CBox, SqliteDriver, SqlitePrepared, SqliteTransaction, error_message_from_ptr,
    extract::{extract_name, extract_value},
};
use async_stream::{stream, try_stream};
use libsqlite3_sys::*;
use std::{
    borrow::Cow,
    ffi::{CStr, CString, c_char, c_int},
    pin::pin,
    ptr,
    sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    },
};
use tank_core::{
    Connection, Driver, Error, ErrorContext, Executor, Query, QueryResult, Result, RowLabeled,
    RowsAffected,
    future::Either,
    printable_query,
    stream::{Stream, StreamExt, TryStreamExt},
};
use tokio::task::spawn_blocking;

pub struct SqliteConnection {
    pub(crate) connection: CBox<*mut sqlite3>,
    pub(crate) _transaction: bool,
}

impl SqliteConnection {
    pub(crate) fn run_prepared(
        &mut self,
        statement: CBox<*mut sqlite3_stmt>,
    ) -> impl Stream<Item = Result<QueryResult>> {
        unsafe {
            stream! {
                let count = sqlite3_column_count(*statement);
                let labels = (0..count)
                    .map(|i| extract_name(*statement, i))
                    .collect::<Result<Arc<[_]>>>()?;
                loop {
                    match sqlite3_step(*statement) {
                        SQLITE_BUSY => {
                            continue;
                        }
                        SQLITE_DONE => {
                            if sqlite3_stmt_readonly(*statement) == 0 {
                                yield Ok(QueryResult::Affected(RowsAffected {
                                    rows_affected: sqlite3_changes64(*self.connection) as u64,
                                    last_affected_id: Some(sqlite3_last_insert_rowid(*self.connection)),
                                }))
                            }
                            break;
                        }
                        SQLITE_ROW => {
                            yield Ok(QueryResult::RowLabeled(RowLabeled {
                                labels: labels.clone(),
                                values: (0..count).map(|i| extract_value(*statement, i)).collect()?,
                            }))
                        }
                        _ => {
                            let error = Error::msg(
                                error_message_from_ptr(&sqlite3_errmsg(sqlite3_db_handle(*statement)))
                                    .to_string(),
                            );
                            yield Err(error);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn run_unprepared(
        &mut self,
        sql: String,
    ) -> impl Stream<Item = Result<QueryResult>> {
        try_stream! {
            let mut len = sql.trim_end().len();
            let buff = sql.into_bytes();
            let mut it = CBox::new(buff.as_ptr() as *const c_char, |_| {});
            loop {
                let connection = CBox::new(*self.connection, |_| {});
                let sql = CBox::new(*it, |_| {});
                let (statement, tail) = spawn_blocking(move || unsafe {
                    let mut statement = CBox::new(ptr::null_mut(), |p| {
                        sqlite3_finalize(p);
                    });
                    let mut sql_tail = CBox::new(ptr::null(), |_| {});
                    let rc = sqlite3_prepare_v2(
                        *connection,
                        *sql,
                        len as c_int,
                        &mut *statement,
                        &mut *sql_tail,
                    );
                    if rc != SQLITE_OK {
                        return Err(Error::msg(
                            error_message_from_ptr(&sqlite3_errmsg(*connection)).to_string(),
                        ));
                    }
                    Ok((statement, sql_tail))
                })
                .await??;
                let mut stream = pin!(self.run_prepared(statement));
                while let Some(value) = stream.next().await {
                    yield value?
                }
                unsafe {
                    len = if *tail != ptr::null() {
                        len - tail.offset_from_unsigned(*it)
                    } else {
                        0
                    };
                    if len == 0 {
                        break;
                    }
                }
                *it = *tail;
            }
        }
    }
}

impl Executor for SqliteConnection {
    type Driver = SqliteDriver;

    fn driver(&self) -> &Self::Driver {
        &SqliteDriver {}
    }

    async fn prepare(&mut self, sql: String) -> Result<Query<Self::Driver>> {
        let connection = AtomicPtr::new(*self.connection);
        let context = format!(
            "While preparing the query:\n{}",
            printable_query!(sql.as_str())
        );
        let prepared = spawn_blocking(move || unsafe {
            let connection = connection.load(Ordering::Relaxed);
            let len = sql.len();
            let sql = CString::new(sql.as_bytes())?;
            let mut statement = CBox::new(ptr::null_mut(), |p| {
                sqlite3_finalize(p);
            });
            let mut tail = ptr::null();
            let rc = sqlite3_prepare_v2(
                connection,
                sql.as_ptr(),
                len as c_int,
                &mut *statement,
                &mut tail,
            );
            if rc != SQLITE_OK {
                let error =
                    Error::msg(error_message_from_ptr(&sqlite3_errmsg(connection)).to_string())
                        .context(context);
                log::error!("{:#}", error);
                return Err(error);
            }
            if tail != ptr::null() && *tail != '\0' as i8 {
                let error = Error::msg(format!(
                    "Cannot prepare more than one statement at a time (remaining: {})",
                    CStr::from_ptr(tail).to_str().unwrap_or("unprintable")
                ))
                .context(context);
                log::error!("{:#}", error);
                return Err(error);
            }
            Ok(statement)
        })
        .await?;
        Ok(SqlitePrepared::new(prepared?).into())
    }

    fn run(
        &mut self,
        query: Query<Self::Driver>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        let context = Arc::new(format!("While executing the query:\n{}", query));
        match query {
            Query::Raw(sql) => Either::Left(self.run_unprepared(sql)),
            Query::Prepared(prepared) => Either::Right(self.run_prepared(prepared.statement)),
        }
        .map_err(move |e| {
            let e = e.context(context.clone());
            log::error!("{:#}", e);
            e
        })
    }
}

impl Connection for SqliteConnection {
    #[allow(refining_impl_trait)]
    async fn connect(url: Cow<'static, str>) -> Result<SqliteConnection> {
        let prefix = format!("{}://", <Self::Driver as Driver>::NAME);
        if !url.starts_with(&prefix) {
            let error = Error::msg(format!(
                "Sqlite connection url must start with `{}`",
                &prefix
            ));
            log::error!("{:#}", error);
            return Err(error);
        }
        let url = CString::new(format!("file:{}", url.trim_start_matches(&prefix)))
            .with_context(|| format!("Invalid database url `{}`", url))?;
        let mut connection;
        unsafe {
            connection = CBox::new(ptr::null_mut(), |p| {
                if sqlite3_close(p) != SQLITE_OK {
                    log::error!("Could not close sqlite connection")
                }
            });
            let rc = sqlite3_open_v2(
                url.as_ptr(),
                &mut *connection,
                SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_URI,
                ptr::null(),
            );
            if rc != SQLITE_OK {
                let error =
                    Error::msg(error_message_from_ptr(&sqlite3_errmsg(*connection)).to_string())
                        .context(format!(
                            "Failed to connect to database url `{}`",
                            url.to_str().unwrap_or("unprintable value")
                        ));
                log::error!("{:#}", error);
                return Err(error);
            }
        }
        Ok(Self {
            connection,
            _transaction: false,
        })
    }

    #[allow(refining_impl_trait)]
    fn begin(&mut self) -> impl Future<Output = Result<SqliteTransaction>> {
        SqliteTransaction::new(self)
    }
}
