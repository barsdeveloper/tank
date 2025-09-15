use crate::{
    CBox, SqliteDriver, SqlitePrepared, error_message_from_ptr,
    extract::{extract_name, extract_value},
};
use async_stream::{stream, try_stream};
use libsqlite3_sys::{
    SQLITE_BUSY, SQLITE_DONE, SQLITE_OK, SQLITE_OPEN_URI, SQLITE_ROW, sqlite3, sqlite3_close,
    sqlite3_column_count, sqlite3_db_handle, sqlite3_errmsg, sqlite3_finalize, sqlite3_open_v2,
    sqlite3_prepare_v2, sqlite3_step, sqlite3_stmt,
};
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
    Connection, Context, Driver, Error, Executor, Query, QueryResult, Result, RowLabeled,
    future::Either,
    printable_query,
    stream::{Stream, StreamExt},
};
use tokio::task::spawn_blocking;

pub struct SqliteConnection {
    pub(crate) connection: CBox<*mut sqlite3>,
    pub(crate) transaction: bool,
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
                            log::error!("{}", error);
                            yield Err(error);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn run_unprepared(
        &mut self,
        sql: Arc<str>,
    ) -> impl Stream<Item = Result<QueryResult>> {
        let connection = CBox::new(*self.connection, |_| {});
        try_stream! {
            let len = sql.len();
            let (statement, _remaining) = spawn_blocking(move || unsafe {
                let mut statement = CBox::new(ptr::null_mut(), |p| {
                    sqlite3_finalize(p);
                });
                let mut sql_tail = ptr::null();
                let c_ptr = sql.as_ptr() as *const c_char;
                let rc = sqlite3_prepare_v2(
                    *connection,
                    c_ptr,
                    len as c_int,
                    &mut *statement,
                    &mut sql_tail,
                );
                if rc != SQLITE_OK {
                    return Err(Error::msg(
                        error_message_from_ptr(&sqlite3_errmsg(*connection)).to_string(),
                    ));
                }
                let remaining_sql = CStr::from_ptr(sql_tail).to_owned();
                Ok((statement, remaining_sql.to_owned()))
            })
            .await??;
            let mut stream = pin!(self.run_prepared(statement));
            while let Some(value) = stream.next().await {
                yield value?
            }
        }
    }
}

impl Executor for SqliteConnection {
    type Driver = SqliteDriver;

    fn driver(&self) -> &Self::Driver {
        &SqliteDriver {}
    }

    async fn prepare(
        &mut self,
        query: String,
    ) -> Result<Query<<Self::Driver as Driver>::Prepared>> {
        let connection = AtomicPtr::new(*self.connection);
        let context = format!(
            "While preparing the query:\n{}",
            printable_query!(query.as_str())
        );
        let prepared = spawn_blocking(move || unsafe {
            let connection = connection.load(Ordering::Relaxed);
            let len = query.len();
            let sql = match CString::new(query.as_bytes()) {
                Ok(query) => query,
                Err(e) => {
                    let error =
                        Error::new(e).context("Could not create a CString from the query String");
                    log::error!("{}", error);
                    return Err(error);
                }
            };
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
                log::error!("{}", error);
                return Err(error);
            }
            if tail != ptr::null() {
                let error =
                    Error::msg("Cannot prepare more than one statement at a time").context(context);
                log::error!("{}", error);
                return Err(error);
            }
            Ok(statement)
        })
        .await?;
        Ok(SqlitePrepared::new(prepared?).into())
    }

    fn run(
        &mut self,
        query: Query<<Self::Driver as Driver>::Prepared>,
    ) -> impl Stream<Item = Result<QueryResult>> + Send {
        match query {
            Query::Raw(sql) => Either::Left(self.run_unprepared(sql)),
            Query::Prepared(prepared) => Either::Right(self.run_prepared(prepared.statement)),
        }
    }
}

impl Connection for SqliteConnection {
    #[allow(refining_impl_trait)]
    async fn connect(url: Cow<'static, str>) -> Result<SqliteConnection> {
        let prefix = format!("{}://", <Self::Driver as Driver>::NAME);
        if !url.starts_with(&prefix) {
            return Err(Error::msg(format!(
                "Expected sqlite connection url to start with `{}`",
                &prefix
            )));
        }
        let context = || format!("Error while decoding connection URL: `{}`", url);
        let url = CString::new(url.trim_start_matches(&prefix)).with_context(context)?;
        let mut connection: CBox<*mut sqlite3>;
        unsafe {
            connection = CBox::new(ptr::null_mut(), |p| {
                sqlite3_close(p);
            });
            sqlite3_open_v2(url.as_ptr(), &mut *connection, SQLITE_OPEN_URI, ptr::null());
        }
        Ok(Self {
            connection,
            transaction: false,
        })
    }
}
