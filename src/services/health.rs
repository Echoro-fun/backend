use actix_web::{web, Responder, Result};
use serde::Serialize;

pub async fn health_check() -> Result<impl Responder> {
    Ok(web::Json(HealthResponse {
        message: "Echoro backend server running".to_string(),
        status: "OK".to_string(),
    }))
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub message: String,
    pub status: String,
}
