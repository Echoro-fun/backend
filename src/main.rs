pub mod services;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use sea_orm::Database;
pub use services::{
    health::health_check, mail::add_email_to_mailing_list, upload::upload_audio_file,
};
use std::env;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&db_url)
        .await
        .expect("DB connection failed");

    log::info!("Running server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(db.clone()))
            .route("/health", web::get().to(health_check))
            .route(
                "/api/mail/{email}",
                web::post().to(add_email_to_mailing_list),
            )
            .route("/api/upload", web::post().to(upload_audio_file))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
