mod init;

#[cfg(test)]
mod tests {
    use super::init::init;
    use std::sync::Mutex;
    use tank_core::Connection;
    use tank_postgres::PostgresConnection;
    use tank_tests::execute_tests;
    use tank_tests::init_logs;
    use tank_tests::silent_logs;

    static MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn postgres() {
        init_logs();
        let _guard = MUTEX.lock().unwrap();

        // Unencrypted
        let (url, container) = init(false).await;
        let error_msg = format!("Could not connect to `{url}`");
        let connection = PostgresConnection::connect(url.into())
            .await
            .expect(&error_msg);
        execute_tests(connection).await;
        drop(container);

        // SSL
        let (url, container) = init(true).await;
        let error_msg = format!("Could not connect to `{url}`");
        let connection = PostgresConnection::connect(url.into())
            .await
            .expect(&error_msg);
        execute_tests(connection).await;
        drop(container);
    }

    #[tokio::test]
    async fn wrong_url() {
        silent_logs! {
            assert!(
                PostgresConnection::connect("mysql://some_url".into())
                    .await
                    .is_err()
            );
        }
    }
}
