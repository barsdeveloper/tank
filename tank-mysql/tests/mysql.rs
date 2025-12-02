mod init;

#[cfg(test)]
mod tests {
    use crate::init::init;
    use std::sync::Mutex;
    use tank_core::Driver;
    use tank_mysql::MySQLDriver;
    use tank_tests::{execute_tests, init_logs};

    static MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn mysql() {
        init_logs();
        let _guard = MUTEX.lock().unwrap();

        // Unencrypted
        let (url, container) = init(false).await;
        let container = container.expect("Could not launch container");
        let error_msg = format!("Could not connect to `{url}`");
        let driver = MySQLDriver::new();
        let connection = driver
            .connect(url.into())
            .await
            .expect("Could not open the database");
        execute_tests(connection).await;
    }
}
