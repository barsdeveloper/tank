use std::{env, process::Command, time::Duration};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};

pub async fn init() -> (String, Option<ContainerAsync<Postgres>>) {
    if let Ok(url) = env::var("TANK_POSTGRES_TEST") {
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
    // let CONTAINER: OnceCell<ContainerAsync<Postgres>> = OnceCell::const_new();
    let container = Postgres::default()
        .with_user("tank-user")
        .with_password("armored")
        .with_db_name("military")
        .with_startup_timeout(Duration::from_secs(10))
        .start()
        .await
        .expect("Failed to start Postgres container, most likely docker is not running.");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Cannot get the port of Postgres");
    (
        format!("postgres://tank-user:armored@127.0.0.1:{port}/military"),
        Some(container),
    )
}
