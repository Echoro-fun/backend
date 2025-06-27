use actix_multipart::{form::{MultipartForm, tempfile::TempFile}, Multipart};
use actix_web::{error, http::StatusCode, web, HttpResponse, Responder, Result};
use derive_more::derive::{Display, Error as CustomError};
use entity::upload::{self, ActiveModel as UploadActiveModel, Entity as Upload};
use futures_util::StreamExt as _;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Iden, QueryFilter, QueryOrder,
    Set,
};
use serde::Serialize;
use std::{fs::{create_dir_all, File}, io::Write};

pub async fn upload_audio_file(
    db: web::Data<DatabaseConnection>,
    mut payload: Multipart,
) -> Result<impl Responder, UploadError> {
    create_dir_all("./data").map_err(|e| UploadError {
        message: format!("Failed to create data directory: {}", e),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    let mut file_path = None;

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        if field.name().map(|n| n == "audioFile").unwrap_or(false) {
            let Some(content_disposition) = field.content_disposition() else {
                continue;
            };
            let filename = content_disposition.get_filename().unwrap();
            if !filename.ends_with(".mp3")
                || field
                    .content_type()
                    .map(|ct| ct.to_string() != "audio/mpeg")
                    .unwrap_or(true)
            {
                return Err(UploadError {
                    message: "Only MP3 format supported".to_string(),
                    status_code: StatusCode::CONFLICT,
                });
            }
            let filepath = format!("./data/{}", filename);
            log::info!("Uploading audio file: {}", filepath);
            let filepath_clone = filepath.clone();
            let mut file = web::block(move || File::create(&filepath_clone).unwrap()).await
                .map_err(|e| UploadError {
                    message: format!("Failed to block: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })
                .map_err(|e| UploadError {
                    message: format!("Failed to create file: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;
            while let Some(chunk) = field.next().await {
                file.write_all(&chunk.unwrap()).unwrap();
            }
            file_path = Some(filepath);
        }
    }

    let file_path = file_path.ok_or_else(|| UploadError {
        message: "No valid MP3 file uploaded".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    Ok(web::Json(UploadResponse {
        message: "File uploaded successfully.".to_string(),
        data: Some(file_path),
    }))
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "audio_file")]
    audio_file: Vec<TempFile>,
}

#[derive(Debug, Display, CustomError)]
#[display("{message}")]
pub struct UploadError {
    pub message: String,
    pub status_code: StatusCode,
}

impl error::ResponseError for UploadError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code).json(UploadResponse::<()> {
            message: self.message.clone(),
            data: None,
        })
    }
}

#[derive(Serialize)]
pub struct UploadResponse<T> {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Serialize)]
pub struct UploadData {
    uri: String,
}
