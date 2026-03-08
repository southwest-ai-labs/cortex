use serde::{Deserialize, Serialize};
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryDocument {
    pub id: Option<surrealdb::sql::Thing>,
    pub path: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub embedding: Vec<f32>,
}

pub struct QmdMemory {
    db: Surreal<Any>,
}

impl QmdMemory {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// search (default): fast keyword match (BM25)
    pub async fn search(&self, query_text: &str, limit: usize) -> Result<Vec<MemoryDocument>> {
        let mut response = self
            .db
            .query("SELECT * FROM memory WHERE content @@ $query LIMIT $limit")
            .bind(("query", query_text))
            .bind(("limit", limit))
            .await?;
        let docs: Vec<MemoryDocument> = response.take(0)?;
        Ok(docs)
    }

    /// vsearch: semantic similarity (vector)
    pub async fn vsearch(&self, query_vector: Vec<f32>, limit: usize) -> Result<Vec<MemoryDocument>> {
        let mut response = self
            .db
            .query("SELECT * FROM memory ORDER BY vector::similarity::cosine(embedding, $query_vector) DESC LIMIT $limit")
            .bind(("query_vector", query_vector))
            .bind(("limit", limit))
            .await?;
        let docs: Vec<MemoryDocument> = response.take(0)?;
        Ok(docs)
    }

    /// query: hybrid search + LLM reranking (fusion)
    pub async fn query(
        &self,
        query_text: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<MemoryDocument>> {
        let mut response = self
            .db
            .query(
                "SELECT *, search::score(1) AS bm25_score, vector::similarity::cosine(embedding, $query_vector) AS vector_score FROM memory WHERE content @@ $query_text ORDER BY vector_score DESC LIMIT $limit"
            )
            .bind(("query_text", query_text))
            .bind(("query_vector", query_vector))
            .bind(("limit", limit))
            .await?;
        let docs: Vec<MemoryDocument> = response.take(0)?;
        Ok(docs)
    }

    /// get: retrieve specific document by path or ID
    pub async fn get(&self, path_or_id: &str) -> Result<Option<MemoryDocument>> {
        if path_or_id.contains(':') {
            // Treat as ID
            let mut response = self
                .db
                .query("SELECT * FROM type::thing($id)")
                .bind(("id", path_or_id))
                .await?;
            let mut docs: Vec<MemoryDocument> = response.take(0)?;
            Ok(docs.pop())
        } else {
            // Treat as path
            let mut response = self
                .db
                .query("SELECT * FROM memory WHERE path = $path LIMIT 1")
                .bind(("path", path_or_id))
                .await?;
            let mut docs: Vec<MemoryDocument> = response.take(0)?;
            Ok(docs.pop())
        }
    }
}
