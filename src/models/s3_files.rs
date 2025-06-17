use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::s3_files)]
pub struct NewS3File {
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: NaiveDateTime,
    pub s3_key: String,
    pub etag: Option<String>,
    pub user_id: String,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::s3_files)]
pub struct S3File {
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub created_at: NaiveDateTime,
    pub file_id: i32,
    pub s3_key: String,
    pub etag: Option<String>,
    pub user_id: String,
}
