use actix_web::HttpMessage;
use actix_web::{Error, HttpRequest};
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
    ) -> Result<(String, PathBuf, i64, Option<String>, Option<i32>), Error> {
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

        // Extract user_id from request extensions, if available
        let user_id = req.extensions().get::<i32>().copied();

        Ok((
            original_name.to_string(),
            file_path,
            bytes,
            mime_type_val,
            user_id,
        ))
    }

    pub fn delete_file(&self, file_path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::remove_file(file_path)
    }

    pub fn ensure_upload_dir_exists(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.upload_dir)
    }
}
