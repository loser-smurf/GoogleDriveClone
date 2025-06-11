use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::Serialize;

#[derive(Serialize, Queryable)]
pub struct File {
    pub id: i32,
    pub name: String,
    pub storage_path: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub created_at: NaiveDateTime,
    pub user_id: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::files)]
pub struct NewFile {
    pub name: String,
    pub storage_path: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub user_id: Option<i32>,
}
