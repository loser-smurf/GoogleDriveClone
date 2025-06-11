use crate::repositories::files::{
    delete_file_by_id, find_file_by_id, find_files_by_name, get_file_metadata, insert_file,
    load_all_files,
};
use crate::storage::FilesStorage;
use crate::{database::DbPool, models::files::NewFile, requests::query::SearchQuery};
use actix_web::{Error, HttpResponse, web};
use mime_guess::from_path;

/// GET /api/files
/// Returns a list of all uploaded files with metadata.
pub async fn list_files(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let files_list = load_all_files(&pool)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB error: {}", e)))?;

    Ok(HttpResponse::Ok().json(files_list))
}

/// GET api/file/{id}/meta
/// Returns file metadata without storage path.
pub async fn get_metadata(
    pool: web::Data<DbPool>,
    file_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let (name, mime_type, size, created_at) = get_file_metadata(&pool, file_id.into_inner())
        .map_err(|e| match e {
            diesel::result::Error::NotFound => actix_web::error::ErrorNotFound("File not found"),
            _ => actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)),
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "name": name,
        "mime_type": mime_type,
        "size": size,
        "created_at": created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    })))
}

/// GET /api/files/search
/// Searches files by name with pagination
pub async fn search_files(
    pool: web::Data<DbPool>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, Error> {
    match find_files_by_name(&pool, &query.q) {
        Ok(files) => Ok(HttpResponse::Ok().json(files)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(format!("Database error: {}", e))),
    }
}

/// POST /api/files
/// Accepts file upload as stream, save to disk, stores metadata in DB.
pub async fn upload_file(
    pool: web::Data<DbPool>,
    storage: web::Data<FilesStorage>,
    payload: web::Payload,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    // Save the uploaded file using the storage service
    let (original_name, file_path, size, _mime_type_from_save, user_id_opt) =
        storage.save_file(&req, payload).await?;

    // Determine the MIME type based on the file extension
    let mime_type = from_path(&original_name)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Create a new file record with metadata
    let new_file = NewFile {
        name: original_name,
        storage_path: file_path.to_string_lossy().to_string(),
        size,
        mime_type: Some(mime_type),
        user_id: user_id_opt,
    };

    // Insert the file metadata into the database, handling possible errors
    insert_file(&pool, &new_file).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("DB insert error: {}", e))
    })?;

    // Return success response
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
