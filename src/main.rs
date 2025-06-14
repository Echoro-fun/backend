use actix_web::{web, App, HttpServer, Responder};
use dotenv::dotenv;
use std::env;
use std::io::Result;

#[actix_web::get("/health")]
async fn health_check() -> impl Responder {
    format!("Echoro backend server running...")
}

#[actix_web::get("/api/mail/{email}")]
async fn add_email_to_mailing_list(email: web::Path<String>) -> impl Responder {}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    println!("Running server at http://{}:{}", host, port);

    HttpServer::new(|| App::new().service(health_check))
        .bind(format!("{}:{}", host, port))?
        .workers(2)
        .run()
        .await
}
