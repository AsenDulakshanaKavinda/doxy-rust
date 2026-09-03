use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreTokenResponse,
};
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl, Scope, TokenResponse,
};

use std::env;


/// di

async fn init_google_client() -> Result<CoreClient, Box<dyn std::error::Error>> {
    // read credetials from the env variables
    let google_client_id = ClientId::new(env::var("GOOGLE_CLIENT_ID")?);
    let google_client_secret = ClientSecret::new(env::var("GOOGLE_CLIENT_SECRET")?);

    // google's standard issuer url
    let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string())?;

    // fetch google's OpenID connect discovery document
    let provider_metadata = CoreProviderMetadata::discover_async(
        issuer_url,
        &openidconnect::reqwest::async_http_client,
    )
    .await?;

    // set registerd redirect URI
    let redirect_url = RedirectUrl::new("http://localhost:8080/auth/callback".to_string())?;

    // create the ClientId
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    .set_redirect_uri(redirect_url);

    Ok(client)
}

fn generate_login_url(client: &CoreClient) -> (String, CsrfToken, Nonce) {
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            AuthenticationFlow::<openidconnect::core::CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // add basic OpenID scope for identity information
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    // Redirect the user to auth_url.to_string()
    // CRITICAL: Save csrf_token and nonce securely in the user's session storage.
    (auth_url.to_string(), csrf_token, nonce)
}

async fn handle_callback(
    client: &CoreClient,
    incoming_code: String,
    incoming_state: String,
    saved_csrf_token: CsrfToken, // Retrieve from session
    saved_nonce: Nonce, // Retrieve from session
) -> Result<(), Box<dyn std::error::Error>> {

    // 
    if incoming_state != *saved_csrf_token.secret() {
        return Err("CSRF token mismatch! Potential attack detected.".into());
    }

    // 
    let token_response = client
        .exchange_code(AuthorizationCode::new(incoming_code))
        .request_async(&openidconnect::reqwest::async_http_client)
        .await?;

    // 
    let id_token = token_response
        .id_token()
        .ok_or_else(|| "Server did not return an ID token")?;

    let claims = id_token.claims(
        &client.id_token_verifier(), 
        &saved_nonce
    )?;

    // 
    let user_id = claims.subject().to_string();
    let user_email = claims
        .email()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "No email provided".to_string());

    println!("Successfully authenticated User ID: {}, Email: {}", user_id, user_email);
    
    // Establish application session using user_id or user_email here...
    Ok(())


}






