use actix_web::{Error, HttpResponse, web};
use diesel::prelude::*;
use futures_util::StreamExt;
use std::{io::Write, path::Path};
use uuid::Uuid;

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
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to load files"))?;

    Ok(HttpResponse::Ok().json(files_list))
}

/// POST /api/files
/// Accepts file upload as stream, save to disk, stores metadata in DB.
pub async fn upload_file(
    pool: web::Data<DbPool>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    use crate::schema::files::dsl::*;

    // Ensure upload directory exists
    std::fs::create_dir_all(UPLOAD_DIR)?;

    // Generate unique file ID and path
    let file_id = Uuid::new_v4();
    let file_name = format!("{}.data", file_id);
    let file_path = Path::new(UPLOAD_DIR).join(&file_name);

    let mut file = std::fs::File::create(&file_path)?;
    let mut bytes: i64 = 0;

    // Read the payload stream and write to file while counting bytes
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        bytes += chunk.len() as i64;
        file.write_all(&chunk)?;
    }

    // Create new file record
    let new_file = NewFile {
        name: file_id.to_string(),
        storage_path: file_path.to_string_lossy().to_string(),
        size: bytes,
        mime_type: None,
    };

    // Save file metadata to database
    let mut conn = pool
        .get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB pool error: {}", e)))?;
    diesel::insert_into(files)
        .values(&new_file)
        .execute(&mut conn)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Insert error: {}", e)))?;

    Ok(HttpResponse::Ok().json("File uploaded successfully"))
}

/// DELETE /api/files/{id}
/// Deltes a file from disk and its metadata from the database.
pub async fn delete_file(
    pool: web::Data<DbPool>,
    file_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    use crate::schema::files::dsl::*;

    let mut conn = pool
        .get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("DB pool error: {}", e)))?;

    let file = files
        .filter(id.eq(file_id.into_inner()))
        .first::<File>(&mut conn)
        .map_err(|_| actix_web::error::ErrorNotFound("File not found in database"))?;

    // Delete file from filesystem
    let file_path = Path::new(&file.storage_path);
    std::fs::remove_file(&file_path).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to delete file: {}", e))
    })?;

    // Delete record from database
    diesel::delete(files.filter(id.eq(file.id)))
        .execute(&mut conn)
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("DB delete error: {}", e))
        })?;
    Ok(HttpResponse::Ok().json("File deleted"))
}
