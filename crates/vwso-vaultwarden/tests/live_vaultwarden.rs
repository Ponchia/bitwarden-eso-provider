use std::{env, time::Duration};

use vwso_vaultwarden::{
    VaultwardenApiClient, VaultwardenAuth, VaultwardenCacheConfig, VaultwardenDevice,
    VaultwardenEndpoint, VaultwardenProvider, VaultwardenSelector,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn resolves_configured_live_vaultwarden_secret() -> TestResult {
    let Some(config) = LiveConfig::from_env() else {
        eprintln!(
            "skipping live Vaultwarden test; set VWSO_TEST_VAULTWARDEN_URL, \
             VWSO_TEST_CLIENT_ID, VWSO_TEST_CLIENT_SECRET, VWSO_TEST_MASTER_PASSWORD, \
             and VWSO_TEST_ITEM_KEY"
        );
        return Ok(());
    };

    let endpoint = VaultwardenEndpoint::parse(&config.vaultwarden_url)?;
    let auth = VaultwardenAuth {
        client_id: config.client_id,
        client_secret: config.client_secret.into(),
        master_password: config.master_password.into(),
    };
    let client = VaultwardenApiClient::with_device_and_cache(
        endpoint,
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
    vaultwarden_url: String,
    client_id: String,
    client_secret: String,
    master_password: String,
    item_key: String,
    property: Option<String>,
}

impl LiveConfig {
    fn from_env() -> Option<Self> {
        Some(Self {
            vaultwarden_url: required_env("VWSO_TEST_VAULTWARDEN_URL")?,
            client_id: required_env("VWSO_TEST_CLIENT_ID")?,
            client_secret: required_env("VWSO_TEST_CLIENT_SECRET")?,
            master_password: required_env("VWSO_TEST_MASTER_PASSWORD")?,
            item_key: required_env("VWSO_TEST_ITEM_KEY")?,
            property: optional_env("VWSO_TEST_PROPERTY"),
        })
    }
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
