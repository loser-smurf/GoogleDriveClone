
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
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Int4,
        sub -> Varchar,
        token_version -> Int4,
        exp -> Int8,
        iat -> Int8,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_tokens (id) {
        id -> Int4,
        user_id -> Int4,
        access_token -> Text,
        refresh_token -> Text,
        expires_at -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        google_id -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 100]
        name -> Nullable<Varchar>,
        picture -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(user_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    files,
    refresh_tokens,
    user_tokens,
    users,
);
