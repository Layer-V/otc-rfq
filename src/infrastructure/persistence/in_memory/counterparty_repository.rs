//! # In-Memory Counterparty Repository
//!
//! In-memory implementation of [`CounterpartyRepository`] for testing.
//!
//! This implementation uses a thread-safe `HashMap` for storage,
//! making it suitable for unit tests without database dependencies.

use crate::domain::entities::counterparty::Counterparty;
use crate::domain::value_objects::CounterpartyId;
use crate::infrastructure::persistence::traits::{CounterpartyRepository, RepositoryResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory implementation of [`CounterpartyRepository`].
///
/// Uses a thread-safe `HashMap` for storage. Suitable for unit tests
/// without database dependencies.
#[derive(Debug, Clone)]
pub struct InMemoryCounterpartyRepository {
    storage: Arc<RwLock<HashMap<CounterpartyId, Counterparty>>>,
}

impl InMemoryCounterpartyRepository {
    /// Creates a new empty in-memory counterparty repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the number of counterparties in the repository.
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

    /// Clears all counterparties from the repository.
    pub async fn clear(&self) {
        let mut storage = self.storage.write().await;
        storage.clear();
    }
}

impl Default for InMemoryCounterpartyRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CounterpartyRepository for InMemoryCounterpartyRepository {
    async fn save(&self, counterparty: &Counterparty) -> RepositoryResult<()> {
        let mut storage = self.storage.write().await;
        storage.insert(counterparty.id().clone(), counterparty.clone());
        Ok(())
    }

    async fn get(&self, id: &CounterpartyId) -> RepositoryResult<Option<Counterparty>> {
        let storage = self.storage.read().await;
        Ok(storage.get(id).cloned())
    }

    async fn get_all(&self) -> RepositoryResult<Vec<Counterparty>> {
        let storage = self.storage.read().await;
        Ok(storage.values().cloned().collect())
    }

    async fn find_active(&self) -> RepositoryResult<Vec<Counterparty>> {
        let storage = self.storage.read().await;
        let active: Vec<Counterparty> = storage
            .values()
            .filter(|c| c.can_trade())
            .cloned()
            .collect();
        Ok(active)
    }

    async fn find_by_name(&self, name: &str) -> RepositoryResult<Vec<Counterparty>> {
        let storage = self.storage.read().await;
        let name_lower = name.to_lowercase();
        let matches: Vec<Counterparty> = storage
            .values()
            .filter(|c| c.name().to_lowercase().contains(&name_lower))
            .cloned()
            .collect();
        Ok(matches)
    }

    async fn delete(&self, id: &CounterpartyId) -> RepositoryResult<bool> {
        let mut storage = self.storage.write().await;
        Ok(storage.remove(id).is_some())
    }

    async fn count(&self) -> RepositoryResult<u64> {
        let storage = self.storage.read().await;
        Ok(storage.len() as u64)
    }

    async fn count_active(&self) -> RepositoryResult<u64> {
        let active = self.find_active().await?;
        Ok(active.len() as u64)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entities::CounterpartyType;

    fn create_test_counterparty(id: &str, name: &str) -> Counterparty {
        // Use Internal type which doesn't require KYC, so can_trade() returns true
        Counterparty::new(CounterpartyId::new(id), name, CounterpartyType::Internal)
    }

    #[tokio::test]
    async fn new_repository_is_empty() {
        let repo = InMemoryCounterpartyRepository::new();
        assert!(repo.is_empty());
        assert_eq!(repo.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn save_and_get() {
        let repo = InMemoryCounterpartyRepository::new();
        let cp = create_test_counterparty("cp-1", "Test Client");
        let id = cp.id().clone();

        repo.save(&cp).await.unwrap();

        let retrieved = repo.get(&id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), &id);
    }

    #[tokio::test]
    async fn get_nonexistent_returns_none() {
        let repo = InMemoryCounterpartyRepository::new();
        let id = CounterpartyId::new("nonexistent");

        let result = repo.get(&id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_all() {
        let repo = InMemoryCounterpartyRepository::new();

        repo.save(&create_test_counterparty("cp-1", "Client One"))
            .await
            .unwrap();
        repo.save(&create_test_counterparty("cp-2", "Client Two"))
            .await
            .unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn find_active() {
        let repo = InMemoryCounterpartyRepository::new();

        // Active counterparty (default is active)
        repo.save(&create_test_counterparty("cp-1", "Active Client"))
            .await
            .unwrap();

        let active = repo.find_active().await.unwrap();
        assert_eq!(active.len(), 1);
    }

    #[tokio::test]
    async fn find_by_name() {
        let repo = InMemoryCounterpartyRepository::new();

        repo.save(&create_test_counterparty("cp-1", "Acme Corp"))
            .await
            .unwrap();
        repo.save(&create_test_counterparty("cp-2", "Acme Trading"))
            .await
            .unwrap();
        repo.save(&create_test_counterparty("cp-3", "Other Company"))
            .await
            .unwrap();

        let acme = repo.find_by_name("acme").await.unwrap();
        assert_eq!(acme.len(), 2);

        let other = repo.find_by_name("other").await.unwrap();
        assert_eq!(other.len(), 1);
    }

    #[tokio::test]
    async fn delete() {
        let repo = InMemoryCounterpartyRepository::new();
        let cp = create_test_counterparty("cp-1", "Test Client");
        let id = cp.id().clone();

        repo.save(&cp).await.unwrap();
        assert_eq!(repo.count().await.unwrap(), 1);

        let deleted = repo.delete(&id).await.unwrap();
        assert!(deleted);
        assert_eq!(repo.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn clear() {
        let repo = InMemoryCounterpartyRepository::new();

        repo.save(&create_test_counterparty("cp-1", "Client One"))
            .await
            .unwrap();
        repo.save(&create_test_counterparty("cp-2", "Client Two"))
            .await
            .unwrap();
        assert_eq!(repo.count().await.unwrap(), 2);

        repo.clear().await;
        assert_eq!(repo.count().await.unwrap(), 0);
    }
}
