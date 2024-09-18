use anyhow::{Context, Result};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    reqwest,
    url::Url,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, StandardTokenResponse,
    TokenUrl,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
    scope: String,
}

pub struct OAuth {
    client: BasicClient,
    pkce_code_verifier: Option<PkceCodeVerifier>,
}

impl OAuth {
    pub fn new(client_id: String, client_secret: String) -> Self {
        // Set up the config for the Google OAuth2 process.
        Self {
            client: BasicClient::new(
                ClientId::new(client_id),
                Some(ClientSecret::new(client_secret)),
                AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                    .expect("Invalid authorization endpoint URL"),
                Some(
                    TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
                        .expect("Invalid token endpoint URL"),
                ),
            )
            .set_redirect_uri(
                RedirectUrl::new("http://127.0.0.1:5000/auth".to_string())
                    .expect("Invalid redirect URL"),
            ),
            pkce_code_verifier: None,
        }
    }

    pub async fn exhange_refresh(
        &self,
        token: String,
    ) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>> {
        Ok(self
            .client
            .exchange_refresh_token(&RefreshToken::new(token))
            .request_async(reqwest::async_http_client)
            .await?)
    }

    pub fn auth_url(&mut self) -> Url {
        // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
        // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
        self.pkce_code_verifier = Some(pkce_code_verifier);

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, _csrf_state) = self
            .client
            .authorize_url(CsrfToken::new_random)
            // This example is requesting access to the "calendar" features and the user's profile.
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/calendar".to_string(),
            ))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/plus.me".to_string(),
            ))
            .set_pkce_challenge(pkce_code_challenge)
            .url();
        authorize_url
    }

    pub async fn auth(
        &mut self,
        request: AuthRequest,
    ) -> Result<(
        CsrfToken,
        StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    )> {
        let _scope = request.scope;

        // Exchange the code with a token.
        Ok((
            CsrfToken::new(request.state),
            self.client
                .exchange_code(AuthorizationCode::new(request.code))
                .set_pkce_verifier(
                    self.pkce_code_verifier
                        .take()
                        .context("PKCE code verifier should exist at this point")?,
                )
                .request_async(reqwest::async_http_client)
                .await?,
        ))
    }
}
