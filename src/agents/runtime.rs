//! Runtime / Orchestrator
//! 
//! Coordina System 1 → 2 → 3, maneja timeouts y errores.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::agents::system1::{RetrieverConfig, System1Retriever};
use crate::agents::system2::{ReasonerConfig, System2Reasoner};
use crate::agents::system3::{ActorConfig, System3Actor};
use crate::memory::qmd_memory::QmdMemory;

/// Estado de la sesión
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<ConversationMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Response final del agente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub session_id: String,
    pub query: String,
    pub response: String,
    pub confidence: f32,
    pub system_timings: SystemTimings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTimings {
    pub system1_ms: u64,
    pub system2_ms: u64,
    pub system3_ms: u64,
    pub total_ms: u64,
}

/// Configuración del Runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub timeout_seconds: u64,
    pub max_retries: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 2,
        }
    }
}

/// Runtime que orquesta los tres sistemas
pub struct AgentRuntime {
    system1: System1Retriever,
    system2: System2Reasoner,
    system3: System3Actor,
    config: RuntimeConfig,
}

impl AgentRuntime {
    pub fn new(memory: QmdMemory, config: RuntimeConfig) -> Self {
        Self {
            system1: System1Retriever::new(memory, RetrieverConfig::default()),
            system2: System2Reasoner::new(ReasonerConfig::default()),
            system3: System3Actor::new(ActorConfig::default()),
            config,
        }
    }

    /// Ejecuta el ciclo completo: System 1 → System 2 → System 3
    pub async fn run(&self, query: &str, session_id: Option<String>) -> Result<AgentResponse> {
        let start = std::time::Instant::now();
        let session_id = session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        info!("🚀 Starting agent runtime for session: {}", session_id);
        
        // System 1: Retrieval
        let s1_start = std::time::Instant::now();
        let retrieval_result = self.system1.run(query, None).await?;
        let s1_ms = s1_start.elapsed().as_millis() as u64;
        
        debug!("✅ System 1 completed in {}ms", s1_ms);
        
        // System 2: Reasoning  
        let s2_start = std::time::Instant::now();
        let reasoning_result = self.system2.run(query, &retrieval_result).await?;
        let s2_ms = s2_start.elapsed().as_millis() as u64;
        
        debug!("✅ System 2 completed in {}ms", s2_ms);
        
        // System 3: Action
        let s3_start = std::time::Instant::now();
        let action_result = self.system3.run(query, &reasoning_result).await?;
        let s3_ms = s3_start.elapsed().as_millis() as u64;
        
        debug!("✅ System 3 completed in {}ms", s3_ms);
        
        let total_ms = start.elapsed().as_millis() as u64;
        
        info!(
            "✅ Agent runtime completed: {}ms total",
            total_ms
        );
        
        Ok(AgentResponse {
            session_id,
            query: query.to_string(),
            response: action_result.response,
            confidence: reasoning_result.confidence,
            system_timings: SystemTimings {
                system1_ms: s1_ms,
                system2_ms: s2_ms,
                system3_ms: s3_ms,
                total_ms,
            },
        })
    }
}

/// Builder para crear el runtime
pub struct RuntimeBuilder {
    config: RuntimeConfig,
    memory: Option<QmdMemory>,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig::default(),
            memory: None,
        }
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }

    pub fn with_memory(mut self, memory: QmdMemory) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn build(self) -> Result<AgentRuntime> {
        let memory = self
            .memory
            .ok_or_else(|| anyhow::anyhow!("RuntimeBuilder requires a memory backend"))?;
        Ok(AgentRuntime::new(memory, self.config))
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config() {
        let config = RuntimeConfig::default();
        assert_eq!(config.timeout_seconds, 30);
    }
}
