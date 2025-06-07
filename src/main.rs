use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
};
use std::env;

#[get("/auth/google")]
async fn google_auth() -> HttpResponse {
    // Initialize OAuth2 client with Google endpoints
    let client = BasicClient::new(
        ClientId::new(env::var("CLIENT_ID").expect("CLIENT_ID not set")),
        Some(ClientSecret::new(env::var("CLIENT_SECRET").expect("CLIENT_SECRET not set"))),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .expect("Invalid authorization endpoint URL"),
        Some(
            TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
                .expect("Invalid token endpoint URL"),
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new(env::var("REDIRECT_URI").expect("REDIRECT_URI not set"))
            .expect("Invalid redirect URL"),
    );

    // Generate authorization URL with required scopes
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))  // Request access to user's email
        .add_scope(Scope::new("profile".to_string()))  // Request basic profile info
        .url();

    // Redirect user to Google's authorization page
    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}

#[get("/auth/google/callback")]
async fn google_callback(
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // Log incoming parameters
    println!("OAuth callback received: {:?}", query.0);

    match (query.get("code"), query.get("error")) {
        // Successful authorization case
        (Some(code), None) => {
            println!("Authorization code received: {}", code);
            
            // Here you would:
            // 1. Exchange code for tokens
            // 2. Fetch user info
            // 3. Authenticate user
            
            HttpResponse::Ok()
                .content_type("text/plain")
                .body("Authorization successful. Code received. Check server logs.")
        }
        
        // Error from Google
        (None, Some(error)) => {
            let error_desc = query.get("error_description")
                .map(|s| s.as_str())
                .unwrap_or("no description");
                
            eprintln!("OAuth error: {} ({})", error, error_desc);
            
            HttpResponse::BadRequest()
                .content_type("text/plain")
                .body(format!("Authorization error: {} ({})", error, error_desc))
        }
        
        // Invalid request case
        _ => {
            eprintln!("Invalid callback parameters: {:?}", query.0);
            
            HttpResponse::BadRequest()
                .content_type("text/plain")
                .body("Invalid request: missing authorization code or error parameter")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    println!("Server running at http://localhost:8080");

    // Configure and start HTTP server
    HttpServer::new(|| {
        App::new()
            .service(google_auth)  // OAuth initiation endpoint
            .service(google_callback)  // OAuth callback endpoint
    })
    .bind("127.0.0.1:8080")?  // Bind to localhost
    .run()
    .await
}
