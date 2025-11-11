#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use tank_core::Driver;
    use tank_mysql::MySQLDriver;
    use tank_tests::{execute_tests, init_logs};

    static MUTEX: Mutex<()> = Mutex::new(());

    // #[tokio::test]
    // async fn mysql() {
    //     init_logs();
    //     const URL: &'static str = "mysql://";
    //     let _guard = MUTEX.lock().unwrap();
    //     let driver = MySQLDriver::new();
    //     let connection = driver
    //         .connect(URL.into())
    //         .await
    //         .expect("Could not open the database");
    //     execute_tests(connection).await;
    // }
}
