use actix_web::{Error, HttpRequest};
use futures_util::future::{Ready, ready};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // Subject (user ID)
    exp: usize,  // Expiration timestamp (unix seconds)
}

// Fetch the JWT secret from environment variable at runtime
fn jwt_secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set in environment")
        .into_bytes()
}

/// Creates a JWT token for a given user ID with 24H expiration.
pub fn create_jwt(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Calculate expiration time: current time + 24 hours
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
        + 60 * 60 * 24;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&jwt_secret()),
    )
}

/// Validates the JWT token and returns the user_id (subject) if valid.
pub fn validate_jwt(token: &str) -> Option<String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(&jwt_secret()),
        &Validation::default(),
    )
    .map(|data| data.claims.sub)
    .ok()
}

/// Extractor for authenticated user from the "auth_token" cookie.
pub struct AuthenticatedUser {
    pub user_id: String,
}

impl actix_web::FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    /// Extract user info from JWT stored in "auth_token" cookie.
    /// Returns Unauthorized error if missing or invalid.
    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(cookie) = req.cookie("auth_token") {
            if let Some(user_id) = validate_jwt(cookie.value()) {
                return ready(Ok(AuthenticatedUser { user_id }));
            }
        }
        ready(Err(actix_web::error::ErrorUnauthorized("Unauthorized")))
    }
}
