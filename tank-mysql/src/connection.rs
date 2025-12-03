use crate::{MySQLDriver, MySQLQueryable, MySQLTransaction};
use mysql_async::{Conn, Opts};
use std::borrow::Cow;
use tank_core::{
    Connection, Driver, Error, ErrorContext, Result, impl_executor_transaction, truncate_long,
};
use url::Url;

pub struct MySQLConnection {
    pub(crate) conn: MySQLQueryable<Conn>,
}

impl_executor_transaction!(MySQLDriver, MySQLConnection, conn);

impl Connection for MySQLConnection {
    async fn connect(url: Cow<'static, str>) -> Result<MySQLConnection> {
        let context = || format!("While trying to connect to `{}`", truncate_long!(url));
        let prefix = format!("{}://", <Self::Driver as Driver>::NAME);
        if !url.starts_with(&prefix) {
            let error = Error::msg(format!(
                "MySQL connection url must start with `{}`",
                &prefix
            ))
            .context(context());
            log::error!("{:#}", error);
            return Err(error);
        }
        let url = Url::parse(&url).with_context(context)?;
        let config = Opts::from_url(url.as_str()).with_context(context)?;
        let connection = Conn::new(config).await.with_context(context)?;
        Ok(MySQLConnection {
            conn: MySQLQueryable {
                executor: connection,
            },
        })
    }

    #[allow(refining_impl_trait)]
    fn begin(&mut self) -> impl Future<Output = Result<MySQLTransaction<'_>>> {
        MySQLTransaction::new(self)
    }
}
