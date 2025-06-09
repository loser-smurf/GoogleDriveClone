use actix_web::{web, App, HttpServer};
use database::DbPool;

mod database;
mod models;
mod schema;
mod handlers;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = database::create_pool();

    HttpServer::new( move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/api/files")
                    .route("", web::post().to(handlers::files::upload_file))   
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
