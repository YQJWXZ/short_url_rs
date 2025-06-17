use crate::pb::abi::{CreateShortUrlRequest, ShortUrlResponse};
use crate::pb::ApiResponse;
use crate::services::UrlService;
use crate::utils::url_validator::{is_valid_url, normalize_url};
use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;

#[actix_web::post("/shorten")]
pub async fn create_short_url(
    pool: web::Data<SqlitePool>,
    request: web::Json<CreateShortUrlRequest>,
) -> Result<HttpResponse> {
    let mut req = request.into_inner();

    // 先验证URL是否有效，再规范化
    if !is_valid_url(&req.long_url) && !is_valid_url(&normalize_url(&req.long_url)) {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error("Invalid URL format")));
    }

    req.long_url = normalize_url(&req.long_url);

    match UrlService::create_short_url(&pool, req).await {
        Ok(short_url) => {
            let response = short_url.to_response("http://localhost:8080");
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                "Short URL created successfully",
                response,
            )))
        }
        Err(err) => Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(&err))),
    }
}

#[actix_web::get("/urls/{user_id}")]
pub async fn get_short_urls(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    match UrlService::get_user_urls(&pool, &user_id).await {
        Ok(urls) => {
            let responses: Vec<ShortUrlResponse> = urls
                .into_iter()
                .map(|url| url.to_response("http://localhost:8080"))
                .collect();
            Ok(HttpResponse::Ok().json(ApiResponse::success(
                "URLs retrieved successfully",
                responses,
            )))
        }
        Err(err) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&err))),
    }
}

#[actix_web::delete("/urls/{id}/{user_id}")]
pub async fn delete_short_url(
    pool: web::Data<SqlitePool>,
    path: web::Path<(i64, String)>,
) -> Result<HttpResponse> {
    let (id, user_id) = path.into_inner();

    match UrlService::delete_short_url(&pool, id, &user_id).await {
        Ok(deleted) => {
            if deleted {
                Ok(HttpResponse::Ok().json(ApiResponse::success("URL deleted successfully", ())))
            } else {
                Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                    "URL not found or not owned by user",
                )))
            }
        }
        Err(err) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&err))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::pb::abi::ShortUrl;
    use actix_web::{http, test, App};
    use chrono::Utc;
    use serde_json::json;
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

    async fn insert_test_url(
        pool: &SqlitePool,
        id: i64,
        short_code: &str,
        long_url: &str,
        user_id: &str,
    ) {
        sqlx::query(
            r#"
            INSERT INTO short_urls (id, long_url, short_code, created_at, user_id)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(long_url)
        .bind(short_code)
        .bind(Utc::now().to_rfc3339())
        .bind(user_id)
        .execute(pool)
        .await
        .expect("Failed to insert test URL");
    }

    #[actix_web::test]
    async fn test_create_short_url_success() {
        // Setup
        let pool = setup_test_db().await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(create_short_url),
        )
        .await;

        // Create request with test data
        let req = test::TestRequest::post()
            .uri("/shorten")
            .set_json(json!({
                "long_url": "https://example.com",
                "user_id": "test_user"
            }))
            .to_request();

        // Send request
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::OK);

        // Parse response body
        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to parse response");

        assert_eq!(response["success"], true);
        assert_eq!(response["message"], "Short URL created successfully");
        assert!(response["data"]["short_url"].is_string());
        assert_eq!(response["data"]["long_url"], "https://example.com");
    }

    #[actix_web::test]
    async fn test_create_short_url_with_custom_code() {
        // Setup
        let pool = setup_test_db().await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(create_short_url),
        )
        .await;

        // Create request with custom code
        let req = test::TestRequest::post()
            .uri("/shorten")
            .set_json(json!({
                "long_url": "https://example.com",
                "custom_code": "customcode",
                "user_id": "test_user"
            }))
            .to_request();

        // Send request
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::OK);

        // Parse response body
        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to parse response");

        assert_eq!(response["data"]["short_code"], "customcode");
    }

    #[actix_web::test]
    async fn test_create_short_url_invalid_url() {
        // Setup
        let pool = setup_test_db().await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(create_short_url),
        )
        .await;

        // Create request with invalid URL - 使用一个确保无效的URL格式
        let req = test::TestRequest::post()
            .uri("/shorten")
            .set_json(json!({
                "long_url": "://invalid-url",
                "user_id": "test_user"
            }))
            .to_request();

        // Send request
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_get_short_urls() {
        // Setup
        let pool = setup_test_db().await;

        // Insert test URLs
        insert_test_url(&pool, 1, "code1", "https://example1.com", "test_user").await;
        insert_test_url(&pool, 2, "code2", "https://example2.com", "test_user").await;
        insert_test_url(&pool, 3, "code3", "https://example3.com", "other_user").await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(get_short_urls),
        )
        .await;

        // Send request
        let req = test::TestRequest::get().uri("/urls/test_user").to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::OK);

        // Parse response body
        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to parse response");

        assert_eq!(response["success"], true);
        assert_eq!(response["data"].as_array().unwrap().len(), 2);
    }

    #[actix_web::test]
    async fn test_delete_short_url_success() {
        // Setup
        let pool = setup_test_db().await;

        // Insert test URL
        insert_test_url(&pool, 1, "deleteme", "https://example.com", "test_user").await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(delete_short_url),
        )
        .await;

        // Send delete request
        let req = test::TestRequest::delete()
            .uri("/urls/1/test_user")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::OK);

        // Parse response body
        let body = test::read_body(resp).await;
        let response: serde_json::Value =
            serde_json::from_slice(&body).expect("Failed to parse response");

        assert_eq!(response["success"], true);

        // Verify URL is deleted
        let url = sqlx::query_as::<_, ShortUrl>("SELECT * FROM short_urls WHERE id = 1")
            .fetch_optional(&pool)
            .await
            .expect("Failed to query database");

        assert!(url.is_none());
    }

    #[actix_web::test]
    async fn test_delete_short_url_not_found() {
        // Setup
        let pool = setup_test_db().await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(delete_short_url),
        )
        .await;

        // Send delete request for non-existent URL
        let req = test::TestRequest::delete()
            .uri("/urls/999/test_user")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_delete_short_url_wrong_user() {
        // Setup
        let pool = setup_test_db().await;

        // Insert test URL
        insert_test_url(&pool, 1, "usertest", "https://example.com", "owner_user").await;

        // Create test app with the route
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(delete_short_url),
        )
        .await;

        // Send delete request with wrong user
        let req = test::TestRequest::delete()
            .uri("/urls/1/wrong_user")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Assert
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }
}
