//! # In-Memory Venue Repository
//!
//! In-memory implementation of [`VenueRepository`] for testing.
//!
//! This implementation uses a thread-safe `HashMap` for storage,
//! making it suitable for unit tests without database dependencies.

use crate::domain::value_objects::VenueId;
use crate::infrastructure::persistence::traits::{RepositoryResult, VenueRepository};
use crate::infrastructure::venues::registry::VenueConfig;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory implementation of [`VenueRepository`].
///
/// Uses a thread-safe `HashMap` for storage. Suitable for unit tests
/// without database dependencies.
#[derive(Debug, Clone)]
pub struct InMemoryVenueRepository {
    storage: Arc<RwLock<HashMap<VenueId, VenueConfig>>>,
}

impl InMemoryVenueRepository {
    /// Creates a new empty in-memory venue repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the number of venues in the repository.
    #[must_use]
    pub fn len(&self) -> usize {
        self.storage
            .try_read()
            .map(|guard| guard.len())
            .unwrap_or(0)
    }

    /// Returns true if the repository is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all venues from the repository.
    pub async fn clear(&self) {
        let mut storage = self.storage.write().await;
        storage.clear();
    }
}

impl Default for InMemoryVenueRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VenueRepository for InMemoryVenueRepository {
    async fn save(&self, config: &VenueConfig) -> RepositoryResult<()> {
        let mut storage = self.storage.write().await;
        storage.insert(config.venue_id().clone(), config.clone());
        Ok(())
    }

    async fn get(&self, id: &VenueId) -> RepositoryResult<Option<VenueConfig>> {
        let storage = self.storage.read().await;
        Ok(storage.get(id).cloned())
    }

    async fn get_all(&self) -> RepositoryResult<Vec<VenueConfig>> {
        let storage = self.storage.read().await;
        Ok(storage.values().cloned().collect())
    }

    async fn find_enabled(&self) -> RepositoryResult<Vec<VenueConfig>> {
        let storage = self.storage.read().await;
        let enabled: Vec<VenueConfig> = storage
            .values()
            .filter(|c| c.is_enabled())
            .cloned()
            .collect();
        Ok(enabled)
    }

    async fn delete(&self, id: &VenueId) -> RepositoryResult<bool> {
        let mut storage = self.storage.write().await;
        Ok(storage.remove(id).is_some())
    }

    async fn count(&self) -> RepositoryResult<u64> {
        let storage = self.storage.read().await;
        Ok(storage.len() as u64)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_config(venue_id: &str, enabled: bool) -> VenueConfig {
        VenueConfig::new(VenueId::new(venue_id)).with_enabled(enabled)
    }

    #[tokio::test]
    async fn new_repository_is_empty() {
        let repo = InMemoryVenueRepository::new();
        assert!(repo.is_empty());
        assert_eq!(repo.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn save_and_get() {
        let repo = InMemoryVenueRepository::new();
        let config = create_test_config("venue-1", true);
        let id = config.venue_id().clone();

        repo.save(&config).await.unwrap();

        let retrieved = repo.get(&id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().venue_id(), &id);
    }

    #[tokio::test]
    async fn get_nonexistent_returns_none() {
        let repo = InMemoryVenueRepository::new();
        let id = VenueId::new("nonexistent");

        let result = repo.get(&id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_all() {
        let repo = InMemoryVenueRepository::new();

        repo.save(&create_test_config("venue-1", true))
            .await
            .unwrap();
        repo.save(&create_test_config("venue-2", false))
            .await
            .unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn find_enabled() {
        let repo = InMemoryVenueRepository::new();

        repo.save(&create_test_config("venue-1", true))
            .await
            .unwrap();
        repo.save(&create_test_config("venue-2", false))
            .await
            .unwrap();
        repo.save(&create_test_config("venue-3", true))
            .await
            .unwrap();

        let enabled = repo.find_enabled().await.unwrap();
        assert_eq!(enabled.len(), 2);
    }

    #[tokio::test]
    async fn delete() {
        let repo = InMemoryVenueRepository::new();
        let config = create_test_config("venue-1", true);
        let id = config.venue_id().clone();

        repo.save(&config).await.unwrap();
        assert_eq!(repo.count().await.unwrap(), 1);

        let deleted = repo.delete(&id).await.unwrap();
        assert!(deleted);
        assert_eq!(repo.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn clear() {
        let repo = InMemoryVenueRepository::new();

        repo.save(&create_test_config("venue-1", true))
            .await
            .unwrap();
        repo.save(&create_test_config("venue-2", true))
            .await
            .unwrap();
        assert_eq!(repo.count().await.unwrap(), 2);

        repo.clear().await;
        assert_eq!(repo.count().await.unwrap(), 0);
    }
}
