use actix_web::{Error, HttpResponse, web};
use crate::repo::files::{delete_file_by_id, find_file_by_id, insert_file, load_all_files};
use crate::storage::FilesStorage;
use crate::{database::DbPool, models::NewFile};

/// GET /api/files
/// Returns a list of all uploaded files with metadata.
pub async fn list_files(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let files_list = load_all_files(&pool)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {}", e)))?;

    Ok(HttpResponse::Ok().json(files_list))
}

/// POST /api/files
/// Accepts file upload as stream, save to disk, stores metadata in DB.
pub async fn upload_file(
    pool: web::Data<DbPool>,
    storage: web::Data<FilesStorage>,
    payload: web::Payload,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    let (original_name, file_path, size, mime_type) = storage.save_file(&req, payload).await?;

    // Create new file record
    let new_file = NewFile {
        name: original_name,
        storage_path: file_path.to_string_lossy().to_string(),
        size,
        mime_type,
    };

    // Save file metadata to database
    insert_file(&pool, &new_file).unwrap();

    Ok(HttpResponse::Ok().json("File uploaded successfully"))
}

/// DELETE /api/files/{id}
/// Deletes a file from disk and its metadata from the database.
pub async fn delete_file(
    pool: web::Data<DbPool>,
    storage: web::Data<FilesStorage>,
    file_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let file = find_file_by_id(&pool, file_id.into_inner())
        .map_err(|e| actix_web::error::ErrorNotFound(format!("File not found: {}", e)))?;

    // Delete file from filesystem
    storage.delete_file(&file.storage_path).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to delete file: {}", e))
    })?;
    
    // Delete record from database
    delete_file_by_id(&pool, file.id).map_err(|e| {
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
    use actix_web::http::header;

    let file = find_file_by_id(&pool, file_id.into_inner())
        .map_err(|e| actix_web::error::ErrorNotFound(format!("File not found: {}", e)))?;

  
    let tokio_file = tokio::fs::File::open(&file.storage_path)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Failed to open file: {}", e))
        })?;

    let stream = tokio_util::io::ReaderStream::new(tokio_file);
    let content_type = file
        .mime_type
        .as_deref()
        .unwrap_or("application/octet-stream");

    Ok(HttpResponse::Ok()
        .append_header((
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.name),
        ))
        .append_header((header::CONTENT_TYPE, content_type))
        .streaming(stream))
}
