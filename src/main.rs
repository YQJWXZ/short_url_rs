use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use short_url_rs::{api::*, db};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Initialize database
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:short_url.db".to_string());

    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    println!("Starting server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .service(
                web::scope("/api")
                    .service(shorten::create_short_url)
                    .service(shorten::get_short_urls)
                    .service(shorten::delete_short_url)
                    .service(qrcode::redirect_qrcode),
            )
            .service(redirect::redirect_to_long_url)
            .route(
                "/",
                web::get()
                    .to(|| async { HttpResponse::Ok().body("Short URL Service is running!") }),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
