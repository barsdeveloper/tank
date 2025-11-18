use std::{borrow::Cow, env, future, path::PathBuf, process::Command, time::Duration};
use tank_core::{
    future::{BoxFuture, FutureExt},
    indoc::indoc,
};
use testcontainers_modules::{
    mysql::Mysql,
    testcontainers::{
        ContainerAsync, ImageExt,
        core::logs::{LogFrame, consumer::LogConsumer},
        runners::AsyncRunner,
    },
};

struct TestcontainersLogConsumer;
impl LogConsumer for TestcontainersLogConsumer {
    fn accept<'a>(&'a self, record: &'a LogFrame) -> BoxFuture<'a, ()> {
        let log = str::from_utf8(record.bytes())
            .unwrap_or("Invalid error message")
            .trim();
        future::ready(if !log.is_empty() {
            match record {
                LogFrame::StdOut(..) => log::trace!("{log}",),
                LogFrame::StdErr(..) => log::debug!("{log}"),
            }
        })
        .boxed()
    }
}

pub async fn init(ssl: bool) -> (String, Option<ContainerAsync<Mysql>>) {
    if let Ok(url) = env::var("TANK_MYSQL_TEST") {
        return (url, None);
    };
    if !Command::new("docker")
        .arg("ps")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        log::error!("Cannot access docker");
    }
    let container = Mysql::default()
        .with_init_sql(
            indoc! {r#"
                CREATE DATABASE mysql_database;
                CREATE USER 'tank-mysql-user'@'%' IDENTIFIED BY 'Sup3r$ecur3';
                FLUSH PRIVILEGES;
                GRANT ALL PRIVILEGES ON *.* TO 'tank-mysql-user'@'%';
                DROP USER IF EXISTS 'root'@'localhost';
                DROP USER IF EXISTS 'root'@'127.0.0.1';
                DROP USER IF EXISTS 'root'@'::1';
                FLUSH PRIVILEGES;
            "#}
            .to_string()
            .into_bytes(),
        )
        .with_startup_timeout(Duration::from_secs(60))
        .with_log_consumer(TestcontainersLogConsumer);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if ssl {}
    let container = container
        .start()
        .await
        .expect("Could not start the container");
    let port = container
        .get_host_port_ipv4(3306)
        .await
        .expect("Cannot get the port of Mysql");
    (
        format!(
            "mysql://tank-mysql-user:Sup3r$ecur3@localhost:{port}/mysql_database{}",
            if ssl {
                Cow::Owned(format!(
                    "?sslmode=require&sslrootcert={}&sslcert={}&sslkey={}",
                    path.join("tests/assets/root.crt").to_str().unwrap(),
                    path.join("tests/assets/client.crt").to_str().unwrap(),
                    path.join("tests/assets/client.key").to_str().unwrap(),
                ))
            } else {
                Cow::Borrowed("")
            }
        ),
        Some(container),
    )
}
