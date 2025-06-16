use actix_web::{web, HttpResponse, Responder};
use entity::mail::{self, ActiveModel as MailActiveModel, Entity as Mail};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

async fn exists_in_db(email: &str, db: &DatabaseConnection) -> bool {
    match Mail::find()
        .filter(mail::Column::Email.contains(email))
        .order_by_asc(mail::Column::Email)
        .all(db)
        .await
    {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn add_email_to_mailing_list(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> impl Responder {
    let email = path.into_inner();

    if exists_in_db(&email, db.get_ref()).await {
        return HttpResponse::BadRequest().body("Email already exists");
    }

    let new_entry = MailActiveModel {
        email: Set(email.clone()),
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
