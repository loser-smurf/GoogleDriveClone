use crate::auth::google::GoogleOAuthClient;
use crate::config::UPLOAD_DIR;
use crate::storage::FilesStorage;
use actix_web::{App, HttpServer, web};

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

    let pool = database::create_pool();
    let storage = FilesStorage::new(UPLOAD_DIR);
    let oauth_client = web::Data::new(GoogleOAuthClient::new());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(storage.clone()))
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
