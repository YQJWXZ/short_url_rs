use crate::pb::abi::{CreateShortUrlRequest, ShortUrlResponse};
use crate::pb::ApiResponse;
use crate::services::UrlService;
use crate::utils::url_validator::normalize_url;
use actix_web::{web, HttpResponse, Result};
use sqlx::SqlitePool;

#[actix_web::post("/shorten")]
pub async fn create_short_url(
    pool: web::Data<SqlitePool>,
    request: web::Json<CreateShortUrlRequest>,
) -> Result<HttpResponse> {
    let mut req = request.into_inner();
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
