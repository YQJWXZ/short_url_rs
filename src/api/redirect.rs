use crate::services::UrlService;
use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;

#[actix_web::get("/{short_code}")]
pub async fn redirect_to_long_url(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let short_code = path.into_inner();

    match UrlService::get_long_url(&pool, &short_code).await {
        Ok(Some(long_url)) => Ok(HttpResponse::Found()
            .append_header(("Location", long_url))
            .finish()),
        Ok(None) => Ok(HttpResponse::NotFound().body("Short URL not found or expired")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Internal server error")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::pb::abi::CreateShortUrlRequest;
    use crate::services::UrlService;
    use actix_web::{http, test, App};

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        db::run_migrations(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[actix_web::test]
    async fn test_redirect_to_long_url_success() {
        // Setup
        let pool = setup_test_db().await;

        // Create a test URL
        let request = CreateShortUrlRequest {
            long_url: "https://example.com".to_string(),
            custom_code: Some("testcode".to_string()),
            timeout: None,
            user_id: "test_user".to_string(),
        };

        UrlService::create_short_url(&pool, request)
            .await
            .expect("Failed to create test URL");

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(redirect_to_long_url),
        )
        .await;

        // Send request
        let req = test::TestRequest::get().uri("/testcode").to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::FOUND);

        let location = resp
            .headers()
            .get(http::header::LOCATION)
            .expect("No Location header");

        assert_eq!(location, "https://example.com");
    }

    #[actix_web::test]
    async fn test_redirect_to_nonexistent_url() {
        // Setup
        let pool = setup_test_db().await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(redirect_to_long_url),
        )
        .await;

        // Send request for non-existent code
        let req = test::TestRequest::get().uri("/nonexistent").to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }
}
