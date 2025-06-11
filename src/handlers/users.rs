use crate::auth::google::GoogleOAuthClient;
use crate::models::users::NewUser;
use crate::requests::oauth::{GoogleUserInfo, OAuthCallbackQuery};
use actix_web::{Error, HttpResponse, web};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse, reqwest::async_http_client};

/// GET /auth/google
/// Redirects the user to Google's OAuth 2.0 authorization endpoint.
pub async fn google_auth(
    oauth_client: web::Data<GoogleOAuthClient>,
) -> Result<HttpResponse, Error> {
    let (auth_url, _csrf_token) = oauth_client
        .client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish())
}

/// GET /auth/google/callback
/// Handles the OAuth 2.0 callback from Google after user authorization.
/// Exchanges the authorization code for an access token, fetches user info,
/// and constructs a new user record.
pub async fn google_callback(
    query: web::Query<OAuthCallbackQuery>,
    oauth_client: web::Data<GoogleOAuthClient>,
) -> Result<HttpResponse, Error> {
    match (&query.code, &query.error) {
        (Some(auth_code), None) => {
            let token_response = oauth_client
                .client
                .exchange_code(AuthorizationCode::new(auth_code.clone()))
                .request_async(async_http_client)
                .await
                .map_err(|_| actix_web::error::ErrorInternalServerError("Token exchange failed"))?;

            let user_info: GoogleUserInfo = reqwest::Client::new()
                .get("https://www.googleapis.com/oauth2/v3/userinfo")
                .bearer_auth(token_response.access_token().secret())
                .send()
                .await
                .map_err(|_| {
                    actix_web::error::ErrorInternalServerError("Failed to fetch user info")
                })?
                .json()
                .await
                .map_err(|_| {
                    actix_web::error::ErrorInternalServerError("Failed to parse user info")
                })?;

            let new_user = NewUser {
                oauth_provider: "google".to_string(),
                oauth_user_id: user_info.sub,
                email: user_info.email,
                username: user_info.name,
                avatar_url: user_info.picture,
            };

            println!("New User Info:");
            println!("   - Provider: {}", new_user.oauth_provider);
            println!("   - User ID: {}", new_user.oauth_user_id);
            println!("   - Email: {:?}", new_user.email);
            println!("   - Username: {:?}", new_user.username);
            println!("   - Avatar URL: {:?}", new_user.avatar_url);

            Ok(HttpResponse::Found()
                .append_header(("Location", "/auth-success"))
                .finish())
        }
        (None, Some(err)) => Ok(HttpResponse::Found()
            .append_header(("Location", format!("/auth-error?error={}", err)))
            .finish()),
        _ => Ok(HttpResponse::Found()
            .append_header(("Location", "/auth-error?error=invalid_request"))
            .finish()),
    }
}
