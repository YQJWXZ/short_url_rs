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
