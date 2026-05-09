use std::{env, fs, time::Duration};

use vwso_core::SecretDocument;
use vwso_vaultwarden::{
    VaultwardenApiClient, VaultwardenAuth, VaultwardenCacheConfig, VaultwardenDevice,
    VaultwardenEndpoint, VaultwardenEndpoints, VaultwardenProvider, VaultwardenSelector,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn resolves_configured_live_bitwarden_compatible_secret() -> TestResult {
    let Some(config) = LiveConfig::from_env()? else {
        eprintln!(
            "skipping live Bitwarden-compatible test; set VWSO_TEST_VAULTWARDEN_URL \
             or both VWSO_TEST_IDENTITY_URL and VWSO_TEST_API_URL, plus \
             VWSO_TEST_CLIENT_ID, VWSO_TEST_CLIENT_SECRET, VWSO_TEST_MASTER_PASSWORD, \
             and VWSO_TEST_ITEM_KEY or VWSO_TEST_ALLOW_ANY_ITEM=true"
        );
        return Ok(());
    };

    let endpoints = config.endpoint.endpoints()?;
    let auth = VaultwardenAuth {
        client_id: config.client_id,
        client_secret: config.client_secret.into(),
        master_password: config.master_password.into(),
    };
    let client = VaultwardenApiClient::with_endpoints_device_and_cache(
        endpoints,
        auth,
        VaultwardenDevice::default(),
        VaultwardenCacheConfig::new(Duration::from_secs(1)),
    )?;

    let selector = match config.selector {
        LiveSelectorConfig::Explicit { key, property } => VaultwardenSelector { key, property },
        LiveSelectorConfig::FirstExtractable { property } => {
            first_extractable_selector(&client, property).await?
        }
    };
    let requested_property = selector.property.clone();

    if let Some(path) = config.selector_output_path {
        write_selector_output(&path, &selector)?;
    }

    let document = client.resolve(selector).await?;
    assert_document_contains_expected_data(&document, requested_property.as_deref());

    Ok(())
}

struct LiveConfig {
    endpoint: LiveEndpointConfig,
    client_id: String,
    client_secret: String,
    master_password: String,
    selector: LiveSelectorConfig,
    selector_output_path: Option<String>,
}

enum LiveSelectorConfig {
    Explicit {
        key: String,
        property: Option<String>,
    },
    FirstExtractable {
        property: Option<String>,
    },
}

impl LiveConfig {
    fn from_env() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let endpoint = match (
            optional_env("VWSO_TEST_VAULTWARDEN_URL"),
            optional_env("VWSO_TEST_IDENTITY_URL"),
            optional_env("VWSO_TEST_API_URL"),
        ) {
            (Some(vaultwarden_url), None, None) => {
                LiveEndpointConfig::SingleOrigin { vaultwarden_url }
            }
            (None, Some(identity_url), Some(api_url)) => LiveEndpointConfig::Split {
                identity_url,
                api_url,
            },
            (None, None, None) => return Ok(None),
            _ => {
                return Err(config_error(
                    "live test endpoint config must use VWSO_TEST_VAULTWARDEN_URL or both VWSO_TEST_IDENTITY_URL and VWSO_TEST_API_URL",
                ));
            }
        };

        let Some(client_id) = required_env("VWSO_TEST_CLIENT_ID") else {
            return Ok(None);
        };
        let Some(client_secret) = required_env("VWSO_TEST_CLIENT_SECRET") else {
            return Ok(None);
        };
        let Some(master_password) = required_env("VWSO_TEST_MASTER_PASSWORD") else {
            return Ok(None);
        };
        let property = optional_env("VWSO_TEST_PROPERTY");
        let selector = match (
            optional_env("VWSO_TEST_ITEM_KEY"),
            truthy_env("VWSO_TEST_ALLOW_ANY_ITEM"),
        ) {
            (Some(key), _) => LiveSelectorConfig::Explicit { key, property },
            (None, true) => LiveSelectorConfig::FirstExtractable { property },
            (None, false) => return Ok(None),
        };

        Ok(Some(Self {
            endpoint,
            client_id,
            client_secret,
            master_password,
            selector,
            selector_output_path: optional_env("VWSO_TEST_SELECTOR_OUTPUT"),
        }))
    }
}

async fn first_extractable_selector(
    client: &VaultwardenApiClient,
    property: Option<String>,
) -> Result<VaultwardenSelector, Box<dyn std::error::Error>> {
    let session = client.login_with_api_key().await?;
    let sync = client.sync(&session).await?;
    let mut decrypted_count = 0usize;
    let mut non_extractable_count = 0usize;

    for cipher in &sync.ciphers {
        let Ok(decrypted) = cipher.decrypt(&session.user_key) else {
            continue;
        };
        decrypted_count += 1;

        if let Some(property) = property.as_deref() {
            if decrypted.extract_property(property).is_ok() {
                return Ok(VaultwardenSelector {
                    key: cipher.id.clone(),
                    property: Some(property.to_string()),
                });
            }
        } else if let Ok(document) = decrypted.to_secret_document() {
            return Ok(VaultwardenSelector {
                key: cipher.id.clone(),
                property: document.data.keys().next().cloned(),
            });
        }

        non_extractable_count += 1;
    }

    Err(dynamic_test_error(format!(
        "live test could not find a decryptable cipher with extractable secret fields; \
         synced {} ciphers, decrypted {decrypted_count}, non-extractable {non_extractable_count}",
        sync.ciphers.len()
    )))
}

fn assert_document_contains_expected_data(document: &SecretDocument, property: Option<&str>) {
    if let Some(property) = property {
        assert!(
            document.data.contains_key(property.trim()),
            "resolved document did not contain the requested property key"
        );
    } else {
        assert!(
            !document.data.is_empty(),
            "resolved document did not contain any secret data"
        );
    }
}

fn write_selector_output(path: &str, selector: &VaultwardenSelector) -> TestResult {
    let output = serde_json::json!({
        "key": &selector.key,
        "property": &selector.property,
    });
    fs::write(path, serde_json::to_vec_pretty(&output)?)?;
    Ok(())
}

enum LiveEndpointConfig {
    SingleOrigin {
        vaultwarden_url: String,
    },
    Split {
        identity_url: String,
        api_url: String,
    },
}

impl LiveEndpointConfig {
    fn endpoints(&self) -> Result<VaultwardenEndpoints, vwso_vaultwarden::VaultwardenClientError> {
        match self {
            Self::SingleOrigin { vaultwarden_url } => {
                let endpoint = VaultwardenEndpoint::parse(vaultwarden_url)?;
                Ok(VaultwardenEndpoints::from_single_origin(endpoint))
            }
            Self::Split {
                identity_url,
                api_url,
            } => VaultwardenEndpoints::parse_split(identity_url, api_url),
        }
    }
}

fn config_error(message: &'static str) -> Box<dyn std::error::Error> {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message).into()
}

fn dynamic_test_error(message: String) -> Box<dyn std::error::Error> {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message).into()
}

fn required_env(name: &str) -> Option<String> {
    optional_env(name)
}

fn truthy_env(name: &str) -> bool {
    optional_env(name).is_some_and(|value| {
        matches!(
            value.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "y" | "on"
        )
    })
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
