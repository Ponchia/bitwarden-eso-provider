use std::{env, time::Duration};

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
             and VWSO_TEST_ITEM_KEY"
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
    let selector = VaultwardenSelector {
        key: config.item_key,
        property: config.property.clone(),
    };

    let document = client.resolve(selector).await?;

    if let Some(property) = config.property {
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

    Ok(())
}

struct LiveConfig {
    endpoint: LiveEndpointConfig,
    client_id: String,
    client_secret: String,
    master_password: String,
    item_key: String,
    property: Option<String>,
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
        let Some(item_key) = required_env("VWSO_TEST_ITEM_KEY") else {
            return Ok(None);
        };

        Ok(Some(Self {
            endpoint,
            client_id,
            client_secret,
            master_password,
            item_key,
            property: optional_env("VWSO_TEST_PROPERTY"),
        }))
    }
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

fn required_env(name: &str) -> Option<String> {
    optional_env(name)
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
