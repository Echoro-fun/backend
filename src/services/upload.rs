use actix_multipart::Multipart;
use actix_web::{error, http::StatusCode, web, HttpResponse, Responder, Result};
use derive_more::derive::{Display, Error as CustomError};
use entity::upload::{ActiveModel as UploadActiveModel};
use futures_util::StreamExt as _;
use reqwest::Client;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::signer::{keypair::Keypair, Signer};
use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
};

/// Service to handles token data upload
pub async fn upload_token_data(
    db: web::Data<DatabaseConnection>,
    mut payload: Multipart,
) -> Result<impl Responder, UploadError> {
    fs::create_dir_all("./data").map_err(|e| UploadError {
        message: format!("Failed to create data directory: {}", e),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    // todo: check if these are being set correctly
    let mut audio_track_path = None;
    let mut image_path = None;
    let mut token_data = None;

    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        if field.name().map(|n| n == "audioFile").unwrap_or(false) {
            let content_disposition = field.content_disposition().ok_or_else(|| UploadError {
                message: "Missing content disposition for audioFile".to_string(),
                status_code: StatusCode::BAD_REQUEST,
            })?;
            let filename = content_disposition
                .get_filename()
                .ok_or_else(|| UploadError {
                    message: "Invalid filename for audioFile".to_string(),
                    status_code: StatusCode::BAD_REQUEST,
                })?;
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

            log::info!("Uploading audio file started: {filepath}");

            let filepath_clone = filepath.clone();
            let mut file = web::block(move || File::create(&filepath_clone).unwrap())
                .await
                .map_err(|e| UploadError {
                    message: format!("Failed to block while creating audio file: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })
                .map_err(|e| UploadError {
                    message: format!("Failed to create audio file: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|e| UploadError {
                    message: format!("Failed to read audio file chunk: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;
                file.write_all(&chunk).map_err(|e| UploadError {
                    message: format!("Failed to write audio file chunk: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;
            }
            audio_track_path = Some(filepath);
        } else if field.name().map(|n| n == "imageFile").unwrap_or(false) {
            let Some(content_disposition) = field.content_disposition() else {
                continue;
            };
            let filename = content_disposition.get_filename().unwrap();
            if !filename.ends_with(".png")
                || field
                    .content_type()
                    .map(|ct| ct.to_string() != "image/png")
                    .unwrap_or(true)
            {
                return Err(UploadError {
                    message: "Only PNG format supported".to_string(),
                    status_code: StatusCode::CONFLICT,
                });
            }
            let filepath = format!("./data/{}", filename);

            log::info!("Uploading image file started: {filepath}");

            let filepath_clone = filepath.clone();
            let mut file = web::block(move || File::create(&filepath_clone).unwrap())
                .await
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
            image_path = Some(filepath);
        } else if field.name().map(|n| n == "tokenData").unwrap_or(false) {
            let data = field
                .next()
                .await
                .ok_or_else(|| UploadError {
                    message: "Missing token data".to_string(),
                    status_code: StatusCode::BAD_REQUEST,
                })?
                .map_err(|e| UploadError {
                    message: format!("Failed to read token data: {}", e),
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                })?;

            token_data =
                Some(
                    serde_json::from_slice::<TokenData>(&data).map_err(|e| UploadError {
                        message: format!("Failed to parse token data: {}", e),
                        status_code: StatusCode::BAD_REQUEST,
                    })?,
                );

            log::info!(
                "Uploading token data started: {{ name: {}, symbol: {}, decimal: {} }}",
                token_data.clone().unwrap().name,
                token_data.clone().unwrap().symbol,
                token_data.clone().unwrap().decimal
            );
        }
    }

    let audio_track_path = audio_track_path.ok_or_else(|| UploadError {
        message: "No valid MP3 file provided".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    // todo: try to get the cover image from the uploaded audio track
    // let image_path = image_path
    //     .ok_or_else(|| get_image_from_audio_track())
    //     .map_err(|e| UploadError {
    //         message: format!("Failed to get image from audio track: {}", e),
    //         status_code: StatusCode::INTERNAL_SERVER_ERROR,
    //     })?;
    let image_path = image_path.ok_or_else(|| UploadError {
        message: "No valid PNG file provided".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    let token_data = token_data.ok_or_else(|| UploadError {
        message: "No valid token data provided".to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })?;

    let payer_secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    let keypair = Keypair::from_base58_string(&payer_secret);

    let audio_uri = upload_file(&audio_track_path, &keypair, FileType::AudioFile)
        .await
        .map_err(|e| UploadError {
            message: format!("Failed to upload audio file: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    log::info!("Uploaded audio file: {}", audio_uri);

    let image_uri = upload_file(&image_path, &keypair, FileType::ImageFile)
        .await
        .map_err(|e| UploadError {
            message: format!("Failed to upload image file: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    log::info!("Uploaded image file: {}", image_uri);

    let metadata_uri = get_metadata(&image_uri, &token_data)
        .await
        .map_err(|e| UploadError {
            message: format!("Failed to get metadata: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    log::info!("Metadata fetched: {}", metadata_uri);

    fs::remove_file(&audio_track_path).map_err(|e| UploadError {
        message: format!("Failed to clean up file: {}", e),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    fs::remove_file(&image_path).map_err(|e| UploadError {
        message: format!("Failed to clean up image file: {}", e),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    let new_entry = UploadActiveModel {
        name: Set(token_data.clone().name),
        symbol: Set(token_data.clone().symbol),
        decimal: Set(token_data.clone().decimal),
        audio_uri: Set(audio_uri.clone()),
        image_uri: Set(image_uri.clone()),
        metadata_uri: Set(metadata_uri.clone()),
        ..Default::default()
    };

    match new_entry.insert(db.get_ref()).await {
        Ok(_) => Ok(web::Json(UploadResponse {
            message: "Token data uploaded successfully!".to_string(),
            data: Some(UploadData {
                token_data: Some(token_data),
                audio_uri,
                image_uri,
                metadata_uri,
            }),
        })),
        Err(err) => {
            eprintln!("Insert error: {}", err);
            Err(UploadError {
                message: "Failed to add token data to uploads_table".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            })
        }
    }
}

enum FileType {
    AudioFile,
    ImageFile,
}

/// Utility function to upload the audio file to Arweave
async fn upload_file(
    file_path: &str,
    keypair: &Keypair,
    file_type: FileType,
) -> Result<String, UploadError> {
    // todo: use filetype to set content type and tags
    let client = Client::new();
    let content_type = match file_type {
        FileType::AudioFile => "audio/mpeg",
        FileType::ImageFile => "image/png",
    };
    let mut audio_file = File::open(file_path).map_err(|e| UploadError {
        message: format!("Failed to open audio file: {}", e),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;
    let mut buffer = Vec::new();
    audio_file
        .read_to_end(&mut buffer)
        .map_err(|e| UploadError {
            message: format!("Failed to read audio file: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    let data_size = buffer.len();
    let data = base64::encode(&buffer);
    let reward = estimate_arweave_fee(data_size).await?;

    // Sign transaction (simplified, using Solana keypair for demo)
    let message = format!("{}{}", data, reward.to_string());
    let signature = keypair.sign_message(message.as_bytes());
    let transaction = serde_json::json!({
        "id": "", // Will be set by Arweave node
        "last_tx": "", // Optional for now
        "owner": keypair.pubkey().to_string(),
        "tags": [
            {"name": "Content-Type", "value": content_type},
        ],
        "target": "",
        "quantity": "0",
        "data": base64::encode(&buffer),
        "reward": reward.to_string(),
        "signature": base64::encode(&buffer) // Will be set after signing
    });

    // Post transaction to Arweave
    let response = client
        .post("https://arweave.net/tx")
        .json(&transaction)
        .send()
        .await
        .map_err(|e| UploadError {
            message: format!("Failed to upload to Arweave: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .json::<Value>()
        .await
        .map_err(|e| UploadError {
            message: format!("Failed to parse Arweave response: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    let id = response["id"].as_str().ok_or_else(|| UploadError {
        message: "No transaction ID in Arweave response".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(format!("https://arweave.net/{}", id))
}

// Utility function to extract the cover image from an audio file if there is one associated with the audio file
async fn get_image_from_audio_track(audio_filepath: &str) -> Result<String, UploadError> {
    todo!()
}

// Utility function to bootstrap the token and get the metadata
async fn get_metadata(image_uri: &str, token_data: &TokenData) -> Result<String, UploadError> {
    todo!()
}

// Utility function to estimate Arweave fee
async fn estimate_arweave_fee(data_size: usize) -> Result<u64, UploadError> {
    let client = Client::new();
    let response = client
        .get("https://arweave.net/price/".to_string() + &data_size.to_string())
        .send()
        .await
        .map_err(|e| UploadError {
            message: format!("Failed to fetch Arweave price: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    let price: u64 = response
        .text()
        .await
        .map_err(|e| UploadError {
            message: format!("Failed to parse Arweave price: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .parse()
        .map_err(|e| UploadError {
            message: format!("Invalid Arweave price format: {}", e),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(price)
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
pub struct UploadResponse<UploadData> {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<UploadData>,
}

#[derive(Serialize)]
pub struct UploadData<TokenData> {
    token_data: Option<TokenData>,
    audio_uri: String,
    image_uri: String,
    metadata_uri: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenData {
    name: String,
    symbol: String,
    decimal: u8,
}
