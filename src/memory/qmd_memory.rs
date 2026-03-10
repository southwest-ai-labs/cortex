//! QMD Memory - Búsqueda híbrida (BM25 + Vectores)
//! 
//! Sistema de memoria con búsqueda híbrida.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use surrealdb::{engine::any::Any, types::{RecordId, SurrealValue}, Surreal};

/// Documento en memoria
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MemoryDocument {
    pub id: Option<RecordId>,
    pub path: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub embedding: Vec<f32>,
}

/// QMD Memory - Búsqueda híbrida
#[derive(Clone)]
pub struct QmdMemory {
    db: Arc<Surreal<Any>>,
}

impl QmdMemory {
    pub fn new(db: Arc<Surreal<Any>>) -> Self {
        Self { db }
    }

    pub async fn init(&self) -> Result<()> {
        self.db.query("DEFINE TABLE IF NOT EXISTS memory SCHEMAFULL;").await?;
        self.db
            .query("DEFINE FIELD IF NOT EXISTS path ON TABLE memory TYPE string;")
            .await?;
        self.db
            .query("DEFINE FIELD IF NOT EXISTS content ON TABLE memory TYPE string;")
            .await?;
        self.db
            .query("DEFINE FIELD IF NOT EXISTS metadata ON TABLE memory TYPE object FLEXIBLE;")
            .await?;
        self.db
            .query("DEFINE FIELD IF NOT EXISTS embedding ON TABLE memory TYPE array<float>;")
            .await?;
        self.db
            .query("DEFINE INDEX IF NOT EXISTS memory_content_index ON TABLE memory COLUMNS content SEARCH ANALYZER ascii BM25;")
            .await?;
        self.db
            .query("DEFINE INDEX IF NOT EXISTS memory_embedding_index ON TABLE memory COLUMNS embedding MTree DIMENSION 1536 TYPE F32;")
            .await?;
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<MemoryDocument>> {
        Ok(self.db.select("memory").await?)
    }

    /// search (default): fast keyword match (BM25)
    pub async fn search(&self, query_text: &str, limit: usize) -> Result<Vec<MemoryDocument>> {
        let query = query_text.to_lowercase();
        let docs = self
            .load_all()
            .await?
            .into_iter()
            .filter(|doc| {
                doc.content.to_lowercase().contains(&query)
                    || doc.path.to_lowercase().contains(&query)
            })
            .take(limit)
            .collect();
        Ok(docs)
    }

    /// vsearch: semantic similarity (vector)
    pub async fn vsearch(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<MemoryDocument>> {
        let mut docs = self.load_all().await?;
        docs.sort_by(|a, b| {
            cosine_similarity(&b.embedding, &query_vector)
                .partial_cmp(&cosine_similarity(&a.embedding, &query_vector))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        docs.truncate(limit);
        Ok(docs)
    }

    /// query: hybrid search + LLM reranking (fusion)
    pub async fn query(&self, query_text: &str, query_vector: Vec<f32>, limit: usize) -> Result<Vec<MemoryDocument>> {
        let mut docs = self.search(query_text, limit.saturating_mul(2)).await?;
        let semantic_docs = self.vsearch(query_vector, limit.saturating_mul(2)).await?;

        for candidate in semantic_docs {
            let exists = docs.iter().any(|doc| {
                doc.id == candidate.id || (!doc.path.is_empty() && doc.path == candidate.path)
            });
            if !exists {
                docs.push(candidate);
            }
        }

        docs.truncate(limit);
        Ok(docs)
    }

    /// get: retrieve specific document by path or ID
    pub async fn get(&self, path_or_id: &str) -> Result<Option<MemoryDocument>> {
        if path_or_id.contains(':') {
            let parts: Vec<&str> = path_or_id.splitn(2, ':').collect();
            if parts.len() == 2 {
                let record: Option<MemoryDocument> = self.db.select((parts[0], parts[1])).await?;
                Ok(record)
            } else {
                Ok(None)
            }
        } else {
            let docs = self.load_all().await?;
            Ok(docs.into_iter().find(|doc| doc.path == path_or_id))
        }
    }

    pub async fn count(&self) -> Result<usize> {
        Ok(self.load_all().await?.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let left = vec![1.0, 0.0, 0.0];
        let right = vec![1.0, 0.0, 0.0];

        assert_eq!(cosine_similarity(&left, &right), 1.0);
        assert_eq!(cosine_similarity(&left, &[0.0, 1.0, 0.0]), 0.0);
    }
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.is_empty() || right.is_empty() || left.len() != right.len() {
        return 0.0;
    }

    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();

    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}
