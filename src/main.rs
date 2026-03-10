use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use surrealdb::{
    engine::any::connect,
    opt::auth::Root,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod agents;
mod memory;
mod server;
mod tools;

use agents::{AgentRuntime, RuntimeConfig};
use memory::{belief_graph::SharedBeliefGraph, qmd_memory::QmdMemory};

#[derive(Clone)]
pub(crate) struct AppState {
    pub memory: QmdMemory,
    pub runtime: Arc<AgentRuntime>,
    pub belief_graph: SharedBeliefGraph,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Cortex - Cognitive Memory System");

    let db = connect("ws://localhost:8000").await?;
    db.signin(Root {
        username: "root".to_string(),
        password: "root".to_string(),
    })
    .await?;
    db.use_ns("agentrag").use_db("system3").await?;

    let memory = QmdMemory::new(Arc::new(db));
    memory.init().await?;

    let runtime = Arc::new(AgentRuntime::new(memory.clone(), RuntimeConfig::default()));
    let belief_graph = Arc::new(tokio::sync::RwLock::new(
        memory::belief_graph::BeliefGraph::new(),
    ));

    let state = AppState {
        memory,
        runtime,
        belief_graph,
    };

    let app = Router::new()
        .route("/health", get(server::http::health))
        .route("/memory/search", post(server::http::memory_search))
        .route("/memory/query", post(server::http::memory_query))
        .route("/memory/graph", get(server::http::memory_graph))
        .route("/agents/run", post(server::http::agents_run))
        .route("/sync", post(server::http::sync_tier1))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8003));
    tracing::info!("Cortex HTTP server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
