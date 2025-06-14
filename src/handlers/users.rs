use crate::auth::jwt::AuthenticatedUser;
use actix_web::HttpResponse;
use log::info;

pub async fn protected_route(user: AuthenticatedUser) -> HttpResponse {
    info!("Accessing protected route by user_id: {}", user.user_id);
    HttpResponse::Ok().body(format!("Hello, user_id: {}", user.user_id))
}
