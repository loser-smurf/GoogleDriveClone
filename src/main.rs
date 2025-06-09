use actix_web::{App, HttpServer, web};
use database::DbPool;

mod database;
mod handlers;
mod models;
mod schema;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = database::create_pool();

    HttpServer::new(move || {
        App::new().app_data(web::Data::new(pool.clone())).service(
            web::scope("/api/files")
                .route("", web::get().to(handlers::files::list_files))
                .route("", web::post().to(handlers::files::upload_file))
                .route("/{id}", web::get().to(handlers::files::download_file))
                .route("/{id}", web::delete().to(handlers::files::delete_file)),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
