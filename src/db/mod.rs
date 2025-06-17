use sqlx::{migrate::MigrateDatabase, Executor, SqlitePool};

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create database if it doesn't exist
    if !sqlx::Sqlite::database_exists(database_url)
        .await
        .unwrap_or(false)
    {
        sqlx::Sqlite::create_database(database_url).await?;
    }

    let pool = SqlitePool::connect(database_url).await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    pool.execute(
        r#"
        CREATE TABLE IF NOT EXISTS short_urls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            long_url TEXT NOT NULL,
            short_code TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL,
            expires_at TEXT,
            user_id TEXT NOT NULL
        )
        "#,
    )
    .await?;

    // Create index on short_code for faster lookups
    pool.execute("CREATE INDEX IF NOT EXISTS idx_short_code ON short_urls(short_code)")
        .await?;

    // Create index on user_id for faster user queries
    pool.execute("CREATE INDEX IF NOT EXISTS idx_user_id ON short_urls(user_id)")
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    #[tokio::test]
    async fn test_create_pool() {
        // Use in-memory database for testing
        let database_url = "sqlite::memory:";

        let result = create_pool(database_url).await;
        assert!(result.is_ok());

        let pool = result.unwrap();
        assert!(pool.acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_run_migrations() {
        // Create a test pool
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        // Run migrations
        let result = run_migrations(&pool).await;
        assert!(result.is_ok());

        // Verify the table was created
        let table_exists =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='short_urls'")
                .fetch_optional(&pool)
                .await
                .expect("Failed to check if table exists");

        assert!(table_exists.is_some());

        // Verify the indexes were created
        let indexes = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='short_urls'",
        )
        .fetch_all(&pool)
        .await
        .expect("Failed to check if indexes exist");

        let index_names: Vec<String> = indexes.iter().map(|row| row.get("name")).collect();
        assert!(index_names.contains(&"idx_short_code".to_string()));
        assert!(index_names.contains(&"idx_user_id".to_string()));
    }

    #[tokio::test]
    async fn test_table_structure() {
        // Create a test pool and run migrations
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");
        run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        // Get table info
        let columns = sqlx::query("PRAGMA table_info(short_urls)")
            .fetch_all(&pool)
            .await
            .expect("Failed to get table info");

        // Check column names and types
        let column_names: Vec<String> = columns.iter().map(|row| row.get("name")).collect();
        assert!(column_names.contains(&"id".to_string()));
        assert!(column_names.contains(&"long_url".to_string()));
        assert!(column_names.contains(&"short_code".to_string()));
        assert!(column_names.contains(&"created_at".to_string()));
        assert!(column_names.contains(&"expires_at".to_string()));
        assert!(column_names.contains(&"user_id".to_string()));

        // Check primary key
        let pk_column: String = columns
            .iter()
            .filter(|row| {
                let pk: i64 = row.get("pk");
                pk > 0
            })
            .map(|row| row.get("name"))
            .next()
            .expect("No primary key found");

        assert_eq!(pk_column, "id");
    }
}
