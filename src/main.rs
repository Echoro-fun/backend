use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use std::env;
use std::io::Result;

use entity::mail::ActiveModel as MailActiveModel;

async fn health_check() -> impl Responder {
    format!("Echoro backend server running...")
}

async fn add_email_to_mailing_list(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> impl Responder {
    let email = path.into_inner();
    println!("Adding email to mailing list: {}", email);

    let new_entry = MailActiveModel {
        email: Set(email),
        ..Default::default()
    };

    match new_entry.insert(db.get_ref()).await {
        Ok(_) => HttpResponse::Ok().body("Email saved successfully"),
        Err(err) => {
            eprintln!("Insert error: {}", err);
            HttpResponse::InternalServerError().body("Failed to save email")
        }
    }
}

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
            .route("/api/mail/{email}", web::get().to(add_email_to_mailing_list))
    })
    .bind(format!("{}:{}", host, port))?
    .workers(2)
    .run()
    .await
}
