use std::{env, path::PathBuf, process::Command, time::Duration};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};

pub async fn init(ssl: bool) -> (String, Option<ContainerAsync<Postgres>>) {
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
    let mut container = Postgres::default()
        .with_user("tank-user")
        .with_password("armored")
        .with_db_name("military")
        .with_startup_timeout(Duration::from_secs(10));
    if ssl {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        container = container
            .with_copy_to(
                "/var/lib/postgresql/server.crt",
                manifest_dir.join("tests/assets/server.crt"),
            )
            .with_copy_to(
                "/var/lib/postgresql/server.key",
                manifest_dir.join("tests/assets/server.key"),
            )
            .with_copy_to(
                "/docker-entrypoint-initdb.d/ssl-setup.sh",
                manifest_dir.join("tests/assets/00-ssl-setup.sh"),
            )
            .with_cmd([
                "-c",
                "ssl=on",
                "-c",
                "ssl_cert_file=/var/lib/postgresql/server.crt",
                "-c",
                "ssl_key_file=/var/lib/postgresql/server.key",
            ]);
    }
    let container = container
        .start()
        .await
        .expect("Could not start the container");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Cannot get the port of Postgres");
    (
        format!(
            "postgres://tank-user:armored@127.0.0.1:{port}/military{}",
            if ssl { "?sslmode=require" } else { "" }
        ),
        Some(container),
    )
}
