//! HTTP handlers for the minimal Cortex vertical slice.

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    memory::belief_graph::{BeliefNode, BeliefRelation},
    memory::qmd_memory::MemoryDocument,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Debug, Deserialize)]
pub struct AgentRunRequest {
    pub query: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub status: String,
    pub results: Vec<serde_json::Value>,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub status: String,
    pub response: String,
    pub confidence: f32,
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub status: String,
    pub nodes: Vec<BeliefNode>,
    pub edges: Vec<BeliefRelation>,
}

#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub status: String,
    pub session_id: String,
    pub response: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub status: String,
    pub synced: usize,
}

fn default_limit() -> usize {
    10
}

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "cortex",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn memory_search(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    info!("🔍 Search request: {}", payload.query);

    let results = state
        .memory
        .search(&payload.query, payload.limit)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|doc: MemoryDocument| {
            let id = doc
                .id
                .map(|record_id| format!("{}:{:?}", record_id.table.as_str(), record_id.key));
            serde_json::json!({
                "id": id,
                "path": doc.path,
                "content": doc.content,
                "metadata": doc.metadata,
            })
        })
        .collect();

    Json(SearchResponse {
        status: "ok".to_string(),
        results,
        query: payload.query,
    })
}

pub async fn memory_query(
    State(state): State<AppState>,
    Json(payload): Json<SearchRequest>,
) -> impl IntoResponse {
    info!("🧠 Query request: {}", payload.query);

    let response = state.runtime.run(&payload.query, None).await;

    match response {
        Ok(result) => Json(QueryResponse {
            status: "ok".to_string(),
            response: result.response,
            confidence: result.confidence,
            session_id: result.session_id,
        }),
        Err(error) => Json(QueryResponse {
            status: format!("error: {}", error),
            response: String::new(),
            confidence: 0.0,
            session_id: String::new(),
        }),
    }
}

pub async fn memory_graph(State(state): State<AppState>) -> impl IntoResponse {
    info!("🔗 Graph request");

    let graph = state.belief_graph.read().await;

    Json(GraphResponse {
        status: "ok".to_string(),
        nodes: graph.get_nodes().into_iter().cloned().collect(),
        edges: graph.get_relations().to_vec(),
    })
}

pub async fn agents_run(
    State(state): State<AppState>,
    Json(payload): Json<AgentRunRequest>,
) -> impl IntoResponse {
    info!("🤖 Agent run request: {}", payload.query);

    let response = state.runtime.run(&payload.query, payload.session_id).await;

    match response {
        Ok(result) => Json(AgentResponse {
            status: "ok".to_string(),
            session_id: result.session_id,
            response: result.response,
            confidence: result.confidence,
        }),
        Err(error) => Json(AgentResponse {
            status: format!("error: {}", error),
            session_id: String::new(),
            response: String::new(),
            confidence: 0.0,
        }),
    }
}

pub async fn sync_tier1(State(state): State<AppState>) -> impl IntoResponse {
    info!("🔄 Tier 1 sync request");

    let synced = state.memory.count().await.unwrap_or(0);

    Json(SyncResponse {
        status: "ok".to_string(),
        synced,
    })
}
