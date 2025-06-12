use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::Serialize;

#[derive(Debug, Queryable, Serialize)]
pub struct User {
    pub id: i32,
    pub oauth_provider: String,
    pub oauth_user_id: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub oauth_provider: String,
    pub oauth_user_id: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}
