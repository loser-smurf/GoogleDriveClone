use actix_web::{Error, HttpResponse, web};
use diesel::prelude::*;
use futures_util::StreamExt;
use std::{io::Write, path::Path};
use uuid::Uuid;
use mime_guess::from_path;

use crate::{
    database::DbPool,
    models::{File, NewFile},
};

const UPLOAD_DIR: &str = "./uploads/";

/// GET /api/files
/// Returns a list of all uploaded files with metadata.
pub async fn list_files(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    use crate::schema::files::dsl::*;

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to get DB connection: {}", e))
    })?;

    let files_list = files
        .load::<File>(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to load files: {}", e)))?;

    Ok(HttpResponse::Ok().json(files_list))
}

/// POST /api/files
/// Accepts file upload as stream, save to disk, stores metadata in DB.
pub async fn upload_file(
    pool: web::Data<DbPool>,
    mut payload: web::Payload,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    use crate::schema::files::dsl::*;

    // Ensure upload directory exists
    std::fs::create_dir_all(UPLOAD_DIR).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to create upload directory: {}", e))
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
    
    let file_path = Path::new(UPLOAD_DIR).join(&file_name);

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

    // Create new file record
    let new_file = NewFile {
        name: original_name.to_string(),
        storage_path: file_path.to_string_lossy().to_string(),
        size: bytes,
        mime_type: mime_type_val, 
    };


    // Save file metadata to database
    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("DB pool error: {}", e))
    })?;
    
    diesel::insert_into(files)
        .values(&new_file)
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Insert error: {}", e)))?;

    Ok(HttpResponse::Ok().json("File uploaded successfully"))
}

/// DELETE /api/files/{id}
/// Deletes a file from disk and its metadata from the database.
pub async fn delete_file(
    pool: web::Data<DbPool>,
    file_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    use crate::schema::files::dsl::*;

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("DB pool error: {}", e))
    })?;

    let file = files
        .filter(id.eq(file_id.into_inner()))
        .first::<File>(&mut conn)
        .map_err(|e| actix_web::error::ErrorNotFound(format!("File not found: {}", e)))?;

    // Delete file from filesystem
    std::fs::remove_file(&file.storage_path).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to delete file: {}", e))
    })?;

    // Delete record from database
    diesel::delete(files.filter(id.eq(file.id)))
        .execute(&mut conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("DB delete error: {}", e))
        })?;

    Ok(HttpResponse::Ok().json("File deleted successfully"))
}

/// GET /api/files/{id}
/// Downloads a file by its ID with proper headers.
pub async fn download_file(
    pool: web::Data<DbPool>,
    file_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    use crate::schema::files::dsl::*;
    use actix_web::http::header;

    let mut conn = pool.get().map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database connection error: {}", e))
    })?;

    let file = files
        .filter(id.eq(file_id.into_inner()))
        .first::<File>(&mut conn)
        .map_err(|e| actix_web::error::ErrorNotFound(format!("File not found: {}", e)))?;

    let tokio_file = tokio::fs::File::open(&file.storage_path).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to open file: {}", e))
    })?;

    let stream = tokio_util::io::ReaderStream::new(tokio_file);
    let content_type = file.mime_type.as_deref().unwrap_or("application/octet-stream");

    Ok(HttpResponse::Ok()
        .append_header((header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file.name)))
        .append_header((header::CONTENT_TYPE, content_type))
        .streaming(stream))
}
