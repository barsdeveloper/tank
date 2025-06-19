#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Mutex;
    use tank::Connection;
    use tank_duckdb::DuckDBConnection;
    use tank_tests::execute_tests;
    use tokio::fs;

    static MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn tests() {
        const DB_PATH: &'static str = "../target/debug/tesats.duckdb";
        let _guard = MUTEX.lock().unwrap();
        if Path::new(DB_PATH).exists() {
            fs::remove_file(DB_PATH).await.expect(
                format!("Failed to remove existing test database file {}", DB_PATH).as_str(),
            );
        }
        assert!(
            !Path::new(DB_PATH).exists(),
            "Database file should not exist before test"
        );
        let mut connection =
            DuckDBConnection::connect(format!("duckdb://{}?mode=rw", DB_PATH).as_str())
                .await
                .expect("Could not open the database");
        assert!(
            Path::new(DB_PATH).exists(),
            "Database file should be created after connection"
        );
        execute_tests(&mut connection).await;
    }
}
