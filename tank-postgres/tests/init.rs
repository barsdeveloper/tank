use rcgen::{
    BasicConstraints, Certificate, CertificateParams, IsCa, KeyPair, SanType,
    generate_simple_self_signed,
};
use std::{
    env, future, net::IpAddr, path::PathBuf, process::Command, str::FromStr, time::Duration,
};
use tank_core::{
    Result,
    future::{BoxFuture, FutureExt},
};
use testcontainers::core::logs::{LogFrame, consumer::LogConsumer};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use tokio::fs;

struct TestcontainersLogConsumer;
impl LogConsumer for TestcontainersLogConsumer {
    fn accept<'a>(&'a self, record: &'a LogFrame) -> BoxFuture<'a, ()> {
        future::ready(match record {
            LogFrame::StdOut(bytes) => log::warn!(
                "{:?}",
                str::from_utf8(bytes.trim_ascii()).unwrap_or("Invalid error message")
            ),
            LogFrame::StdErr(bytes) => log::warn!(
                "{:?}",
                str::from_utf8(bytes.trim_ascii()).unwrap_or("Invalid error message")
            ),
        })
        .boxed()
    }
}

async fn generate_postgres_ssl_files() -> Result<()> {
    let mut ca_params = CertificateParams::new(vec!["tank_postgres".to_string()])?;
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let signing_key = KeyPair::generate()?;
    let ca_cert = ca_params.self_signed(&signing_key)?;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::write(path.join("tests/assets/root.crt"), ca_cert.pem()).await?;

    let mut server_params = CertificateParams::new(["localhost".to_string()])?;
    server_params.subject_alt_names = vec![
        SanType::DnsName("localhost".try_into().unwrap()),
        SanType::IpAddress(IpAddr::from_str("127.0.0.1").unwrap()),
    ];
    server_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let server_cert = server_params.self_signed(&signing_key)?;
    fs::write(path.join("tests/assets/server.crt"), server_cert.pem()).await?;
    fs::write(
        path.join("tests/assets/server.key"),
        signing_key.serialize_pem(),
    )
    .await?;

    let client_params = CertificateParams::new(vec!["tank-user".to_string()])?;
    let client_cert = client_params.self_signed(&signing_key)?;
    fs::write(path.join("tests/assets/root.crt"), client_cert.pem()).await?;

    Ok(())
}

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
        .with_db_name("military")
        .with_user("tank-user")
        .with_password("armored")
        .with_startup_timeout(Duration::from_secs(10))
        .with_log_consumer(TestcontainersLogConsumer);
    if ssl {
        generate_postgres_ssl_files()
            .await
            .expect("Could not create the certificate files for ssl session");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        container = container
            .with_copy_to(
                "/docker-entrypoint-initdb.d/pg_hba.conf",
                path.join("tests/assets/pg_hba.conf"),
            )
            .with_copy_to(
                "/docker-entrypoint-initdb.d/root.crt",
                path.join("tests/assets/root.crt"),
            )
            .with_copy_to(
                "/docker-entrypoint-initdb.d/server.crt",
                path.join("tests/assets/server.crt"),
            )
            .with_copy_to(
                "/docker-entrypoint-initdb.d/server.key",
                path.join("tests/assets/server.key"),
            )
            .with_copy_to(
                "/docker-entrypoint-initdb.d/00-ssl.sh",
                path.join("tests/assets/00-ssl.sh"),
            );
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
