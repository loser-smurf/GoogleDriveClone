use crate::auth::jwt::create_jwt;
use crate::database::DbPool;
use crate::models::users::NewUser;
use crate::repositories::users::{find_user_by_oauth, insert_user};
use crate::requests::oauth::{GoogleUserInfo, OAuthCallbackQuery};

use actix_web::cookie::{Cookie, SameSite};
use actix_web::{Error, HttpResponse, web};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, basic::BasicClient, reqwest::async_http_client,
};
use std::env;

use log::{error, info};

pub struct GoogleOAuthClient {
    pub client: BasicClient,
}

impl GoogleOAuthClient {
    pub fn new() -> Self {
        let client = BasicClient::new(
            ClientId::new(env::var("CLIENT_ID").expect("CLIENT_ID must be set")),
            Some(ClientSecret::new(
                env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set"),
            )),
            AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).unwrap(),
            Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).unwrap()),
        )
        .set_redirect_uri(
            RedirectUrl::new(env::var("REDIRECT_URI").expect("REDIRECT_URI must be set")).unwrap(),
        );

        GoogleOAuthClient { client }
    }
}

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

    info!("Redirecting to Google OAuth URL: {}", auth_url);

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish())
}

/// GET /auth/google/callback
/// Handles the OAuth 2.0 callback from Google after user authorization.
pub async fn google_callback(
    query: web::Query<OAuthCallbackQuery>,
    oauth_client: web::Data<GoogleOAuthClient>,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    match (&query.code, &query.error) {
        (Some(auth_code), None) => {
            info!("Received OAuth callback with code");

            // Exchange the code for a token
            let token_response = oauth_client
                .client
                .exchange_code(AuthorizationCode::new(auth_code.clone()))
                .request_async(async_http_client)
                .await
                .map_err(|e| {
                    error!("Token exchange failed: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Token exchange failed")
                })?;

            info!("Token exchange successful");

            // Request user info from Google
            let user_info: GoogleUserInfo = reqwest::Client::new()
                .get("https://www.googleapis.com/oauth2/v3/userinfo")
                .bearer_auth(token_response.access_token().secret())
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to fetch user info: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Failed to fetch user info")
                })?
                .json()
                .await
                .map_err(|e| {
                    error!("Failed to parse user info: {:?}", e);
                    actix_web::error::ErrorInternalServerError("Failed to parse user info")
                })?;

            info!(
                "Fetched user info: email={:?}, id={:?}",
                user_info.email, user_info.sub
            );

            // Create a new user object
            let new_user = NewUser {
                oauth_provider: "google".to_string(),
                oauth_user_id: user_info.sub.clone(),
                email: user_info.email.clone(),
                username: user_info.name.clone(),
                avatar_url: user_info.picture.clone(),
            };

            // Check if the user exists in the database
            let user_id = match find_user_by_oauth(&db_pool, "google", &user_info.sub) {
                Ok(Some(user)) => {
                    info!("User already exists in DB: id={}", user.id);
                    user.id.to_string()
                }
                Ok(None) => {
                    info!("User not found in DB, inserting new user");
                    // If not, insert the new user
                    match insert_user(&db_pool, &new_user) {
                        Ok(user) => {
                            info!("Inserted new user with id={}", user.id);
                            user.id.to_string()
                        }
                        Err(e) => {
                            error!("Failed to insert user: {}", e);
                            return Err(actix_web::error::ErrorInternalServerError(
                                "Failed to insert user",
                            ));
                        }
                    }
                }
                Err(e) => {
                    error!("Database error: {}", e);
                    return Err(actix_web::error::ErrorInternalServerError("Database error"));
                }
            };

            // Create a JWT with the user_id
            let jwt = create_jwt(&user_id).map_err(|e| {
                error!("JWT creation failed: {:?}", e);
                actix_web::error::ErrorInternalServerError("JWT creation failed")
            })?;

            info!("JWT created for user_id={}", user_id);

            // Set the JWT in an HTTP-only cookie
            let cookie = Cookie::build("auth_token", jwt)
                .http_only(true)
                .secure(false) // should be true in production with HTTPS
                .path("/")
                .same_site(SameSite::Lax)
                .finish();

            info!("Setting auth_token cookie and redirecting to /auth-success");

            // Redirect to /auth-success with the cookie set
            Ok(HttpResponse::Found()
                .append_header(("Location", "/auth-success"))
                .cookie(cookie)
                .finish())
        }
        (None, Some(err)) => {
            error!("OAuth error received: {}", err);
            Ok(HttpResponse::Found()
                .append_header(("Location", format!("/auth-error?error={}", err)))
                .finish())
        }
        _ => {
            error!("Invalid OAuth callback request");
            Ok(HttpResponse::Found()
                .append_header(("Location", "/auth-error?error=invalid_request"))
                .finish())
        }
    }
}
