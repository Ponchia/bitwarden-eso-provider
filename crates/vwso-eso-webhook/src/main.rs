#![forbid(unsafe_code)]

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use vwso_core::{RemoteRef, SecretDocument};
use vwso_vaultwarden::{
    CipherError, NotImplementedProvider, VaultwardenClientError, VaultwardenProvider,
    VaultwardenSelector,
};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(long, env = "VWSO_LISTEN", default_value = "0.0.0.0:8080")]
    listen: SocketAddr,
}

#[derive(Clone)]
struct AppState {
    provider: Arc<dyn VaultwardenProvider>,
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

    let state = AppState {
        provider: Arc::new(NotImplementedProvider),
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    tracing::info!(address = %args.listen, "starting ESO webhook provider");

    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/resolve", post(resolve))
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

async fn healthz() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

async fn resolve(
    State(state): State<AppState>,
    Json(request): Json<ResolveRequest>,
) -> Result<Json<SecretDocument>, (StatusCode, Json<ErrorResponse>)> {
    let selector = VaultwardenSelector::try_from(request.remote_ref)
        .map_err(|error| provider_error(&error))?;
    let document = state
        .provider
        .resolve(selector)
        .await
        .map_err(|error| provider_error(&error))?;

    Ok(Json(document))
}

fn provider_error(error: &VaultwardenClientError) -> (StatusCode, Json<ErrorResponse>) {
    let message = error.to_string();
    let status = match error {
        VaultwardenClientError::Validation(_)
        | VaultwardenClientError::Cipher(CipherError::BlankProperty) => StatusCode::BAD_REQUEST,
        VaultwardenClientError::Cipher(CipherError::MissingProperty { .. }) => {
            StatusCode::NOT_FOUND
        }
        VaultwardenClientError::Crypto(_)
        | VaultwardenClientError::Cipher(CipherError::Crypto(_)) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
        VaultwardenClientError::NotImplemented { .. } => StatusCode::NOT_IMPLEMENTED,
        VaultwardenClientError::InvalidEndpoint { .. }
        | VaultwardenClientError::InsecureEndpoint => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (status, Json(ErrorResponse { error: message }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::{to_bytes, Body};
    use http::{header, Method, Request};
    use tower::ServiceExt;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    struct StaticProvider;

    #[async_trait]
    impl VaultwardenProvider for StaticProvider {
        async fn resolve(
            &self,
            selector: VaultwardenSelector,
        ) -> Result<SecretDocument, VaultwardenClientError> {
            let data_key = selector.property.unwrap_or_else(|| "value".to_string());

            Ok(SecretDocument::single(data_key, "resolved-secret"))
        }
    }

    fn test_app(provider: Arc<dyn VaultwardenProvider>) -> Router {
        build_router(AppState { provider })
    }

    #[tokio::test]
    async fn healthz_returns_no_content() -> TestResult {
        let response = test_app(Arc::new(StaticProvider))
            .oneshot(Request::builder().uri("/healthz").body(Body::empty())?)
            .await?;

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
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
