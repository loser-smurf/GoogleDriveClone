use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use std::env;

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
