use crate::auth::jwt::AuthenticatedUser;
use crate::models::s3_files::NewS3File;
use crate::repositories::s3_files::load_all_s3_files;
use crate::repositories::s3_files::{
    delete_s3_file_by_id, find_s3_file_by_id, find_s3_files_by_key, insert_s3_file,
};
use crate::storage::S3Storage;
use crate::{database::DbPool, requests::query::SearchQuery};
use actix_web::http::header;
use actix_web::{Error, HttpResponse, web};
use log::{debug, error, info, warn};
use mime_guess::from_path;
use tokio_util::io::ReaderStream;

/// GET /api/files
/// Returns a list of all files stored in the database.
pub async fn list_files(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    info!("Fetching all files from the database");

    let s3_files = load_all_s3_files(&pool).map_err(|e| {
        error!("Database error while loading files: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(s3_files))
}

/// GET api/file/{id}/meta
/// Returns file metadata without storage path.
pub async fn get_metadata(
    pool: web::Data<DbPool>,
    file_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    info!("Fetching metadata for file_id: {}", file_id);

    let file_id = file_id.into_inner();

    // Get metadata from S3
    let s3_file = find_s3_file_by_id(&pool, file_id).map_err(|e| {
        warn!("S3 file not found for metadata: {}", e);
        actix_web::error::ErrorNotFound(format!("S3 file not found: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "name": s3_file.name,
        "mime_type": s3_file.mime_type,
        "size": s3_file.size,
        "created_at": s3_file.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    })))
}

/// GET /api/files/search
/// Searches files by name with pagination
pub async fn search_files(
    pool: web::Data<DbPool>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, Error> {
    info!("Searching files by name {}:", query.q);

    // Get files from S3
    let s3_files = find_s3_files_by_key(&pool, &query.q).map_err(|e| {
        warn!("S3 files not found for search: {}", e);
        actix_web::error::ErrorNotFound(format!("S3 files not found: {}", e))
    })?;

    Ok(HttpResponse::Ok().json(s3_files))
}

/// POST /api/files
/// Accepts file upload as stream, saves to S3, stores metadata in DB.
pub async fn upload_file(
    pool: web::Data<DbPool>,
    storage_s3: web::Data<S3Storage>,
    user: AuthenticatedUser,
    req: actix_web::HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse, Error> {
    info!("User: {} is uploading a file", user.user_id);

    // Get the original filename from the request headers
    let original_name = req.headers().get("X-Filename").unwrap().to_str().unwrap();

    // Determine the MIME type based on the file extension
    let mime_type = from_path(original_name)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Save the file to S3
    let (original_name, s3_key, size, mime_type_from_save) =
        storage_s3.save_file(&req, payload).await?;

    // Create a new S3 file record with metadata
    let new_s3_file = NewS3File {
        name: original_name.clone(),
        mime_type: mime_type_from_save.unwrap_or_else(|| mime_type),
        size,
        created_at: chrono::Utc::now().naive_utc(),
        s3_key,
        etag: None, // S3 etag will be set after upload
        user_id: user.user_id,
    };

    // Insert the file metadata into the database
    insert_s3_file(&pool, &new_s3_file).map_err(|e| {
        error!("Failed to insert S3 file metadata: {}", e);
        actix_web::error::ErrorInternalServerError(format!("DB insert error: {}", e))
    })?;

    info!("Inserted file '{}' into DB", original_name);

    // Return success response
    Ok(HttpResponse::Ok().json("File uploaded successfully"))
}

/// DELETE /api/files/{id}
/// Deletes a file from S3 and its metadata from the database.
pub async fn delete_file(
    pool: web::Data<DbPool>,
    storage_s3: web::Data<S3Storage>,
    file_id: web::Path<i32>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, Error> {
    info!(
        "User {} attempts to delete file ID {}",
        user.user_id, file_id
    );

    let file = find_s3_file_by_id(&pool, file_id.into_inner()).map_err(|e| {
        warn!("File not found for deletion: {}", e);
        actix_web::error::ErrorNotFound(format!("File not found: {}", e))
    })?;

    // Check that the file belongs to the authenticated user
    if file.user_id != user.user_id {
        warn!(
            "User {} tried to delete someone else's file (ID: {})",
            user.user_id, file.file_id
        );
        return Err(actix_web::error::ErrorForbidden("You do not own this file"));
    }

    debug!("Deleting file from S3: {}", file.s3_key);

    // Delete file from S3
    storage_s3.delete_file(&file.s3_key).await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to delete file from S3: {}", e))
    })?;

    // Delete record from database
    delete_s3_file_by_id(&pool, file.file_id).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("DB delete error: {}", e))
    })?;

    Ok(HttpResponse::Ok().json("File deleted successfully"))
}

/// GET /api/files/{id}
/// Downloads a file by its ID with proper headers.
pub async fn download_file(
    pool: web::Data<DbPool>,
    storage_s3: web::Data<S3Storage>,
    file_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    info!("Downloading file with ID: {}", file_id);

    let file = find_s3_file_by_id(&pool, file_id.into_inner()).map_err(|e| {
        warn!("File not found for download: {}", e);
        actix_web::error::ErrorNotFound(format!("File not found: {}", e))
    })?;

    debug!("Downloading file from S3 with key: {}", file.s3_key);
    let byte_stream = storage_s3.download_file(&file.s3_key).await?;

    // Convert ByteStream to a compatible Stream type
    let stream = ReaderStream::new(byte_stream.into_async_read());

    Ok(HttpResponse::Ok()
        .append_header((
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.name),
        ))
        .append_header((header::CONTENT_TYPE, file.mime_type))
        .streaming(stream))
}
