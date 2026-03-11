//! Belief Graph - Grafo de relaciones conceptuales
//!
//! Almacena relaciones entre conceptos para razonamiento sobre conexiones.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Un nodo en el grafo de creencias
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefNode {
    pub id: String,
    pub concept: String,
    pub confidence: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Una relación entre nodos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefRelation {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation_type: String,
    pub weight: f32,
}

/// Belief Graph - Grafo de relaciones conceptuales
pub struct BeliefGraph {
    nodes: HashMap<String, BeliefNode>,
    relations: Vec<BeliefRelation>,
    #[allow(dead_code)]
    adjacency: HashMap<String, HashSet<String>>,
}

#[allow(dead_code)]
impl BeliefGraph {
    /// Crea un nuevo belief graph vacío
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            relations: Vec::new(),
            adjacency: HashMap::new(),
        }
    }

    /// Añade un nodo al grafo
    pub fn add_node(&mut self, concept: String, confidence: f32) {
        let id = uuid::Uuid::new_v4().to_string();

        let node = BeliefNode {
            id: id.clone(),
            concept: concept.clone(),
            confidence,
            created_at: chrono::Utc::now(),
        };

        self.nodes.insert(id, node);

        // Ensure adjacency entry exists
        self.adjacency.entry(concept.clone()).or_default();

        info!("Added node: {}", concept);
    }

    /// Añade una relación entre nodos
    pub fn add_relation(
        &mut self,
        source: String,
        target: String,
        relation_type: String,
        weight: f32,
    ) {
        let id = uuid::Uuid::new_v4().to_string();

        let relation = BeliefRelation {
            id: id.clone(),
            source: source.clone(),
            target: target.clone(),
            relation_type: relation_type.clone(),
            weight,
        };

        self.relations.push(relation);

        // Update adjacency
        self.adjacency
            .entry(source.clone())
            .or_default()
            .insert(target.clone());
        self.adjacency.entry(target.clone()).or_default();

        info!(
            "Added relation: {} -> {} ({})",
            source, target, relation_type
        );
    }

    /// Obtiene nodos relacionados con un concepto
    pub fn get_related(&self, concept: &str) -> Vec<String> {
        self.adjacency
            .get(concept)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Obtiene un nodo por concepto
    pub fn get_node(&self, concept: &str) -> Option<&BeliefNode> {
        self.nodes.values().find(|n| n.concept == concept)
    }

    /// Obtiene todas las relaciones
    pub fn get_relations(&self) -> &[BeliefRelation] {
        &self.relations
    }

    /// Obtiene todos los nodos
    pub fn get_nodes(&self) -> Vec<&BeliefNode> {
        self.nodes.values().collect()
    }

    /// Actualiza la confianza de un nodo
    pub fn update_confidence(&mut self, concept: &str, new_confidence: f32) {
        if let Some(node) = self.nodes.values_mut().find(|n| n.concept == concept) {
            node.confidence = new_confidence;
            info!("Updated confidence for {}: {}", concept, new_confidence);
        }
    }

    /// Consulta el grafo - versión async placeholder
    pub async fn query(&self, query: &str) -> Result<Vec<BeliefNode>> {
        // Simple implementation - find nodes matching query
        let query_lower = query.to_lowercase();

        let results: Vec<BeliefNode> = self
            .nodes
            .values()
            .filter(|n| n.concept.to_lowercase().contains(&query_lower))
            .cloned()
            .collect();

        Ok(results)
    }

    /// Serializa el grafo a JSON
    pub fn to_json(&self) -> Result<String> {
        let data = serde_json::json!({
            "nodes": self.nodes.values().collect::<Vec<_>>(),
            "relations": self.relations,
        });

        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// Carga el grafo desde JSON
    pub fn from_json(json: &str) -> Result<Self> {
        #[derive(Deserialize)]
        struct GraphData {
            nodes: Vec<BeliefNode>,
            relations: Vec<BeliefRelation>,
        }

        let data: GraphData = serde_json::from_str(json)?;

        let mut graph = Self::new();

        for node in data.nodes {
            let concept = node.concept.clone();
            graph.nodes.insert(node.id.clone(), node);
            graph.adjacency.entry(concept).or_default();
        }

        for relation in data.relations {
            graph
                .adjacency
                .entry(relation.source.clone())
                .or_default()
                .insert(relation.target.clone());
            graph.relations.push(relation);
        }

        Ok(graph)
    }
}

impl Default for BeliefGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Versión thread-safe del BeliefGraph
pub type SharedBeliefGraph = Arc<RwLock<BeliefGraph>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut graph = BeliefGraph::new();
        graph.add_node("rust".to_string(), 0.9);

        assert!(graph.get_node("rust").is_some());
    }

    #[test]
    fn test_add_relation() {
        let mut graph = BeliefGraph::new();
        graph.add_node("rust".to_string(), 0.9);
        graph.add_node("performance".to_string(), 0.8);
        graph.add_relation(
            "rust".to_string(),
            "performance".to_string(),
            "enhances".to_string(),
            0.7,
        );

        let related = graph.get_related("rust");
        assert!(related.contains(&"performance".to_string()));
    }

    #[test]
    fn test_serialization() {
        let mut graph = BeliefGraph::new();
        graph.add_node("test".to_string(), 0.5);

        let json = graph.to_json().unwrap();
        let loaded = BeliefGraph::from_json(&json).unwrap();

        assert!(loaded.get_node("test").is_some());
    }
}
