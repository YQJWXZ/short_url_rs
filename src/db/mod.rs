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
