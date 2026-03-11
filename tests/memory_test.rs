//! Memory Module Tests - Unit and Integration Tests

#[cfg(test)]
mod memory_manager_tests {
    use cortex::memory::manager::{MemoryManager, MemoryMetrics, MemoryAction};
    use cortex::memory::qmd_memory::QmdMemory;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::collections::HashMap;

    // Mock QmdMemory for testing
    #[derive(Clone)]
    struct MockQmdMemory {
        documents: Arc<RwLock<HashMap<String, String>>>,
    }

    impl MockQmdMemory {
        fn new() -> Self {
            Self {
                documents: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl QmdMemory for MockQmdMemory {
        async fn add(&self, id: String, content: String) -> anyhow::Result<()> {
            self.documents.write().await.insert(id, content);
            Ok(())
        }

        async fn get(&self, id: &str) -> anyhow::Result<Option<String>> {
            Ok(self.documents.read().await.get(id).cloned())
        }

        async fn search(&self, query: &str, _limit: usize) -> anyhow::Result<Vec<(String, f64)>> {
            let docs = self.documents.read().await;
            let results: Vec<(String, f64)> = docs
                .iter()
                .filter(|(_, v)| v.contains(query))
                .map(|(k, _)| (k.clone(), 0.9))
                .collect();
            Ok(results)
        }

        async fn delete(&self, id: &str) -> anyhow::Result<()> {
            self.documents.write().await.remove(id);
            Ok(())
        }

        async fn count(&self) -> anyhow::Result<usize> {
            Ok(self.documents.read().await.len())
        }

        async fn list(&self, _prefix: Option<&str>) -> anyhow::Result<Vec<(String, String)>> {
            Ok(self.documents.read().await.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
        }
    }

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let mock = MockQmdMemory::new();
        let manager = MemoryManager::new(Arc::new(mock));
        
        assert_eq!(manager.max_documents, 1000);
        assert_eq!(manager.max_age_hours, 24.0 * 7.0);
    }

    #[tokio::test]
    async fn test_get_metrics_empty() {
        let mock = MockQmdMemory::new();
        let manager = MemoryManager::new(Arc::new(mock));
        
        let metrics = manager.get_metrics().await.unwrap();
        assert_eq!(metrics.total_documents, 0);
    }

    #[tokio::test]
    async fn test_analyze_and_recommend_empty() {
        let mock = MockQmdMemory::new();
        let manager = MemoryManager::new(Arc::new(mock));
        
        let actions = manager.analyze_and_recommend().await.unwrap();
        assert!(actions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_recommends_consolidation() {
        let mock = MockQmdMemory::new();
        
        // Add many documents to trigger consolidation
        for i in 0..1500 {
            mock.add(format!("doc_{}", i), format!("content {}", i)).await.unwrap();
        }
        
        let manager = MemoryManager::new(Arc::new(mock));
        let actions = manager.analyze_and_recommend().await.unwrap();
        
        assert!(!actions.is_empty());
        assert!(matches!(actions[0], MemoryAction::Consolidate { .. }));
    }
}

#[cfg(test)]
mod memory_metrics_tests {
    use cortex::memory::manager::MemoryMetrics;

    #[test]
    fn test_memory_metrics_creation() {
        let metrics = MemoryMetrics {
            total_documents: 100,
            total_size_bytes: 50000,
            oldest_document_age_hours: 48.0,
            newest_document_age_hours: 2.0,
            average_relevance: 0.75,
        };
        
        assert_eq!(metrics.total_documents, 100);
        assert_eq!(metrics.total_size_bytes, 50000);
        assert!(metrics.average_relevance > 0.0 && metrics.average_relevance <= 1.0);
    }
}

#[cfg(test)]
mod memory_action_tests {
    use cortex::memory::manager::MemoryAction;

    #[test]
    fn test_memory_action_variants() {
        let keep = MemoryAction::Keep;
        assert!(matches!(keep, MemoryAction::Keep));
        
        let compress = MemoryAction::Compress {
            doc_id: "doc1".to_string(),
            reason: "redundant".to_string(),
        };
        assert!(matches!(compress, MemoryAction::Compress { .. }));
        
        let delete = MemoryAction::Delete {
            doc_id: "doc2".to_string(),
            reason: "outdated".to_string(),
        };
        assert!(matches!(delete, MemoryAction::Delete { .. }));
        
        let update = MemoryAction::Update {
            doc_id: "doc3".to_string(),
            new_content: "new content".to_string(),
        };
        assert!(matches!(update, MemoryAction::Update { .. }));
        
        let consolidate = MemoryAction::Consolidate {
            doc_ids: vec!["doc4".to_string()],
            reason: "merge".to_string(),
        };
        assert!(matches!(consolidate, MemoryAction::Consolidate { .. }));
    }
}

// Integration tests require actual database connection
#[cfg(test)]
#[ignore]
mod integration_tests {
    #[tokio::test]
    async fn test_surreal_integration() {
        // This test requires actual SurrealDB instance
        // Use testcontainers or embedded DB
        todo!("Implement with testcontainers");
    }
}
