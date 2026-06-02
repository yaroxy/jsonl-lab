use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use jsonl_core::{JsonlDataset, load_index, validate_index};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    dataset: Arc<JsonlDataset>,
}

#[derive(Deserialize)]
struct RangeQuery {
    start: usize,
    limit: Option<usize>,
}

pub async fn serve(path: PathBuf, index: PathBuf, host: String, port: u16) -> Result<()> {
    let index = load_index(index)?;
    validate_index(&path, &index)?;
    let dataset = JsonlDataset::open(path, index.offsets)?;

    let state = AppState {
        dataset: Arc::new(dataset),
    };

    let app = Router::new()
        .route("/api/meta", get(meta))
        .route("/api/item/{idx}", get(item))
        .route("/api/range", get(range))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    eprintln!("serving at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn meta(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "path": state.dataset.path,
        "file_size": state.dataset.file_size,
        "num_lines": state.dataset.len(),
    }))
}

async fn item(
    State(state): State<AppState>,
    Path(idx): Path<usize>,
) -> Result<Json<serde_json::Value>, String> {
    let value = state.dataset.json_value(idx).map_err(|e| e.to_string())?;

    Ok(Json(value))
}

async fn range(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<serde_json::Value>, String> {
    let limit = query.limit.unwrap_or(20).min(200);

    let mut rows = Vec::new();

    for idx in query.start..query.start.saturating_add(limit).min(state.dataset.len()) {
        let value = state.dataset.json_value(idx).map_err(|e| e.to_string())?;

        rows.push(json!({
            "idx": idx,
            "value": value,
        }));
    }

    Ok(Json(json!({
        "start": query.start,
        "limit": limit,
        "rows": rows,
    })))
}
