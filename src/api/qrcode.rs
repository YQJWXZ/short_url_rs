use actix_web::{error::ErrorInternalServerError, get, web, HttpResponse, Result};
use sqlx::SqlitePool;

#[get("/qrcode/{short_code}")]
pub async fn redirect_qrcode(
    short_code: web::Path<String>,
    db: web::Data<SqlitePool>,
) -> Result<HttpResponse> {
    let code = short_code.as_ref();
    // 查询长链接
    let url = sqlx::query!("SELECT long_url FROM short_urls WHERE short_code = ?", code)
        .fetch_optional(db.get_ref())
        .await
        .map_err(ErrorInternalServerError)?;

    match url {
        Some(url) => Ok(HttpResponse::Found()
            .append_header(("Location", url.long_url))
            .finish()),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use actix_web::{http, test, App};
    use chrono::Utc;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        db::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    async fn insert_test_url(pool: &SqlitePool, short_code: &str, long_url: &str) {
        sqlx::query(
            r#"
            INSERT INTO short_urls (long_url, short_code, created_at, user_id)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(long_url)
        .bind(short_code)
        .bind(Utc::now().to_rfc3339())
        .bind("test_user")
        .execute(pool)
        .await
        .expect("Failed to insert test URL");
    }

    #[actix_web::test]
    async fn test_redirect_qrcode_success() {
        // Setup
        let pool = setup_test_db().await;

        // Insert test URL
        insert_test_url(&pool, "qrtest", "https://example.com/qr").await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(redirect_qrcode),
        )
        .await;

        // Send request
        let req = test::TestRequest::get().uri("/qrcode/qrtest").to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::FOUND);

        let location = resp
            .headers()
            .get(http::header::LOCATION)
            .expect("No Location header");

        assert_eq!(location, "https://example.com/qr");
    }

    #[actix_web::test]
    async fn test_redirect_qrcode_not_found() {
        // Setup
        let pool = setup_test_db().await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(redirect_qrcode),
        )
        .await;

        // Send request for non-existent code
        let req = test::TestRequest::get()
            .uri("/qrcode/nonexistent")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }
}
