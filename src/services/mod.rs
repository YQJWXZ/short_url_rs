use crate::pb::abi::{CreateShortUrlRequest, ShortUrl};
use crate::utils::short_code_generator::generate_short_code;
use crate::utils::url_validator::is_valid_url;
use chrono::Utc;
use sqlx::{Row, SqlitePool};

pub struct UrlService;

impl UrlService {
    pub async fn create_short_url(
        pool: &SqlitePool,
        request: CreateShortUrlRequest,
    ) -> Result<ShortUrl, String> {
        // Validate URL
        if !is_valid_url(&request.long_url) {
            return Err("Invalid URL format".to_string());
        }

        // Generate or use custom short code
        let short_code = match request.custom_code {
            Some(code) => {
                // Check if custom code already exists
                if Self::code_exists(pool, &code).await? {
                    return Err("Custom code already exists".to_string());
                }
                code
            }
            None => {
                // Generate unique code
                let mut code = generate_short_code();
                while Self::code_exists(pool, &code).await? {
                    code = generate_short_code();
                }
                code
            }
        };

        // Calculate expiration time
        let expires_at = request
            .timeout
            .map(|timeout| Utc::now() + chrono::Duration::seconds(timeout));

        // Insert into database
        let result = sqlx::query(
            r#"
            INSERT INTO short_urls (long_url, short_code, created_at, expires_at, user_id)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&request.long_url)
        .bind(&short_code)
        .bind(Utc::now().to_rfc3339())
        .bind(expires_at.map(|dt| dt.to_rfc3339()))
        .bind(&request.user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        // Fetch the created record
        let short_url = sqlx::query_as::<_, ShortUrl>("SELECT * FROM short_urls WHERE id = ?")
            .bind(result.last_insert_rowid())
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(short_url)
    }

    pub async fn get_long_url(
        pool: &SqlitePool,
        short_code: &str,
    ) -> Result<Option<String>, String> {
        let result = sqlx::query(
            r#"
            SELECT long_url, expires_at FROM short_urls
            WHERE short_code = ? AND (expires_at IS NULL OR expires_at > ?)
            "#,
        )
        .bind(short_code)
        .bind(Utc::now().to_rfc3339())
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.map(|row| row.get("long_url")))
    }

    pub async fn get_user_urls(pool: &SqlitePool, user_id: &str) -> Result<Vec<ShortUrl>, String> {
        let urls = sqlx::query_as::<_, ShortUrl>(
            "SELECT * FROM short_urls WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        Ok(urls)
    }

    pub async fn delete_short_url(
        pool: &SqlitePool,
        id: i64,
        user_id: &str,
    ) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM short_urls WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(result.rows_affected() > 0)
    }

    async fn code_exists(pool: &SqlitePool, code: &str) -> Result<bool, String> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM short_urls WHERE short_code = ?")
            .bind(code)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let count: i64 = result.get("count");
        Ok(count > 0)
    }
}
