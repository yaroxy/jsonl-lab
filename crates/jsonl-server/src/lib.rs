use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use jsonl_core::{
    JsonParser, JsonlDataset, format_bytes, format_count, load_index, validate_index,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub dataset: Arc<JsonlDataset>,
    pub parser: JsonParser,
}

#[derive(Deserialize)]
struct RangeQuery {
    start: usize,
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct PreviewQuery {
    start: usize,
    limit: Option<usize>,
    #[serde(default)]
    max_bytes: Option<usize>,
}

pub async fn serve(
    path: PathBuf,
    index: PathBuf,
    host: String,
    port: u16,
    parser: JsonParser,
) -> Result<()> {
    let index = load_index(index)?;
    validate_index(&path, &index)?;
    let dataset = JsonlDataset::open(path, index.offsets)?;

    let state = AppState {
        dataset: Arc::new(dataset),
        parser,
    };

    let app = Router::new()
        .route("/api/meta", get(meta))
        .route("/api/item/{idx}", get(item))
        .route("/api/range", get(range))
        .route("/api/range-preview", get(range_preview))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    let meta = json!({
        "path": &state.dataset.path,
        "file_size": state.dataset.file_size,
        "file_size_human": format_bytes(state.dataset.file_size as u64),
        "num_lines": state.dataset.len(),
        "num_lines_human": format_count(state.dataset.len() as u64),
    });
    eprintln!("serving at http://{}", addr);
    eprintln!("  {}", serde_json::to_string(&meta)?);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn meta(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "path": state.dataset.path,
        "file_size": state.dataset.file_size,
        "file_size_human": format_bytes(state.dataset.file_size as u64),
        "num_lines": state.dataset.len(),
        "num_lines_human": format_count(state.dataset.len() as u64),
    }))
}

async fn item(
    State(state): State<AppState>,
    Path(idx): Path<usize>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let value = state
        .dataset
        .json_value_with(idx, state.parser)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(value))
}

async fn range(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(200);

    let mut rows = Vec::new();

    for idx in query.start..query.start.saturating_add(limit).min(state.dataset.len()) {
        let value = state
            .dataset
            .json_value_with(idx, state.parser)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

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

async fn range_preview(
    State(state): State<AppState>,
    Query(query): Query<PreviewQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(20).min(200);
    let max_bytes = query.max_bytes.unwrap_or(512);

    let mut rows = Vec::new();

    for idx in query.start..query.start.saturating_add(limit).min(state.dataset.len()) {
        let line = state
            .dataset
            .raw_line(idx)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        let line_str = String::from_utf8_lossy(line);
        let byte_len = line.len();
        let truncated = if line_str.len() > max_bytes {
            format!("{}...", &line_str[..max_bytes])
        } else {
            line_str.to_string()
        };

        rows.push(json!({
            "idx": idx,
            "byte_len": byte_len,
            "preview": truncated,
        }));
    }

    Ok(Json(json!({
        "start": query.start,
        "limit": limit,
        "rows": rows,
    })))
}
