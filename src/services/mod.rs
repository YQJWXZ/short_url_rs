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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::pb::abi::CreateShortUrlRequest;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Once;

    static INIT: Once = Once::new();

    // Helper function to set up an in-memory test database
    async fn setup_test_db() -> SqlitePool {
        // Initialize the logger only once
        INIT.call_once(|| {
            env_logger::builder().is_test(true).init();
        });

        // Create an in-memory database for testing
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        // Run migrations to set up the schema
        db::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_create_short_url() {
        let pool = setup_test_db().await;

        // Create a test request
        let request = CreateShortUrlRequest {
            long_url: "https://example.com".to_string(),
            custom_code: None,
            timeout: None,
            user_id: "test_user".to_string(),
        };

        // Create a short URL
        let result = UrlService::create_short_url(&pool, request).await;
        assert!(result.is_ok());

        let short_url = result.unwrap();
        assert_eq!(short_url.long_url, "https://example.com");
        assert_eq!(short_url.user_id, "test_user");
        assert_eq!(short_url.short_code.len(), 6);
        assert!(short_url.expires_at.is_none());
    }

    #[tokio::test]
    async fn test_create_short_url_with_custom_code() {
        let pool = setup_test_db().await;

        // Create a test request with custom code
        let request = CreateShortUrlRequest {
            long_url: "https://example.com".to_string(),
            custom_code: Some("custom".to_string()),
            timeout: None,
            user_id: "test_user".to_string(),
        };

        // Create a short URL
        let result = UrlService::create_short_url(&pool, request).await;
        assert!(result.is_ok());

        let short_url = result.unwrap();
        assert_eq!(short_url.short_code, "custom");
    }

    #[tokio::test]
    async fn test_create_short_url_with_timeout() {
        let pool = setup_test_db().await;

        // Create a test request with timeout
        let request = CreateShortUrlRequest {
            long_url: "https://example.com".to_string(),
            custom_code: None,
            timeout: Some(3600), // 1 hour
            user_id: "test_user".to_string(),
        };

        // Create a short URL
        let result = UrlService::create_short_url(&pool, request).await;
        assert!(result.is_ok());

        let short_url = result.unwrap();
        assert!(short_url.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_create_short_url_with_invalid_url() {
        let pool = setup_test_db().await;

        // Create a test request with invalid URL
        let request = CreateShortUrlRequest {
            long_url: "not-a-valid-url".to_string(),
            custom_code: None,
            timeout: None,
            user_id: "test_user".to_string(),
        };

        // Create a short URL should fail
        let result = UrlService::create_short_url(&pool, request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid URL format");
    }

    #[tokio::test]
    async fn test_create_short_url_with_duplicate_custom_code() {
        let pool = setup_test_db().await;

        // Create first URL with custom code
        let request1 = CreateShortUrlRequest {
            long_url: "https://example.com".to_string(),
            custom_code: Some("duplicate".to_string()),
            timeout: None,
            user_id: "test_user".to_string(),
        };

        let result1 = UrlService::create_short_url(&pool, request1).await;
        assert!(result1.is_ok());

        // Try to create second URL with same custom code
        let request2 = CreateShortUrlRequest {
            long_url: "https://another-example.com".to_string(),
            custom_code: Some("duplicate".to_string()),
            timeout: None,
            user_id: "test_user".to_string(),
        };

        let result2 = UrlService::create_short_url(&pool, request2).await;
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err(), "Custom code already exists");
    }

    #[tokio::test]
    async fn test_get_long_url() {
        let pool = setup_test_db().await;

        // Create a short URL first
        let request = CreateShortUrlRequest {
            long_url: "https://example.com".to_string(),
            custom_code: Some("testcode".to_string()),
            timeout: None,
            user_id: "test_user".to_string(),
        };

        let create_result = UrlService::create_short_url(&pool, request).await;
        assert!(create_result.is_ok());

        // Now get the long URL
        let result = UrlService::get_long_url(&pool, "testcode").await;
        assert!(result.is_ok());

        let long_url_opt = result.unwrap();
        assert!(long_url_opt.is_some());
        assert_eq!(long_url_opt.unwrap(), "https://example.com");
    }

    #[tokio::test]
    async fn test_get_nonexistent_url() {
        let pool = setup_test_db().await;

        // Try to get a non-existent short URL
        let result = UrlService::get_long_url(&pool, "nonexistent").await;
        assert!(result.is_ok());

        let long_url_opt = result.unwrap();
        assert!(long_url_opt.is_none());
    }

    #[tokio::test]
    async fn test_get_user_urls() {
        let pool = setup_test_db().await;

        // Create multiple short URLs for the same user
        for i in 1..=3 {
            let request = CreateShortUrlRequest {
                long_url: format!("https://example{}.com", i),
                custom_code: Some(format!("code{}", i)),
                timeout: None,
                user_id: "test_user".to_string(),
            };

            let result = UrlService::create_short_url(&pool, request).await;
            assert!(result.is_ok());
        }

        // Create a URL for another user
        let other_request = CreateShortUrlRequest {
            long_url: "https://other.com".to_string(),
            custom_code: Some("othercode".to_string()),
            timeout: None,
            user_id: "other_user".to_string(),
        };

        let other_result = UrlService::create_short_url(&pool, other_request).await;
        assert!(other_result.is_ok());

        // Get URLs for test_user
        let result = UrlService::get_user_urls(&pool, "test_user").await;
        assert!(result.is_ok());

        let urls = result.unwrap();
        assert_eq!(urls.len(), 3);

        // Verify all URLs belong to test_user
        for url in urls {
            assert_eq!(url.user_id, "test_user");
        }
    }

    #[tokio::test]
    async fn test_delete_short_url() {
        let pool = setup_test_db().await;

        // Create a short URL
        let request = CreateShortUrlRequest {
            long_url: "https://example.com".to_string(),
            custom_code: Some("deleteme".to_string()),
            timeout: None,
            user_id: "test_user".to_string(),
        };

        let create_result = UrlService::create_short_url(&pool, request).await;
        assert!(create_result.is_ok());

        let short_url = create_result.unwrap();

        // Delete the URL
        let delete_result = UrlService::delete_short_url(&pool, short_url.id, "test_user").await;
        assert!(delete_result.is_ok());
        assert!(delete_result.unwrap());

        // Verify it's deleted
        let get_result = UrlService::get_long_url(&pool, "deleteme").await;
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_url() {
        let pool = setup_test_db().await;

        // Try to delete a non-existent URL
        let delete_result = UrlService::delete_short_url(&pool, 9999, "test_user").await;
        assert!(delete_result.is_ok());
        assert!(!delete_result.unwrap());
    }

    #[tokio::test]
    async fn test_delete_url_wrong_user() {
        let pool = setup_test_db().await;

        // Create a short URL
        let request = CreateShortUrlRequest {
            long_url: "https://example.com".to_string(),
            custom_code: Some("usertest".to_string()),
            timeout: None,
            user_id: "owner_user".to_string(),
        };

        let create_result = UrlService::create_short_url(&pool, request).await;
        assert!(create_result.is_ok());

        let short_url = create_result.unwrap();

        // Try to delete with wrong user
        let delete_result = UrlService::delete_short_url(&pool, short_url.id, "wrong_user").await;
        assert!(delete_result.is_ok());
        assert!(!delete_result.unwrap());

        // Verify it's not deleted
        let get_result = UrlService::get_long_url(&pool, "usertest").await;
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_some());
    }
}
