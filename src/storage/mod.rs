use actix_web::{Error, HttpRequest};
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use futures_util::StreamExt;
use mime_guess::from_path;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Clone)]
pub struct FilesStorage {
    upload_dir: PathBuf,
}

impl FilesStorage {
    pub fn new(upload_dir: impl AsRef<Path>) -> Self {
        Self {
            upload_dir: upload_dir.as_ref().to_path_buf(),
        }
    }

    pub async fn save_file(
        &self,
        req: &HttpRequest,
        mut payload: actix_web::web::Payload,
    ) -> Result<(String, PathBuf, i64, Option<String>), Error> {
        // Ensure upload directory exists
        self.ensure_upload_dir_exists().map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!(
                "Failed to create upload directory: {}",
                e
            ))
        })?;

        // Get original filename from header
        let original_name = req
            .headers()
            .get("X-Filename")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("file");

        // Generate unique file ID and path with extension
        let file_id = Uuid::new_v4();
        let ext = Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let file_name = if ext.is_empty() {
            file_id.to_string()
        } else {
            format!("{}.{}", file_id, ext)
        };

        let file_path = self.upload_dir.join(&file_name);

        let mut file = std::fs::File::create(&file_path).map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to create file: {}", e))
        })?;

        let mut bytes: i64 = 0;

        // Read the payload stream and write to file while counting bytes
        while let Some(chunk) = payload.next().await {
            let chunk = chunk.map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Stream error: {}", e))
            })?;
            bytes += chunk.len() as i64;
            file.write_all(&chunk).map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Write error: {}", e))
            })?;
        }

        // Get MIME type from headers or guess from filename
        let mime_type_val = req
            .headers()
            .get(actix_web::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| from_path(original_name).first().map(|m| m.to_string()));

        Ok((original_name.to_string(), file_path, bytes, mime_type_val))
    }

    pub fn delete_file(&self, file_path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::remove_file(file_path)
    }

    pub fn ensure_upload_dir_exists(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.upload_dir)
    }
}

#[derive(Clone)]
pub struct S3Storage {
    client: Client,
    bucket_name: String,
}

impl S3Storage {
    pub fn new(client: Client, bucket_name: impl Into<String>) -> Self {
        Self {
            client,
            bucket_name: bucket_name.into(),
        }
    }

    pub async fn save_file(
        &self,
        req: &HttpRequest,
        mut payload: actix_web::web::Payload,
    ) -> Result<(String, String, i64, Option<String>), Error> {
        // Get original filename from header
        let original_name = req
            .headers()
            .get("X-Filename")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("file");

        // Generate unique file ID and key with extension
        let file_id = Uuid::new_v4();
        let ext = Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let key = if ext.is_empty() {
            format!("uploads/{}", file_id)
        } else {
            format!("uploads/{}.{}", file_id, ext)
        };

        let mut bytes: i64 = 0;
        let mut chunks = Vec::new();

        // Collect the payload stream into memory
        while let Some(chunk) = payload.next().await {
            let chunk = chunk.map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("Stream error: {}", e))
            })?;
            bytes += chunk.len() as i64;
            chunks.push(chunk);
        }

        // Combine chunks into a single ByteStream
        let body = ByteStream::from(chunks.concat());

        // Upload to S3
        let _ = self
            .client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .body(body)
            .send()
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("S3 upload error: {}", e))
            })?;

        // Get MIME type from headers or guess from filename
        let mime_type_val = req
            .headers()
            .get(actix_web::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| from_path(original_name).first().map(|m| m.to_string()));

        Ok((original_name.to_string(), key, bytes, mime_type_val))
    }

    pub async fn delete_file(&self, key: impl AsRef<str>) -> Result<(), Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key.as_ref())
            .send()
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("S3 delete error: {}", e))
            })?;
        Ok(())
    }

    pub async fn download_file(&self, key: impl AsRef<str>) -> Result<ByteStream, Error> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key.as_ref())
            .send()
            .await
            .map_err(|e| {
                actix_web::error::ErrorInternalServerError(format!("S3 download error: {}", e))
            })?;

        Ok(response.body)
    }
}
