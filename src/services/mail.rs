use actix_web::{error, http::StatusCode, web, HttpResponse, Responder, Result};
use derive_more::derive::{Display, Error as CustomError};
use entity::mail::{self, ActiveModel as MailActiveModel, Entity as Mail};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Serialize;

async fn exists_in_db(email: &str, db: &DatabaseConnection) -> bool {
    match Mail::find()
        .filter(mail::Column::Email.contains(email))
        .order_by_asc(mail::Column::Email)
        .one(db)
        .await
    {
        Ok(Some(_)) => true,
        Ok(_) => false,
        Err(err) => {
            eprintln!("Database query error: {}", err);
            false
        }
    }
}

pub async fn add_email_to_mailing_list(
    db: web::Data<DatabaseConnection>,
    path: web::Path<String>,
) -> Result<impl Responder, MailError> {
    let email = path.into_inner();

    if exists_in_db(&email, db.get_ref()).await {
        return Err(MailError {
            message: "Email already exists in the mailing list".to_string(),
            status_code: StatusCode::CONFLICT,
        });
    }

    let new_entry = MailActiveModel {
        email: Set(email.clone()),
        ..Default::default()
    };

    match new_entry.insert(db.get_ref()).await {
        Ok(_) => Ok(web::Json(MailResponse {
            message: "Email successfully added to the mailing list".to_string(),
            data: Some(EmailData { email }),
        })),
        Err(err) => {
            eprintln!("Insert error: {}", err);
            Err(MailError {
                message: "Failed to add email to the mailing list".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

#[derive(Debug, Display, CustomError)]
#[display("{message}")]
pub struct MailError {
    pub message: String,
    pub status_code: StatusCode,
}

impl error::ResponseError for MailError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code).json(MailResponse::<()> {
            message: self.message.clone(),
            data: None,
        })
    }
}

#[derive(Serialize)]
pub struct MailResponse<T> {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Serialize)]
pub struct EmailData {
    email: String,
}
