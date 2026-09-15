use std::sync::Arc;
use std::time::Duration;

use axum::{
    Json,
    extract::{Query, State},
};

use bf_viewer_core::{
    preprocess_collection, rdfxml_to_triples, rdfxml_to_turtle, read_records_as_collection,
    to_bibframe,
};

use crate::error::AppError;
use crate::state::AppState;

const CONVERSION_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(serde::Serialize)]
pub struct XmlInfoResponse {
    records: usize,
    size: Option<u64>,
}

#[derive(serde::Deserialize)]
pub struct RecordsQuery {
    #[serde(default)]
    record_index: Option<usize>,
    key: Option<String>,
}

#[derive(serde::Serialize)]
pub struct RecordResponse {
    key: String,
    record_index: usize,
    has_next: bool,
    marc: String,
    bibframe_xml: String,
    turtle: String,
    triples: Vec<bf_viewer_core::Triple>,
}

pub async fn info(State(state): State<Arc<AppState>>) -> Json<XmlInfoResponse> {
    Json(XmlInfoResponse {
        records: state.index.len(),
        size: state.file.metadata().ok().map(|m| m.len()),
    })
}

pub async fn record(
    Query(q): Query<RecordsQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<RecordResponse>, AppError> {
    let idx = match (q.record_index, &q.key) {
        (None, None) => 0,                                                 // no params
        (Some(i), None) => i,                                              // record index
        (None, Some(n)) => *state.by_key.get(n).ok_or(AppError::NotFoud)?, // key (f003+f001)
        _ => return Err(AppError::BadRequest),
    };

    let entry = state.index.get(idx).ok_or(AppError::NotFoud)?;

    // Note: currently this endpoint wraps just a single record to a collection
    // However current core functionality would allow having endpoint for multiple record ids also
    let marc_collection = read_records_as_collection(&state.file, std::slice::from_ref(entry))?;

    // Manage semaphore permits - 503 if exceeded
    let _permit = tokio::time::timeout(CONVERSION_TIMEOUT, state.semaphore.acquire())
        .await
        .map_err(|e| {
            tracing::warn!(?e, "semaphore permit timed out");
            AppError::Busy
        })?
        .map_err(|_| AppError::Busy)?;

    let preprocessed = preprocess_collection(&marc_collection, &state.xsl_dir)
        .await
        .map_err(|e| {
            tracing::error!(?e, record_index = idx, "marc preprocessing failed");
            AppError::Internal(e)
        })?;

    let bibframe_xml = to_bibframe(&preprocessed, &state.xsl_dir)
        .await
        .map_err(|e| {
            tracing::error!(?e, record_index = idx, "bibframe conversion failed");
            AppError::Internal(e)
        })?;

    let triples = rdfxml_to_triples(&bibframe_xml).map_err(|e| {
        tracing::error!(?e, record_index = idx, "generating triples failed");
        AppError::Internal(e)
    })?;

    let turtle = rdfxml_to_turtle(&bibframe_xml).map_err(|e| {
        tracing::error!(?e, record_index = idx, "generating turtle failed");
        AppError::Internal(e)
    })?;

    Ok(Json(RecordResponse {
        key: entry.key.clone(),
        record_index: idx,
        has_next: idx + 1 < state.index.len(),
        marc: marc_collection,
        bibframe_xml,
        triples,
        turtle,
    }))
}
