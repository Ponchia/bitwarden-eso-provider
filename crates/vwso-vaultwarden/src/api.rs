//! Vaultwarden HTTP API client and sync resolver.

use async_trait::async_trait;
use reqwest::{Client as HttpClient, Url};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use vwso_core::SecretDocument;

use crate::{
    AuthenticatedSymmetricKey, DecryptedCipher, EncryptedCipher, KdfConfig,
    MasterPasswordUnlockData, VaultwardenAuth, VaultwardenClientError, VaultwardenEndpoint,
    VaultwardenProvider, VaultwardenSelector,
};

const BITWARDEN_CLIENT_VERSION: &str = "2025.12.0";
const DEFAULT_DEVICE_TYPE_SERVER: u8 = 22;

/// Vaultwarden HTTP API client.
#[derive(Clone)]
pub struct VaultwardenApiClient {
    endpoint: VaultwardenEndpoint,
    auth: VaultwardenAuth,
    http: HttpClient,
    device: VaultwardenDevice,
}

impl VaultwardenApiClient {
    /// Build a Vaultwarden API client with the default HTTP client and device
    /// identity.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn new(
        endpoint: VaultwardenEndpoint,
        auth: VaultwardenAuth,
    ) -> Result<Self, VaultwardenApiError> {
        Self::with_device(endpoint, auth, VaultwardenDevice::default())
    }

    /// Build a Vaultwarden API client with an explicit device identity.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn with_device(
        endpoint: VaultwardenEndpoint,
        auth: VaultwardenAuth,
        device: VaultwardenDevice,
    ) -> Result<Self, VaultwardenApiError> {
        let http = HttpClient::builder()
            .user_agent("vaultwarden-secrets-operator")
            .build()?;

        Ok(Self {
            endpoint,
            auth,
            http,
            device,
        })
    }

    /// Fetch the password prelogin KDF configuration for an email address.
    ///
    /// # Errors
    ///
    /// Returns an error for transport failures, non-success status codes,
    /// malformed responses, or KDF downgrade/resource validation failures.
    pub async fn prelogin(&self, email: &str) -> Result<KdfConfig, VaultwardenClientError> {
        let url = self.endpoint_url(&["identity", "accounts", "prelogin", "password"])?;
        let response = self
            .http
            .post(url)
            .bitwarden_headers()
            .json(&PreloginRequest { email })
            .send()
            .await
            .map_err(VaultwardenApiError::from)?;
        let response = decode_json::<PreloginResponse>(response, "prelogin").await?;

        Ok(response.try_into()?)
    }

    /// Authenticate with the configured user API key and unlock the user key.
    ///
    /// # Errors
    ///
    /// Returns an error when authentication, response parsing, or local unlock
    /// fails.
    pub async fn login_with_api_key(&self) -> Result<VaultwardenSession, VaultwardenClientError> {
        let url = self.endpoint_url(&["identity", "connect", "token"])?;
        let scope = if self.auth.client_id.starts_with("organization.") {
            "api.organization"
        } else {
            "api"
        };
        let request = ApiKeyTokenRequest {
            grant_type: "client_credentials",
            scope,
            client_id: &self.auth.client_id,
            client_secret: self.auth.client_secret.expose_secret(),
            device_identifier: &self.device.identifier,
            device_name: &self.device.name,
            device_type: self.device.device_type,
        };

        let response = self
            .http
            .post(url)
            .bitwarden_headers()
            .form(&request)
            .send()
            .await
            .map_err(VaultwardenApiError::from)?;
        let response = decode_json::<TokenResponse>(response, "token").await?;

        self.unlock_session(response)
    }

    /// Fetch a full vault sync response with an authenticated session.
    ///
    /// # Errors
    ///
    /// Returns an error when the sync request fails or the response cannot be
    /// decoded.
    pub async fn sync(
        &self,
        session: &VaultwardenSession,
    ) -> Result<SyncResponse, VaultwardenClientError> {
        let mut url = self.endpoint_url(&["api", "sync"])?;
        url.query_pairs_mut().append_pair("excludeDomains", "true");

        let response = self
            .http
            .get(url)
            .bitwarden_headers()
            .bearer_auth(session.access_token.expose_secret())
            .send()
            .await
            .map_err(VaultwardenApiError::from)?;

        Ok(decode_json::<SyncResponse>(response, "sync").await?)
    }

    fn unlock_session(
        &self,
        response: TokenResponse,
    ) -> Result<VaultwardenSession, VaultwardenClientError> {
        let unlock = response
            .user_decryption_options
            .and_then(|options| options.master_password_unlock)
            .ok_or(VaultwardenApiError::MissingMasterPasswordUnlock)?;
        let unlock_data = MasterPasswordUnlockData::try_from(unlock)?;
        let user_key = unlock_data.unlock_user_key(self.auth.master_password.expose_secret())?;

        Ok(VaultwardenSession {
            access_token: response.access_token.into(),
            expires_in: response.expires_in,
            token_type: response.token_type.unwrap_or_else(|| "Bearer".to_string()),
            user_key,
        })
    }

    fn resolve_synced_cipher(
        sync: &SyncResponse,
        user_key: &AuthenticatedSymmetricKey,
        key: &str,
    ) -> Result<DecryptedCipher, VaultwardenClientError> {
        for cipher in &sync.ciphers {
            if cipher.id == key {
                return Ok(cipher.decrypt(user_key)?);
            }

            if let Ok(decrypted) = cipher.decrypt(user_key) {
                if decrypted.name.as_deref() == Some(key) {
                    return Ok(decrypted);
                }
            }
        }

        Err(VaultwardenApiError::CipherNotFound {
            key: key.to_string(),
        }
        .into())
    }

    fn endpoint_url(&self, segments: &[&str]) -> Result<Url, VaultwardenApiError> {
        let mut url = self.endpoint.base_url().clone();
        url.set_query(None);
        url.set_fragment(None);

        {
            let mut path = url
                .path_segments_mut()
                .map_err(|()| VaultwardenApiError::InvalidBaseUrl)?;
            path.pop_if_empty();
            for segment in segments {
                path.push(segment);
            }
        }

        Ok(url)
    }
}

#[async_trait]
impl VaultwardenProvider for VaultwardenApiClient {
    async fn resolve(
        &self,
        selector: VaultwardenSelector,
    ) -> Result<SecretDocument, VaultwardenClientError> {
        let session = self.login_with_api_key().await?;
        let sync = self.sync(&session).await?;
        let cipher = Self::resolve_synced_cipher(&sync, &session.user_key, &selector.key)?;

        if let Some(property) = selector.property {
            let value = cipher.extract_property(&property)?;
            return Ok(SecretDocument::single(property.trim(), value));
        }

        Ok(cipher.to_secret_document()?)
    }
}

/// Stable device identity sent to Vaultwarden during API-key login.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaultwardenDevice {
    /// Bitwarden device type numeric value.
    pub device_type: u8,
    /// Stable device identifier.
    pub identifier: String,
    /// Human-readable device name.
    pub name: String,
}

impl Default for VaultwardenDevice {
    fn default() -> Self {
        Self {
            device_type: DEFAULT_DEVICE_TYPE_SERVER,
            identifier: "vaultwarden-secrets-operator".to_string(),
            name: "Vaultwarden Secrets Operator".to_string(),
        }
    }
}

/// Authenticated Vaultwarden session with an unlocked user key.
pub struct VaultwardenSession {
    access_token: SecretString,
    /// Server token expiry in seconds.
    pub expires_in: Option<u64>,
    /// Token type returned by the server.
    pub token_type: String,
    /// Locally unlocked user key.
    pub user_key: AuthenticatedSymmetricKey,
}

/// Minimal sync response fields needed by the provider.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResponse {
    /// Vault ciphers visible to the authenticated user.
    #[serde(default, alias = "Ciphers")]
    pub ciphers: Vec<EncryptedCipher>,
}

#[derive(Debug, Serialize)]
struct PreloginRequest<'a> {
    email: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PreloginResponse {
    #[serde(alias = "Kdf")]
    kdf: u8,
    #[serde(alias = "KdfIterations")]
    kdf_iterations: u32,
    #[serde(default, alias = "KdfMemory")]
    kdf_memory: Option<u32>,
    #[serde(default, alias = "KdfParallelism")]
    kdf_parallelism: Option<u32>,
}

impl TryFrom<PreloginResponse> for KdfConfig {
    type Error = VaultwardenApiError;

    fn try_from(response: PreloginResponse) -> Result<Self, Self::Error> {
        kdf_from_parts(
            response.kdf,
            response.kdf_iterations,
            response.kdf_memory,
            response.kdf_parallelism,
        )
    }
}

#[derive(Debug, Serialize)]
struct ApiKeyTokenRequest<'a> {
    grant_type: &'static str,
    scope: &'static str,
    client_id: &'a str,
    client_secret: &'a str,
    device_identifier: &'a str,
    device_name: &'a str,
    device_type: u8,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(alias = "accessToken")]
    access_token: String,
    #[serde(default)]
    #[serde(alias = "expiresIn")]
    expires_in: Option<u64>,
    #[serde(default)]
    #[serde(alias = "tokenType")]
    token_type: Option<String>,
    #[serde(
        default,
        rename = "userDecryptionOptions",
        alias = "UserDecryptionOptions"
    )]
    user_decryption_options: Option<UserDecryptionOptionsResponse>,
}

#[derive(Debug, Deserialize)]
struct UserDecryptionOptionsResponse {
    #[serde(
        default,
        rename = "masterPasswordUnlock",
        alias = "MasterPasswordUnlock"
    )]
    master_password_unlock: Option<MasterPasswordUnlockResponse>,
}

#[derive(Debug, Deserialize)]
struct MasterPasswordUnlockResponse {
    #[serde(rename = "kdf", alias = "Kdf")]
    kdf: TokenKdfResponse,
    #[serde(
        rename = "masterKeyWrappedUserKey",
        alias = "MasterKeyWrappedUserKey",
        alias = "MasterKeyEncryptedUserKey",
        alias = "masterKeyEncryptedUserKey"
    )]
    master_key_wrapped_user_key: String,
    #[serde(rename = "salt", alias = "Salt")]
    salt: String,
}

impl TryFrom<MasterPasswordUnlockResponse> for MasterPasswordUnlockData {
    type Error = VaultwardenApiError;

    fn try_from(response: MasterPasswordUnlockResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            salt: response.salt,
            kdf: response.kdf.try_into()?,
            master_key_wrapped_user_key: response.master_key_wrapped_user_key,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TokenKdfResponse {
    #[serde(rename = "kdfType", alias = "KdfType")]
    kdf_type: u8,
    #[serde(alias = "Iterations")]
    iterations: u32,
    #[serde(default, alias = "Memory")]
    memory: Option<u32>,
    #[serde(default, alias = "Parallelism")]
    parallelism: Option<u32>,
}

impl TryFrom<TokenKdfResponse> for KdfConfig {
    type Error = VaultwardenApiError;

    fn try_from(response: TokenKdfResponse) -> Result<Self, Self::Error> {
        kdf_from_parts(
            response.kdf_type,
            response.iterations,
            response.memory,
            response.parallelism,
        )
    }
}

fn kdf_from_parts(
    kdf_type: u8,
    iterations: u32,
    memory: Option<u32>,
    parallelism: Option<u32>,
) -> Result<KdfConfig, VaultwardenApiError> {
    match kdf_type {
        0 => Ok(KdfConfig::Pbkdf2Sha256 { iterations }),
        1 => Ok(KdfConfig::Argon2id {
            iterations,
            memory_mib: memory.ok_or(VaultwardenApiError::MissingKdfParameter {
                parameter: "memory",
            })?,
            parallelism: parallelism.ok_or(VaultwardenApiError::MissingKdfParameter {
                parameter: "parallelism",
            })?,
        }),
        value => Err(VaultwardenApiError::UnsupportedKdfType { kdf_type: value }),
    }
}

async fn decode_json<T>(
    response: reqwest::Response,
    endpoint: &'static str,
) -> Result<T, VaultwardenApiError>
where
    T: for<'de> Deserialize<'de>,
{
    let status = response.status();
    if !status.is_success() {
        return Err(VaultwardenApiError::HttpStatus {
            endpoint,
            status: status.as_u16(),
        });
    }

    Ok(response.json::<T>().await?)
}

trait BitwardenRequestHeaders {
    fn bitwarden_headers(self) -> Self;
}

impl BitwardenRequestHeaders for reqwest::RequestBuilder {
    fn bitwarden_headers(self) -> Self {
        self.header("Bitwarden-Client-Version", BITWARDEN_CLIENT_VERSION)
    }
}

/// Vaultwarden API errors.
#[derive(Debug, Error)]
pub enum VaultwardenApiError {
    /// HTTP client error.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    /// Base URL cannot be used for endpoint construction.
    #[error("Vaultwarden base URL cannot be used to build API endpoints")]
    InvalidBaseUrl,
    /// Server returned a non-success status.
    #[error("Vaultwarden {endpoint} request returned HTTP {status}")]
    HttpStatus {
        /// Logical endpoint name.
        endpoint: &'static str,
        /// HTTP status code.
        status: u16,
    },
    /// KDF type is unknown.
    #[error("unsupported Vaultwarden KDF type {kdf_type}")]
    UnsupportedKdfType {
        /// Numeric KDF type returned by the server.
        kdf_type: u8,
    },
    /// KDF response is missing a required parameter.
    #[error("Vaultwarden KDF response is missing {parameter}")]
    MissingKdfParameter {
        /// Missing parameter name.
        parameter: &'static str,
    },
    /// API-key login did not return master-password unlock data.
    #[error("Vaultwarden token response did not include master-password unlock data")]
    MissingMasterPasswordUnlock,
    /// Requested cipher was not present in the sync response.
    #[error("Vaultwarden cipher {key} was not found")]
    CipherNotFound {
        /// Requested selector key.
        key: String,
    },
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use axum::{
        extract::State,
        http::{header, HeaderMap, StatusCode},
        response::IntoResponse,
        routing::{get, post},
        Form, Json, Router,
    };
    use serde_json::json;
    use tokio::net::TcpListener;
    use vwso_core::RemoteRef;

    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const PASSWORD: &str = "correct horse battery staple";
    const WRAPPED_CIPHER_TEST_KEY: &str =
        "2.AAECAwQFBgcICQoLDA0ODw==|rjzJWhStJXa0gPxMK+QGHB11ccKE8Q8NPFwsxqnI2yjMiiEWnwgY5nr1JWhyD4A5Sk4zDqfAoY91Gkr2QBfYQW14lXNe3qb+pHOsLqJ2Qa0=|Jsse4qMpeoqJ6VzlA9ta9PXyWNBJGfyPgRxFo5RupbE=";
    const LOGIN_CIPHER_JSON: &str = r#"
{
  "id": "cipher-login",
  "type": 1,
  "organizationId": null,
  "name": "2.UFFSU1RVVldYWVpbXF1eXw==|StyR/qx1FDl2IiD+llUqbw==|mX23ZTaSooPqZL9DzozpOa4pZH6Q3EO1oEyCfLHAUTA=",
  "notes": "2.gIGCg4SFhoeIiYqLjI2Ojw==|iFVXYOIlaeVXv98BkXhsX9RonhSa845FON4Gz7ibpKk=|OLWFugRmFHwv6y45LU3rP+5CYeUrnlCsOtZGoJIWELI=",
  "fields": [
    {
      "name": "2.kJGSk5SVlpeYmZqbnJ2enw==|2xgwPgtCaGbLNZe2aV+eQA==|rTu4SR2oEKPpx9fpaTt4sBwPF1e2m6D9yS7uoTyNsqg=",
      "value": "2.QEFCQ0RFRkdISUpLTE1OTw==|SgvILpma5dxrOQiNaAGR699WX5rwBVaPsidtZD2BxAKBaMLSm4jnP2eD70tV04Nh|SH6OgAyy4VoHgC7ilEbBcvDKZUdH330hZpp5ImjlwU0=",
      "type": 1
    }
  ],
  "login": {
    "username": "2.YGFiY2RlZmdoaWprbG1ubw==|b+km1T/4QuXHSTO/qKV9+g==|t1Dmr15Mywo7Z0kRd0wlFsoj31Pa+HRs8v/8QC2nG5Q=",
    "password": "2.cHFyc3R1dnd4eXp7fH1+fw==|VOCFi5yrDwretU6eHBCbMLgy3Arezxhx4kmIp9olCcY=|AV5iXNORGRrVvOAyXdJ2aGMu+tv9wPJvpbxUEO8y2/8=",
    "totp": null
  }
}
"#;

    #[derive(Clone)]
    struct FakeState {
        cipher: serde_json::Value,
    }

    #[derive(Debug, Deserialize)]
    struct FakePreloginRequest {
        email: String,
    }

    #[derive(Debug, Deserialize)]
    struct FakeTokenForm {
        grant_type: String,
        scope: String,
        client_id: String,
        client_secret: String,
        device_identifier: String,
        device_name: String,
        device_type: u8,
    }

    #[tokio::test]
    async fn parses_prelogin_kdf_response() -> TestResult {
        let client = fake_client().await?;

        assert_eq!(
            client.prelogin("User@Example.COM").await?,
            KdfConfig::Pbkdf2Sha256 { iterations: 5_000 }
        );
        Ok(())
    }

    #[tokio::test]
    async fn resolves_cipher_property_through_api_key_login_and_sync() -> TestResult {
        let client = fake_client().await?;
        let selector = VaultwardenSelector::try_from(RemoteRef {
            key: "app/database".to_string(),
            property: Some("DATABASE_URL".to_string()),
            version: None,
        })?;

        let document = client.resolve(selector).await?;

        assert_eq!(
            document.data.get("DATABASE_URL"),
            Some(&"postgres://app:secret@db:5432/app".to_string())
        );
        Ok(())
    }

    #[tokio::test]
    async fn resolves_whole_cipher_to_secret_document() -> TestResult {
        let client = fake_client().await?;
        let selector = VaultwardenSelector::try_from(RemoteRef {
            key: "cipher-login".to_string(),
            property: None,
            version: None,
        })?;

        let document = client.resolve(selector).await?;

        assert_eq!(document.data.get("username"), Some(&"app".to_string()));
        assert_eq!(
            document.data.get("DATABASE_URL"),
            Some(&"postgres://app:secret@db:5432/app".to_string())
        );
        assert_eq!(
            document.metadata.get("vaultwarden.cipherId"),
            Some(&"cipher-login".to_string())
        );
        Ok(())
    }

    async fn fake_client() -> Result<VaultwardenApiClient, Box<dyn std::error::Error>> {
        let base_url = spawn_fake_server().await?;
        let endpoint = VaultwardenEndpoint::parse(&base_url)?;
        let auth = VaultwardenAuth {
            client_id: "user.fixture".to_string(),
            client_secret: "api-secret".into(),
            master_password: PASSWORD.into(),
        };

        Ok(VaultwardenApiClient::new(endpoint, auth)?)
    }

    async fn spawn_fake_server() -> Result<String, Box<dyn std::error::Error>> {
        let cipher = serde_json::from_str::<serde_json::Value>(LOGIN_CIPHER_JSON)?;
        let state = FakeState { cipher };
        let app = Router::new()
            .route("/identity/accounts/prelogin/password", post(fake_prelogin))
            .route("/identity/connect/token", post(fake_token))
            .route("/api/sync", get(fake_sync))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, app).await {
                eprintln!("fake Vaultwarden server failed: {error}");
            }
        });

        Ok(format!("http://{}", socket_addr(address)))
    }

    async fn fake_prelogin(Json(request): Json<FakePreloginRequest>) -> impl IntoResponse {
        if request.email.trim().is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "email is required" })),
            )
                .into_response();
        }

        Json(json!({
            "kdf": 0,
            "kdfIterations": 5_000,
            "kdfMemory": null,
            "kdfParallelism": null
        }))
        .into_response()
    }

    async fn fake_token(Form(form): Form<FakeTokenForm>) -> impl IntoResponse {
        let valid = form.grant_type == "client_credentials"
            && form.scope == "api"
            && form.client_id == "user.fixture"
            && form.client_secret == "api-secret"
            && form.device_identifier == "vaultwarden-secrets-operator"
            && form.device_name == "Vaultwarden Secrets Operator"
            && form.device_type == DEFAULT_DEVICE_TYPE_SERVER;
        if !valid {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "invalid token form" })),
            )
                .into_response();
        }

        Json(json!({
            "access_token": "fake-access-token",
            "expires_in": 3600,
            "token_type": "Bearer",
            "UserDecryptionOptions": {
                "HasMasterPassword": true,
                "MasterPasswordUnlock": {
                    "Kdf": {
                        "KdfType": 0,
                        "Iterations": 5_000,
                        "Memory": null,
                        "Parallelism": null
                    },
                    "MasterKeyWrappedUserKey": WRAPPED_CIPHER_TEST_KEY,
                    "Salt": " User@Example.COM "
                }
            }
        }))
        .into_response()
    }

    async fn fake_sync(State(state): State<FakeState>, headers: HeaderMap) -> impl IntoResponse {
        let auth = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());
        if auth != Some("Bearer fake-access-token") {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "missing bearer token" })),
            )
                .into_response();
        }

        Json(json!({
            "profile": {},
            "ciphers": [state.cipher],
            "folders": [],
            "collections": [],
            "domains": null,
            "sends": [],
            "userDecryption": {
                "masterPasswordUnlock": null
            },
            "object": "sync"
        }))
        .into_response()
    }

    fn socket_addr(address: SocketAddr) -> String {
        address.to_string()
    }
}
