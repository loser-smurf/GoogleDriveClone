use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use diesel::{Insertable, Queryable};

#[derive(Serialize, Queryable)]
pub struct File {
    pub id: i32,
    pub name: String,
    pub storage_path: String,
    pub size: i64,
    pub mime_type: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::files)]
pub struct NewFile {
    pub name: String,
    pub storage_path: String,
    pub size: i64,
    pub mime_type: Option<String>,
}
