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
use vwso_vaultwarden::{NotImplementedProvider, VaultwardenProvider, VaultwardenSelector};

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

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/resolve", post(resolve))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    tracing::info!(address = %args.listen, "starting ESO webhook provider");

    axum::serve(listener, app).await?;

    Ok(())
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
    let selector = VaultwardenSelector::try_from(request.remote_ref).map_err(bad_request)?;
    let document = state
        .provider
        .resolve(selector)
        .await
        .map_err(not_implemented)?;

    Ok(Json(document))
}

fn bad_request(error: impl std::error::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}

fn not_implemented(error: impl std::error::Error) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: error.to_string(),
        }),
    )
}
