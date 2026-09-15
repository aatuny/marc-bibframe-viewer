mod error;
mod handlers;
mod state;

use std::sync::Arc;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

use crate::handlers::{info, record};
use crate::state::AppState;

const DEFAULT_MAX_CONCURRENT_CONVERSIONS: usize = 2; // i.e., max concurrent xsltproc
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

async fn shutdown_signal() {
    let sigint = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install signal handler");
    };

    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = sigint => {},
        _ = sigterm => {},
    }

    tracing::info!("shutdown signal received")
}

#[tokio::main]
async fn main() -> Result<(), bf_viewer_core::Error> {
    dotenvy::dotenv().ok();

    let bind_addr: String = env_or("BIND_ADDR", DEFAULT_BIND_ADDR.to_string());
    let max_conversions: usize = env_or(
        "MAX_CONCURRENT_CONVERSIONS",
        DEFAULT_MAX_CONCURRENT_CONVERSIONS,
    );

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    // Construct application state from environmental variables
    let state = Arc::new(AppState::from_env(max_conversions)?);

    tracing::info!(
        records = state.index.len(),
        addr = bind_addr,
        "bf-viewer-api started"
    );

    let app = Router::new()
        .route("/info", get(info))
        .route("/record", get(record))
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = TcpListener::bind(bind_addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
