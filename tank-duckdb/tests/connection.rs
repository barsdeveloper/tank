#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Mutex};
    use tank_core::Connection;
    use tank_duckdb::DuckDBConnection;
    use tokio::fs;

    static MUTEX: Mutex<()> = Mutex::new(());

    #[tokio::test]
    async fn create_database() {
        const DB_PATH: &'static str = "../target/debug/test.db";
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
        DuckDBConnection::connect(format!("duckdb://{}?mode=rw", DB_PATH).as_str())
            .await
            .expect("Could not open the database");
        assert!(
            Path::new(DB_PATH).exists(),
            "Database file should be created after connection"
        );
    }

    #[tokio::test]
    async fn connect() {}
}
