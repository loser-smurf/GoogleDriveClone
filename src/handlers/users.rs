use actix_web::HttpResponse;
use crate::auth::jwt::AuthenticatedUser;

pub async fn protected_route(user: AuthenticatedUser) -> HttpResponse {
    HttpResponse::Ok().body(format!("Hello, user_id: {}", user.user_id))
}
