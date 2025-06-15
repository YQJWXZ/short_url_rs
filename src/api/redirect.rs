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
