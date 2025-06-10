use crate::database::{DbPool, get_db_conn};
use crate::models::{File, NewFile};
use crate::schema::files::dsl::*;
use diesel::prelude::*;

/// Inserts a new file record and returns the created record
pub fn insert_file(pool: &DbPool, new: &NewFile) -> Result<File, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    diesel::insert_into(files).values(new).get_result(&mut conn)
}

/// Loads all file records from the database
pub fn load_all_files(pool: &DbPool) -> Result<Vec<File>, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    files.load::<File>(&mut conn)
}

/// Finds a file record by its ID.
pub fn find_file_by_id(pool: &DbPool, file_id_val: i32) -> Result<File, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    files.filter(id.eq(file_id_val)).first::<File>(&mut conn)
}

/// Deletes a file record from the 'files' table by its ID.
pub fn delete_file_by_id(pool: &DbPool, file_id_val: i32) -> Result<usize, diesel::result::Error> {
    let mut conn = get_db_conn(pool)?;

    diesel::delete(files.filter(id.eq(file_id_val))).execute(&mut conn)
}
