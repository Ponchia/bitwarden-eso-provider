#![forbid(unsafe_code)]

use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{bail, Context};
use axum::{
    body::Body,
    extract::{MatchedPath, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bweso_bitwarden::{
    BitwardenApiClient, BitwardenApiError, BitwardenAuth, BitwardenCacheConfig,
    BitwardenClientError, BitwardenDevice, BitwardenEndpoint, BitwardenEndpoints,
    BitwardenProvider, BitwardenSelector, CipherError,
};
use bweso_core::{require_non_empty, RemoteRef, SecretDocument};
use clap::Parser;
use http::{header, Request, StatusCode};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod lifecycle;
mod metrics;

use lifecycle::Lifecycle;
use metrics::{AppMetrics, PROMETHEUS_CONTENT_TYPE};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(long, env = "BWESO_LISTEN", default_value = "0.0.0.0:8080")]
    listen: SocketAddr,
    #[arg(long, env = "BWESO_SINGLE_ORIGIN_URL")]
    single_origin_url: Option<String>,
    #[arg(long, env = "BWESO_IDENTITY_URL")]
    identity_url: Option<String>,
    #[arg(long, env = "BWESO_API_URL")]
    api_url: Option<String>,
    #[arg(long, env = "BWESO_CLIENT_ID")]
    client_id: String,
    #[arg(long, env = "BWESO_CLIENT_SECRET")]
    client_secret: String,
    #[arg(long, env = "BWESO_MASTER_PASSWORD")]
    master_password: String,
    #[arg(
        long,
        env = "BWESO_DEVICE_IDENTIFIER",
        default_value = "bitwarden-eso-provider"
    )]
    device_identifier: String,
    #[arg(
        long,
        env = "BWESO_DEVICE_NAME",
        default_value = "Bitwarden ESO Provider"
    )]
    device_name: String,
    #[arg(long, env = "BWESO_DEVICE_TYPE", default_value_t = 22)]
    device_type: u8,
    #[arg(long, env = "BWESO_CACHE_TTL_SECONDS", default_value_t = 60)]
    cache_ttl_seconds: u64,
}

#[derive(Clone)]
struct AppState {
    provider: Arc<dyn BitwardenProvider>,
    metrics: Arc<AppMetrics>,
    lifecycle: Lifecycle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolveRequest {
    remote_ref: RemoteRef,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let args = Args::parse();
    let listen = args.listen;
    let lifecycle = Lifecycle::default();

    let state = AppState {
        provider: provider_from_args(args)?,
        metrics: Arc::new(AppMetrics::new()),
        lifecycle: lifecycle.clone(),
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(listen).await?;
    tracing::info!(address = %listen, "starting ESO webhook provider");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(lifecycle))
        .await?;

    Ok(())
}

fn provider_from_args(args: Args) -> anyhow::Result<Arc<dyn BitwardenProvider>> {
    require_non_empty(&args.client_id, "client_id")?;
    require_non_empty(&args.client_secret, "client_secret")?;
    require_non_empty(&args.master_password, "master_password")?;
    require_non_empty(&args.device_identifier, "device_identifier")?;
    require_non_empty(&args.device_name, "device_name")?;

    let endpoints = endpoints_from_args(&args)?;
    let auth = BitwardenAuth {
        client_id: args.client_id,
        client_secret: args.client_secret.into(),
        master_password: args.master_password.into(),
    };
    let device = BitwardenDevice {
        device_type: args.device_type,
        identifier: args.device_identifier,
        name: args.device_name,
    };
    let cache_config = BitwardenCacheConfig::new(Duration::from_secs(args.cache_ttl_seconds));
    let provider =
        BitwardenApiClient::with_endpoints_device_and_cache(endpoints, auth, device, cache_config)
            .context("failed to build Bitwarden-compatible API client")?;

    Ok(Arc::new(provider))
}

fn endpoints_from_args(args: &Args) -> anyhow::Result<BitwardenEndpoints> {
    match (&args.single_origin_url, &args.identity_url, &args.api_url) {
        (Some(single_origin_url), None, None) => {
            let endpoint = BitwardenEndpoint::parse(single_origin_url).context(
                "invalid single-origin Bitwarden/Vaultwarden endpoint configuration",
            )?;
            Ok(BitwardenEndpoints::from_single_origin(endpoint))
        }
        (None, Some(identity_url), Some(api_url)) => {
            BitwardenEndpoints::parse_split(identity_url, api_url)
                .context("invalid split Bitwarden endpoint configuration")
        }
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            bail!(
                "configure either BWESO_SINGLE_ORIGIN_URL or both BWESO_IDENTITY_URL and BWESO_API_URL, not both endpoint modes"
            )
        }
        _ => bail!(
            "configure BWESO_SINGLE_ORIGIN_URL for single-origin Vaultwarden/self-hosted Bitwarden, or both BWESO_IDENTITY_URL and BWESO_API_URL for split Bitwarden endpoints"
        ),
    }
}

fn build_router(state: AppState) -> Router {
    let middleware_state = state.clone();

    Router::new()
        .route("/livez", get(livez))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .route("/v1/resolve", post(resolve))
        .layer(middleware::from_fn_with_state(
            middleware_state,
            record_http_metrics,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

async fn livez() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    if state.lifecycle.is_ready() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, PROMETHEUS_CONTENT_TYPE)],
        state.metrics.render(state.lifecycle.is_ready()),
    )
}

async fn record_http_metrics(
    State(state): State<AppState>,
    matched_path: Option<MatchedPath>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().as_str().to_string();
    let route = matched_path
        .as_ref()
        .map_or("unmatched", MatchedPath::as_str)
        .to_string();
    let started = Instant::now();
    let response = next.run(request).await;
    let status = response.status();

    state
        .metrics
        .record_http_request(&method, &route, status, started.elapsed());

    response
}

async fn resolve(
    State(state): State<AppState>,
    Json(request): Json<ResolveRequest>,
) -> Result<Json<SecretDocument>, (StatusCode, Json<ErrorResponse>)> {
    let started = Instant::now();
    let selector = match BitwardenSelector::try_from(request.remote_ref) {
        Ok(selector) => selector,
        Err(error) => {
            let (status, error_kind) = provider_status_and_kind(&error);
            state
                .metrics
                .record_resolve_request(status, "error", error_kind, started.elapsed());
            return Err(provider_error(&error));
        }
    };

    match state.provider.resolve(selector).await {
        Ok(document) => {
            state.metrics.record_resolve_request(
                StatusCode::OK,
                "success",
                "none",
                started.elapsed(),
            );
            Ok(Json(document))
        }
        Err(error) => {
            let (status, error_kind) = provider_status_and_kind(&error);
            state
                .metrics
                .record_resolve_request(status, "error", error_kind, started.elapsed());
            Err(provider_error(&error))
        }
    }
}

fn provider_error(error: &BitwardenClientError) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let (status, _) = provider_status_and_kind(error);

    (status, Json(ErrorResponse { error: message }))
}

fn provider_status_and_kind(error: &BitwardenClientError) -> (StatusCode, &'static str) {
    match error {
        BitwardenClientError::Validation(_)
        | BitwardenClientError::Cipher(CipherError::BlankProperty) => {
            (StatusCode::BAD_REQUEST, "validation")
        }
        BitwardenClientError::Cipher(CipherError::MissingProperty { .. })
        | BitwardenClientError::Api(BitwardenApiError::CipherNotFound { .. }) => {
            (StatusCode::NOT_FOUND, "not_found")
        }
        BitwardenClientError::Api(BitwardenApiError::Http(_)) => {
            (StatusCode::BAD_GATEWAY, "upstream_http")
        }
        BitwardenClientError::Api(BitwardenApiError::HttpStatus { .. }) => {
            (StatusCode::BAD_GATEWAY, "upstream_status")
        }
        BitwardenClientError::Crypto(_)
        | BitwardenClientError::Cipher(
            CipherError::Crypto(_) | CipherError::NoExtractableFields { .. },
        ) => (StatusCode::INTERNAL_SERVER_ERROR, "crypto"),
        BitwardenClientError::KeyDerivation(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "key_derivation")
        }
        BitwardenClientError::Api(
            BitwardenApiError::UnsupportedKdfType { .. }
            | BitwardenApiError::MissingKdfParameter { .. },
        ) => (StatusCode::INTERNAL_SERVER_ERROR, "kdf_parameters"),
        BitwardenClientError::Api(
            BitwardenApiError::MissingMasterPasswordUnlock
            | BitwardenApiError::MissingMasterKeyWrappedUserKey
            | BitwardenApiError::MissingCachedSync,
        ) => (StatusCode::INTERNAL_SERVER_ERROR, "sync_payload"),
        BitwardenClientError::Api(BitwardenApiError::InvalidBaseUrl)
        | BitwardenClientError::InvalidEndpoint { .. }
        | BitwardenClientError::InsecureEndpoint => (StatusCode::INTERNAL_SERVER_ERROR, "endpoint"),
        BitwardenClientError::NotImplemented { .. } => {
            (StatusCode::NOT_IMPLEMENTED, "not_implemented")
        }
    }
}

async fn shutdown_signal(lifecycle: Lifecycle) {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to install Ctrl-C signal handler");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "failed to install SIGTERM signal handler");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    lifecycle.mark_shutting_down();
    tracing::info!("shutdown signal received; terminating HTTP server");
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::{to_bytes, Body};
    use bweso_bitwarden::NotImplementedProvider;
    use http::{header, Method, Request};
    use tower::ServiceExt;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    struct StaticProvider;

    #[async_trait]
    impl BitwardenProvider for StaticProvider {
        async fn resolve(
            &self,
            selector: BitwardenSelector,
        ) -> Result<SecretDocument, BitwardenClientError> {
            let data_key = selector.property.unwrap_or_else(|| "value".to_string());

            Ok(SecretDocument::single(data_key, "resolved-secret"))
        }
    }

    fn test_app(provider: Arc<dyn BitwardenProvider>) -> Router {
        build_router(test_state(provider))
    }

    fn test_state(provider: Arc<dyn BitwardenProvider>) -> AppState {
        AppState {
            provider,
            metrics: Arc::new(AppMetrics::new()),
            lifecycle: Lifecycle::default(),
        }
    }

    fn valid_args() -> Args {
        Args {
            listen: SocketAddr::from(([127, 0, 0, 1], 8080)),
            single_origin_url: Some("http://127.0.0.1:8081".to_string()),
            identity_url: None,
            api_url: None,
            client_id: "user.fixture".to_string(),
            client_secret: "super-secret-api-key".to_string(),
            master_password: "super-secret-master-password".to_string(),
            device_identifier: "bweso-test".to_string(),
            device_name: "BWESO Test".to_string(),
            device_type: 22,
            cache_ttl_seconds: 60,
        }
    }

    #[test]
    fn provider_config_rejects_insecure_remote_endpoint() {
        let mut args = valid_args();
        args.single_origin_url = Some("http://vault.example.test".to_string());

        let Some(error) = provider_from_args(args).err() else {
            unreachable!("insecure remote endpoint should fail");
        };
        let error = format!("{error:#}");

        assert!(
            error.contains("invalid single-origin Bitwarden/Vaultwarden endpoint configuration")
        );
        assert!(!error.contains("super-secret-api-key"));
        assert!(!error.contains("super-secret-master-password"));
    }

    #[test]
    fn provider_config_accepts_split_bitwarden_endpoints() {
        let mut args = valid_args();
        args.single_origin_url = None;
        args.identity_url = Some("http://127.0.0.1:8081".to_string());
        args.api_url = Some("http://127.0.0.1:8082".to_string());

        if let Err(error) = provider_from_args(args) {
            unreachable!("local split endpoints should be accepted: {error:#}");
        }
    }

    #[test]
    fn provider_config_rejects_partial_split_endpoints() {
        let mut args = valid_args();
        args.single_origin_url = None;
        args.identity_url = Some("http://127.0.0.1:8081".to_string());

        let Some(error) = provider_from_args(args).err() else {
            unreachable!("partial split endpoint configuration should fail");
        };

        assert!(error
            .to_string()
            .contains("both BWESO_IDENTITY_URL and BWESO_API_URL"));
    }

    #[test]
    fn provider_config_rejects_mixed_endpoint_modes() {
        let mut args = valid_args();
        args.identity_url = Some("https://identity.bitwarden.com".to_string());
        args.api_url = Some("https://api.bitwarden.com".to_string());

        let Some(error) = provider_from_args(args).err() else {
            unreachable!("mixed endpoint modes should fail");
        };

        assert!(error.to_string().contains("not both endpoint modes"));
    }

    #[test]
    fn provider_config_rejects_blank_credentials() {
        let mut args = valid_args();
        args.client_secret = " ".to_string();

        let Some(error) = provider_from_args(args).err() else {
            unreachable!("blank client secret should fail");
        };

        assert!(error
            .to_string()
            .contains("client_secret must not be empty"));
    }

    #[tokio::test]
    async fn livez_and_readyz_return_no_content() -> TestResult {
        let app = test_app(Arc::new(StaticProvider));

        let response = app
            .clone()
            .oneshot(Request::builder().uri("/livez").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let response = app
            .oneshot(Request::builder().uri("/readyz").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        Ok(())
    }

    #[tokio::test]
    async fn readyz_returns_unavailable_while_shutting_down() -> TestResult {
        let state = test_state(Arc::new(StaticProvider));
        state.lifecycle.mark_shutting_down();

        let response = build_router(state)
            .oneshot(Request::builder().uri("/readyz").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        Ok(())
    }

    #[tokio::test]
    async fn resolve_returns_secret_document() -> TestResult {
        let response = test_app(Arc::new(StaticProvider))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/resolve")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"remoteRef":{"key":"app/database","property":"DATABASE_URL"}}"#,
                    ))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let document: SecretDocument = serde_json::from_slice(&body)?;

        assert_eq!(
            document.data.get("DATABASE_URL"),
            Some(&"resolved-secret".to_string())
        );
        Ok(())
    }

    #[tokio::test]
    async fn metrics_exports_redacted_http_and_resolve_series() -> TestResult {
        let app = test_app(Arc::new(StaticProvider));

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/resolve")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"remoteRef":{"key":"app/database","property":"DATABASE_URL"}}"#,
                    ))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(Request::builder().uri("/metrics").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE),
            Some(&header::HeaderValue::from_static(PROMETHEUS_CONTENT_TYPE))
        );

        let body = to_bytes(response.into_body(), usize::MAX).await?;
        let body = std::str::from_utf8(&body)?;

        assert!(body.contains("bweso_build_info{version=\""));
        assert!(body.contains("bweso_ready 1"));
        assert!(body.contains("# TYPE bweso_http_requests_total counter"));
        assert!(body.contains(
            "bweso_http_requests_total{method=\"POST\",route=\"/v1/resolve\",status=\"200\"} 1"
        ));
        assert!(body.contains("# TYPE bweso_resolve_requests_total counter"));
        assert!(body.contains(
            "bweso_resolve_requests_total{outcome=\"success\",error_kind=\"none\",status=\"200\"} 1"
        ));
        assert!(!body.contains("app/database"));
        assert!(!body.contains("DATABASE_URL"));
        Ok(())
    }

    #[tokio::test]
    async fn resolve_rejects_blank_remote_key() -> TestResult {
        let response = test_app(Arc::new(StaticProvider))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/resolve")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"remoteRef":{"key":" "}}"#))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[tokio::test]
    async fn placeholder_provider_returns_not_implemented() -> TestResult {
        let response = test_app(Arc::new(NotImplementedProvider))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/v1/resolve")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"remoteRef":{"key":"app/database"}}"#))?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        Ok(())
    }
}
