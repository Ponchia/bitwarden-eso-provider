#![forbid(unsafe_code)]

//! Vaultwarden/Bitwarden-compatible client boundary.
//!
//! This crate will hold API, authentication, and decryption code. It currently
//! exposes a narrow trait so adapters can be built and tested before the
//! HTTP provider implementation is filled in.

pub mod api;
pub mod cipher;
pub mod crypto;
pub mod keys;

use async_trait::async_trait;
use secrecy::SecretString;
use thiserror::Error;
use url::Url;
use vwso_core::{require_non_empty, RemoteRef, SecretDocument, ValidationError};

pub use api::{
    SyncResponse, VaultwardenApiClient, VaultwardenApiError, VaultwardenCacheConfig,
    VaultwardenDevice, VaultwardenSession,
};
pub use cipher::{
    CipherError, DecryptedCipher, DecryptedField, DecryptedLogin, DecryptedSshKey, EncryptedCipher,
    EncryptedField, EncryptedLogin, EncryptedSshKey,
};
pub use crypto::{AuthenticatedSymmetricKey, CryptoError, EncryptedString, EncryptionType};
pub use keys::{
    derive_master_key, master_password_authentication_hash, normalize_master_password_salt,
    stretch_master_key, unwrap_user_key_with_master_key, KdfConfig, KeyDerivationError, MasterKey,
    MasterPasswordUnlockData,
};

/// Single-origin Vaultwarden or self-hosted Bitwarden endpoint configuration.
#[derive(Debug, Clone)]
pub struct VaultwardenEndpoint {
    base_url: Url,
}

impl VaultwardenEndpoint {
    /// Parse and validate a single-origin Vaultwarden or self-hosted Bitwarden
    /// base URL.
    ///
    /// HTTP is allowed only for localhost development endpoints. Production
    /// deployments must use HTTPS.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is empty, malformed, or uses an insecure
    /// non-local transport.
    pub fn parse(raw: &str) -> Result<Self, VaultwardenClientError> {
        Self::parse_named(raw, "vaultwarden_url")
    }

    fn parse_named(raw: &str, field_name: &'static str) -> Result<Self, VaultwardenClientError> {
        require_non_empty(raw, field_name)?;
        let base_url =
            Url::parse(raw).map_err(|source| VaultwardenClientError::InvalidEndpoint { source })?;

        let host = base_url.host_str().unwrap_or_default();
        let is_localhost = matches!(host, "localhost" | "127.0.0.1" | "::1");
        if base_url.scheme() != "https" && !(base_url.scheme() == "http" && is_localhost) {
            return Err(VaultwardenClientError::InsecureEndpoint);
        }

        Ok(Self { base_url })
    }

    /// Return the configured base URL.
    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }
}

/// Fully resolved Bitwarden-compatible endpoint bases.
#[derive(Debug, Clone)]
pub struct VaultwardenEndpoints {
    identity_url: Url,
    api_url: Url,
}

impl VaultwardenEndpoints {
    /// Build endpoint bases from a single Vaultwarden or self-hosted Bitwarden
    /// origin.
    #[must_use]
    pub fn from_single_origin(endpoint: VaultwardenEndpoint) -> Self {
        let base_url = endpoint.base_url;

        Self {
            identity_url: append_path_segments(&base_url, &["identity"]),
            api_url: append_path_segments(&base_url, &["api"]),
        }
    }

    /// Parse explicit identity and API endpoint bases.
    ///
    /// This is the mode used by Bitwarden Cloud, for example
    /// `https://identity.bitwarden.com` plus `https://api.bitwarden.com`.
    ///
    /// # Errors
    ///
    /// Returns an error when either URL is empty, malformed, or uses an
    /// insecure non-local transport.
    pub fn parse_split(identity_url: &str, api_url: &str) -> Result<Self, VaultwardenClientError> {
        let identity = VaultwardenEndpoint::parse_named(identity_url, "identity_url")?;
        let api = VaultwardenEndpoint::parse_named(api_url, "api_url")?;

        Ok(Self {
            identity_url: normalize_endpoint_base(identity.base_url()),
            api_url: normalize_endpoint_base(api.base_url()),
        })
    }

    /// Return the configured identity endpoint base.
    #[must_use]
    pub fn identity_url(&self) -> &Url {
        &self.identity_url
    }

    /// Return the configured API endpoint base.
    #[must_use]
    pub fn api_url(&self) -> &Url {
        &self.api_url
    }
}

fn normalize_endpoint_base(base_url: &Url) -> Url {
    append_path_segments(base_url, &[])
}

fn append_path_segments(base_url: &Url, segments: &[&str]) -> Url {
    let mut url = base_url.clone();
    url.set_query(None);
    url.set_fragment(None);

    let mut path = url.path().trim_end_matches('/').to_string();
    for segment in segments {
        path.push('/');
        path.push_str(segment.trim_matches('/'));
    }
    url.set_path(&path);

    url
}

/// Authentication material for a dedicated Vaultwarden or Bitwarden user.
#[derive(Clone)]
pub struct VaultwardenAuth {
    /// User API key client ID.
    pub client_id: String,
    /// User API key client secret.
    pub client_secret: SecretString,
    /// Master password used for local vault decryption.
    pub master_password: SecretString,
}

/// Source selector understood by the Bitwarden-compatible provider.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VaultwardenSelector {
    /// Vault item key, ID, or stable path.
    pub key: String,
    /// Optional item field to extract.
    pub property: Option<String>,
}

impl TryFrom<RemoteRef> for VaultwardenSelector {
    type Error = VaultwardenClientError;

    fn try_from(remote_ref: RemoteRef) -> Result<Self, Self::Error> {
        require_non_empty(&remote_ref.key, "remote_ref.key")?;
        Ok(Self {
            key: remote_ref.key,
            property: remote_ref.property,
        })
    }
}

/// Provider boundary used by Kubernetes-facing adapters.
#[async_trait]
pub trait VaultwardenProvider: Send + Sync {
    /// Resolve a selector into a secret document.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider cannot authenticate, locate, decrypt,
    /// or map the selected vault item.
    async fn resolve(
        &self,
        selector: VaultwardenSelector,
    ) -> Result<SecretDocument, VaultwardenClientError>;
}

/// Placeholder provider used while the API and crypto implementation is designed.
#[derive(Debug, Default)]
pub struct NotImplementedProvider;

#[async_trait]
impl VaultwardenProvider for NotImplementedProvider {
    async fn resolve(
        &self,
        selector: VaultwardenSelector,
    ) -> Result<SecretDocument, VaultwardenClientError> {
        Err(VaultwardenClientError::NotImplemented { key: selector.key })
    }
}

/// Errors returned by the Bitwarden-compatible client boundary.
#[derive(Debug, Error)]
pub enum VaultwardenClientError {
    /// Shared validation failure.
    #[error(transparent)]
    Validation(#[from] ValidationError),
    /// Symmetric crypto failure.
    #[error(transparent)]
    Crypto(#[from] CryptoError),
    /// Cipher model or field extraction failure.
    #[error(transparent)]
    Cipher(#[from] CipherError),
    /// Master-password key derivation or unlock failure.
    #[error(transparent)]
    KeyDerivation(#[from] KeyDerivationError),
    /// Bitwarden-compatible HTTP API failure.
    #[error(transparent)]
    Api(#[from] VaultwardenApiError),
    /// URL parsing failed.
    #[error("invalid Vaultwarden endpoint")]
    InvalidEndpoint {
        /// URL parser source error.
        #[source]
        source: url::ParseError,
    },
    /// Endpoint does not meet transport security requirements.
    #[error("Bitwarden-compatible endpoint must use HTTPS except for localhost development")]
    InsecureEndpoint,
    /// Requested operation is not implemented yet.
    #[error("Vaultwarden resolution is not implemented for key {key}")]
    NotImplemented {
        /// Requested key.
        key: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_requires_https_for_non_local_hosts() {
        let Err(err) = VaultwardenEndpoint::parse("http://vault.example.test") else {
            unreachable!("non-local HTTP endpoint should fail validation");
        };

        assert!(matches!(err, VaultwardenClientError::InsecureEndpoint));
    }

    #[test]
    fn endpoint_allows_local_http_for_development() {
        let endpoint = match VaultwardenEndpoint::parse("http://127.0.0.1:8080") {
            Ok(endpoint) => endpoint,
            Err(error) => unreachable!("local HTTP endpoint should be accepted: {error}"),
        };

        assert_eq!(endpoint.base_url().scheme(), "http");
    }

    #[test]
    fn single_origin_endpoints_append_identity_and_api_paths() {
        let endpoint = match VaultwardenEndpoint::parse("https://vault.example.test/base/") {
            Ok(endpoint) => endpoint,
            Err(error) => unreachable!("endpoint should parse: {error}"),
        };
        let endpoints = VaultwardenEndpoints::from_single_origin(endpoint);

        assert_eq!(
            endpoints.identity_url().as_str(),
            "https://vault.example.test/base/identity"
        );
        assert_eq!(
            endpoints.api_url().as_str(),
            "https://vault.example.test/base/api"
        );
    }

    #[test]
    fn split_endpoints_keep_identity_and_api_bases_separate() {
        let endpoints = match VaultwardenEndpoints::parse_split(
            "https://identity.bitwarden.com/",
            "https://api.bitwarden.com/",
        ) {
            Ok(endpoints) => endpoints,
            Err(error) => unreachable!("split endpoints should parse: {error}"),
        };

        assert_eq!(
            endpoints.identity_url().as_str(),
            "https://identity.bitwarden.com/"
        );
        assert_eq!(endpoints.api_url().as_str(), "https://api.bitwarden.com/");
    }

    #[test]
    fn split_endpoints_reject_insecure_remote_http() {
        let Err(err) = VaultwardenEndpoints::parse_split(
            "https://identity.bitwarden.com",
            "http://api.example.test",
        ) else {
            unreachable!("split endpoints should reject insecure remote HTTP");
        };

        assert!(matches!(err, VaultwardenClientError::InsecureEndpoint));
    }

    #[test]
    fn selector_rejects_empty_keys() {
        let Err(err) = VaultwardenSelector::try_from(RemoteRef {
            key: " ".to_string(),
            property: None,
            version: None,
        }) else {
            unreachable!("empty selector key should fail validation");
        };

        assert!(matches!(err, VaultwardenClientError::Validation(_)));
    }
}
