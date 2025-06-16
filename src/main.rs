pub mod services;

use actix_web::{web, App, HttpServer};
use sea_orm::Database;
use dotenv::dotenv;
use std::env;
use std::io::Result;

pub use services::{health::health_check, mail::add_email_to_mailing_list};

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&db_url)
        .await
        .expect("DB connection failed");

    println!("Running server at http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .route("/health", web::get().to(health_check))
            .route("/api/mail/{email}", web::post().to(add_email_to_mailing_list))
    })
    .bind(format!("{}:{}", host, port))?
    .workers(2)
    .run()
    .await
}
