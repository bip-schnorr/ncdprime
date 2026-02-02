use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    // Placeholder for future shared caches.
    // For now we construct compressors per-request.
}

#[derive(Debug, Deserialize)]
struct PairRequest {
    a_b64: String,
    b_b64: String,
    #[serde(default = "default_gzip_level")]
    gzip_level: u32,
}

fn default_gzip_level() -> u32 {
    9
}

#[derive(Debug, Serialize)]
struct PairResponse {
    ncd: f64,
}

#[derive(Debug, Deserialize)]
struct MatrixRequest {
    a: Vec<String>,
    b: Vec<String>,
    #[serde(default = "default_gzip_level")]
    gzip_level: u32,
}

#[derive(Debug, Serialize)]
struct MatrixResponse {
    values: Vec<Vec<f64>>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

async fn health() -> &'static str {
    "ok"
}

fn decode_b64(s: &str) -> anyhow::Result<Vec<u8>> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| anyhow::anyhow!("invalid base64: {e}"))?;
    Ok(bytes)
}

async fn pair(State(_st): State<Arc<AppState>>, Json(req): Json<PairRequest>) -> impl IntoResponse {
    let a = match decode_b64(&req.a_b64) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response();
        }
    };

    let b = match decode_b64(&req.b_b64) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response();
        }
    };

    let c = ncdprime_core::Gzip::new(req.gzip_level);
    match ncdprime_core::ncd(&c, &a, &b, ncdprime_core::NcdOptions::default()) {
        Ok(ncd) => (StatusCode::OK, Json(PairResponse { ncd })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn matrix(
    State(_st): State<Arc<AppState>>,
    Json(req): Json<MatrixRequest>,
) -> impl IntoResponse {
    let mut a_vecs = Vec::with_capacity(req.a.len());
    for s in &req.a {
        match decode_b64(s) {
            Ok(v) => a_vecs.push(v),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("invalid base64 in a: {e}"),
                    }),
                )
                    .into_response();
            }
        }
    }

    let mut b_vecs = Vec::with_capacity(req.b.len());
    for s in &req.b {
        match decode_b64(s) {
            Ok(v) => b_vecs.push(v),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("invalid base64 in b: {e}"),
                    }),
                )
                    .into_response();
            }
        }
    }

    let c = ncdprime_core::Gzip::new(req.gzip_level);
    match ncdprime_core::ncd_matrix(&c, &a_vecs, &b_vecs, ncdprime_core::NcdOptions::default()) {
        Ok(values) => (StatusCode::OK, Json(MatrixResponse { values })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let state = Arc::new(AppState {});

    let app = Router::new()
        .route("/health", get(health))
        .route("/ncd/pair", post(pair))
        .route("/ncd/matrix", post(matrix))
        .with_state(state);

    let addr: SocketAddr = std::env::var("NCDPRIME_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8787".to_string())
        .parse()?;

    tracing::info!("listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
