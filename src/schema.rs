// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        s3_key -> Varchar,  // S3 object key (path in bucket)
        #[max_length = 255]
        bucket_name -> Varchar,  // S3 bucket name
        size -> Int8,
        #[max_length = 100]
        mime_type -> Nullable<Varchar>,
        created_at -> Timestamp,
        last_modified -> Timestamp,  // Last modified time in S3
        user_id -> Varchar,
        #[max_length = 255]
        etag -> Nullable<Varchar>,  // S3 ETag for versioning
        #[max_length = 50]
        storage_class -> Nullable<Varchar>,  // S3 storage class (STANDARD, GLACIER, etc.)
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
