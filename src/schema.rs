// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        storage_path -> Varchar,
        size -> Int8,
        #[max_length = 100]
        mime_type -> Nullable<Varchar>,
        created_at -> Timestamp,
        user_id -> Varchar,
    }
}

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
        file_id -> Int4,
        #[max_length = 255]
        bucket -> Varchar,
        #[max_length = 255]
        region -> Varchar,
        #[max_length = 1024]
        s3_key -> Varchar,
        #[max_length = 255]
        etag -> Nullable<Varchar>,
    }
}

diesel::joinable!(s3_files -> files (file_id));

diesel::allow_tables_to_appear_in_same_query!(
    files,
    users,
    s3_files,
);

