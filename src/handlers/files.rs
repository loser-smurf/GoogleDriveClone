use crate::auth::jwt::AuthenticatedUser;
use crate::repositories::files::{
    delete_file_by_id, find_file_by_id, find_files_by_name, get_file_metadata, insert_file,
    load_all_files,
};
use crate::storage::FilesStorage;
use crate::{database::DbPool, models::files::NewFile, requests::query::SearchQuery};
use actix_web::{Error, HttpResponse, web};
use log::{debug, error, info, warn};
use mime_guess::from_path;

/// GET /api/files
/// Returns a list of all uploaded files with metadata.
pub async fn list_files(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    info!("Fetching all uploaded files");
    let files_list = load_all_files(&pool).map_err(|e| {
        error!("DB error while loading files: {}", e);
        actix_web::error::ErrorInternalServerError(format!("DB error: {}", e))
    })?;
    Ok(HttpResponse::Ok().json(files_list))
}

/// GET api/file/{id}/meta
/// Returns file metadata without storage path.
pub async fn get_metadata(
    pool: web::Data<DbPool>,
    file_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    info!("Fetching metadata for file_id: {}", file_id);

    let file_id = file_id.into_inner();

    let (name, mime_type, size, created_at) =
        get_file_metadata(&pool, file_id).map_err(|e| match e {
            diesel::result::Error::NotFound => {
                warn!("Metadata not found for file_id {}", file_id);
                actix_web::error::ErrorNotFound("File not found")
            }
            _ => {
                error!("Error fetching metadata: {}", e);
                actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
            }
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
    info!("Searching files by name {}:", query.q);
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
    user: AuthenticatedUser,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, Error> {
    info!("User: {} is uploading a file", user.user_id);

    // Save the uploaded file using the storage service
    let (original_name, file_path, size, _mime_type_from_save) =
        storage.save_file(&req, payload).await?;
    let file_name_log = original_name.clone();

    debug!("Saved file: {}, size: {}", file_name_log, size);

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
        user_id: user.user_id,
    };

    // Insert the file metadata into the database, handling possible errors
    insert_file(&pool, &new_file).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("DB insert error: {}", e))
    })?;

    info!("Inserted file '{}' into DB", file_name_log);

    // Return success response
    Ok(HttpResponse::Ok().json("File uploaded successfully"))
}

/// DELETE /api/files/{id}
/// Deletes a file from disk and its metadata from the database.
pub async fn delete_file(
    pool: web::Data<DbPool>,
    storage: web::Data<FilesStorage>,
    file_id: web::Path<i32>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, Error> {
    info!(
        "User {} attempts to delete file ID {}",
        user.user_id, file_id
    );
    let file = find_file_by_id(&pool, file_id.into_inner()).map_err(|e| {
        warn!("File not found for deletion: {}", e);
        actix_web::error::ErrorNotFound(format!("File not found: {}", e))
    })?;

    // Check that the file belongs to the authenticated user

    if file.user_id.as_ref() != user.user_id {
        warn!(
            "User {} tried to delete someone else's file (ID: {})",
            user.user_id, file.id
        );
        return Err(actix_web::error::ErrorForbidden("You do not own this file"));
    }

    debug!("Deleting file from storage: {}", file.storage_path);
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
    info!("Downloading file with ID: {}", file_id);
    let file = find_file_by_id(&pool, file_id.into_inner()).map_err(|e| {
        warn!("File not found for download: {}", e);
        actix_web::error::ErrorNotFound(format!("File not found: {}", e))
    })?;

    debug!("Opening file from path: {}", file.storage_path);
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
