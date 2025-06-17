use crate::database::{DbPool, get_db_conn};
use crate::models::s3_files::{S3File, NewS3File};
use crate::schema::s3_files::dsl::*;
use diesel::prelude::*;

/// Inserts a new S3 file record and returns the created record
pub fn insert_s3_file(pool: &DbPool, new: &NewS3File) -> Result<S3File, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    diesel::insert_into(s3_files).values(new).get_result(&mut conn)
}

/// Loads all S3 file records from the database
pub fn load_all_s3_files(pool: &DbPool) -> Result<Vec<S3File>, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    s3_files.load::<S3File>(&mut conn)
}

/// Finds an S3 file record by its ID.
pub fn find_s3_file_by_id(pool: &DbPool, file_id_val: i32) -> Result<S3File, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    s3_files.filter(file_id.eq(file_id_val)).first::<S3File>(&mut conn)
}

/// Deletes an S3 file record from the 's3_files' table by its ID.
pub fn delete_s3_file_by_id(pool: &DbPool, file_id_val: i32) -> Result<usize, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    diesel::delete(s3_files.filter(file_id.eq(file_id_val))).execute(&mut conn)
}

/// Finds S3 files by partial match on s3_key (like a file path)
pub fn find_s3_files_by_key(pool: &DbPool, search_query: &str) -> Result<Vec<S3File>, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    s3_files
        .filter(s3_key.ilike(format!("%{}%", search_query)))
        .load::<S3File>(&mut conn)
}

/// Gets basic S3 file metadata (no path), e.g. etag and user_id
pub fn get_s3_file_metadata(
    pool: &DbPool,
    file_id_val: i32,
) -> Result<(Option<String>, String), diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    s3_files
        .filter(file_id.eq(file_id_val))
        .select((etag, user_id))
        .first::<(Option<String>, String)>(&mut conn)
}