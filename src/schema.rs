// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        oauth_provider -> Varchar,
        oauth_user_id -> Varchar,
        email -> Nullable<Varchar>,
        username -> Nullable<Varchar>,
        avatar_url -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    s3_files (file_id) {
        name -> Varchar,
        mime_type -> Varchar,
        size -> Int8,
        created_at -> Timestamp,
        file_id -> Int4,
        s3_key -> Varchar,
        etag -> Nullable<Varchar>,
        user_id -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(users, s3_files,);
