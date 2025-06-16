use diesel::{Insertable, Queryable, Selectable};
use serde::{Serialize, Deserialize};

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::s3_files)]
pub struct NewS3File {
    pub bucket: String,
    pub region: String,
    pub s3_key: String,
    pub etag: Option<String>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::s3_files)]
pub struct S3File {
    pub file_id: i32,
    pub bucket: String,
    pub region: String,
    pub s3_key: String,
    pub etag: Option<String>,
}
