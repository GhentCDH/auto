//! OIDC provider integration: discovery, authorization URL, code exchange and
//! ID-token validation. Provider metadata is discovered once at startup; the
//! lightweight `CoreClient` is rebuilt per request from the cached metadata
//! (no network), so we never have to name its verbose type-state generics.

use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::reqwest;
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};

use crate::config::Config;
use crate::{Error, Result};

/// Path the IdP redirects back to. Must match the route registered in
/// `api::auth` (`/auth/oidc/callback` nested under `/api`). The redirect URL is
/// derived as `base_url + OIDC_CALLBACK_PATH` unless explicitly overridden.
pub const OIDC_CALLBACK_PATH: &str = "/api/auth/oidc/callback";

/// A normalised OIDC identity, extracted from a validated ID token.
#[derive(Debug, Clone)]
pub struct OidcIdentity {
    pub sub: String,
    pub issuer: String,
    pub email: Option<String>,
}

/// Discovered OIDC provider plus client credentials. Cheap to clone the parts
/// needed to rebuild a client per request.
pub struct OidcProvider {
    metadata: CoreProviderMetadata,
    client_id: ClientId,
    client_secret: Option<ClientSecret>,
    redirect_uri: RedirectUrl,
    http: reqwest::Client,
}

impl OidcProvider {
    /// Run OIDC discovery against the configured issuer. Returns `Ok(None)` when
    /// OIDC is disabled or not fully configured.
    pub async fn discover(config: &Config) -> Result<Option<Self>> {
        if !config.auth_oidc_enabled {
            return Ok(None);
        }
        let (Some(issuer), Some(client_id)) = (&config.oidc_issuer_url, &config.oidc_client_id)
        else {
            return Err(Error::InternalError(
                "AUTH_OIDC_ENABLED is set but OIDC_ISSUER_URL / OIDC_CLIENT_ID are missing"
                    .to_string(),
            ));
        };

        // Default the redirect to base_url + the callback route; an explicit
        // OIDC_REDIRECT_URL overrides it for unusual proxy setups.
        let redirect = match &config.oidc_redirect_url {
            Some(u) => u.to_string(),
            None => format!(
                "{}{}",
                config.base_url.trim_end_matches('/'),
                OIDC_CALLBACK_PATH
            ),
        };

        // Disable redirects on the discovery/token client to mitigate SSRF.
        let http = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| Error::InternalError(format!("oidc http client init failed: {e}")))?;

        let issuer_url = IssuerUrl::new(issuer.to_string())
            .map_err(|e| Error::InternalError(format!("invalid OIDC_ISSUER_URL: {e}")))?;
        let metadata = CoreProviderMetadata::discover_async(issuer_url, &http)
            .await
            .map_err(|e| Error::InternalError(format!("oidc discovery failed: {e}")))?;
        let redirect_uri = RedirectUrl::new(redirect)
            .map_err(|e| Error::InternalError(format!("invalid OIDC redirect URL: {e}")))?;

        Ok(Some(Self {
            metadata,
            client_id: ClientId::new(client_id.clone()),
            client_secret: config.oidc_client_secret.clone().map(ClientSecret::new),
            redirect_uri,
            http,
        }))
    }

    fn client(
        &self,
    ) -> CoreClient<
        openidconnect::EndpointSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointMaybeSet,
        openidconnect::EndpointMaybeSet,
    > {
        CoreClient::from_provider_metadata(
            self.metadata.clone(),
            self.client_id.clone(),
            self.client_secret.clone(),
        )
        .set_redirect_uri(self.redirect_uri.clone())
    }

    /// Build the IdP authorization URL plus the CSRF, nonce and PKCE values that
    /// must be carried across the redirect and replayed on callback.
    pub fn authorize_url(&self) -> (String, CsrfToken, Nonce, PkceCodeVerifier) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, csrf, nonce) = self
            .client()
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        (url.to_string(), csrf, nonce, pkce_verifier)
    }

    /// Exchange an authorization code for tokens and validate the ID token,
    /// returning the normalised identity.
    pub async fn exchange(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
        nonce: &Nonce,
    ) -> Result<OidcIdentity> {
        let client = self.client();
        let token_response = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|e| Error::Unauthorized(format!("oidc token endpoint unavailable: {e}")))?
            .set_pkce_verifier(pkce_verifier)
            .request_async(&self.http)
            .await
            .map_err(|e| Error::Unauthorized(format!("oidc code exchange failed: {e}")))?;

        let id_token = token_response
            .id_token()
            .ok_or_else(|| Error::Unauthorized("oidc response had no id token".to_string()))?;

        let claims = id_token
            .claims(&client.id_token_verifier(), nonce)
            .map_err(|e| Error::Unauthorized(format!("oidc id token invalid: {e}")))?;

        Ok(OidcIdentity {
            sub: claims.subject().to_string(),
            issuer: claims.issuer().to_string(),
            email: claims.email().map(|e| e.to_string()),
        })
    }
}
