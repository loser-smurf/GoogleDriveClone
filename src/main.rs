use crate::auth::google::GoogleOAuthClient;
use crate::config::UPLOAD_DIR;
use crate::storage::FilesStorage;
use crate::storage::S3Storage;
use aws_sdk_s3::Client;
use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::Region;
use actix_web::{App, HttpServer, web};
use std::env;

mod auth;
mod config;
mod database;
mod handlers;
mod models;
mod repositories;
mod requests;
mod schema;
mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    dotenv::dotenv().ok();

    let region_provider = RegionProviderChain::default_provider()
        .or_else(Region::new(env::var("AWS_REGION").unwrap_or_else(|_| "eu-north-1".to_string())));

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let client = Client::new(&config);
    let baucket_name = env::var("AWS_BUCKET_NAME").unwrap_or_else(|_| "file-storage".to_string());
    let storage_s3 = S3Storage::new(client, baucket_name);


    let storage = FilesStorage::new(UPLOAD_DIR);


    let pool = database::create_pool();
    let oauth_client = web::Data::new(GoogleOAuthClient::new());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(storage.clone()))
            .app_data(web::Data::new(storage_s3.clone()))
            .app_data(oauth_client.clone())
            .service(
                web::scope("/api/files")
                    .route("", web::get().to(handlers::files::list_files))
                    .route("", web::post().to(handlers::files::upload_file))
                    .route("/{id}", web::get().to(handlers::files::download_file))
                    .route("/{id}", web::delete().to(handlers::files::delete_file))
                    .route("/{id}/meta", web::get().to(handlers::files::get_metadata))
                    .route("/search", web::get().to(handlers::files::search_files)),
            )
            .service(
                web::scope("/auth")
                    .route("/google", web::get().to(auth::google::google_auth))
                    .route(
                        "/google/callback",
                        web::get().to(auth::google::google_callback),
                    )
                    .route(
                        "/protected",
                        web::post().to(handlers::users::protected_route),
                    ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
