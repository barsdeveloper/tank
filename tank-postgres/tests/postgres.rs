mod init;

#[cfg(test)]
mod tests {
    use super::init::init;
    use std::sync::Mutex;
    use tank_core::Connection;
    use tank_core::Error;
    use tank_postgres::PostgresConnection;
    use tank_tests::execute_tests;
    use tank_tests::init_logs;

    static MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn postgres() {
        init_logs();
        let _guard = MUTEX.lock().unwrap();
        let (url, container) = init().await;
        let error_msg = format!("Could not connect to `{url}`");
        let connection = PostgresConnection::connect(url.into())
            .await
            .expect(&error_msg);
        execute_tests(connection).await;
        if let Some(container) = container
            && let Err(e) = container.stop_with_timeout(Some(10)).await
        {
            log::error!(
                "{:#}",
                Error::new(e).context("While stopping the container")
            );
        }
    }

    #[tokio::test]
    async fn wrong_url() {
        assert!(
            PostgresConnection::connect("mysql://some_value".into())
                .await
                .is_err()
        );
    }
}
