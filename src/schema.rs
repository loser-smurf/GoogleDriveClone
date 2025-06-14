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

diesel::allow_tables_to_appear_in_same_query!(files, users,);
