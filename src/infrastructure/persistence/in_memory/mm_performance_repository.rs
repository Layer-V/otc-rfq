//! # In-Memory MM Performance Repository
//!
//! In-memory implementation of [`MmPerformanceRepository`] for testing
//! without database dependencies.
//!
//! Uses [`DashMap`] for thread-safe concurrent access, keyed by
//! market maker identifier.
//!
//! # Examples
//!
//! ```
//! use otc_rfq::infrastructure::persistence::in_memory::InMemoryMmPerformanceRepository;
//! use otc_rfq::domain::services::mm_performance::MmPerformanceRepository;
//!
//! let repo = InMemoryMmPerformanceRepository::new();
//! ```

use crate::domain::entities::mm_performance::MmPerformanceEvent;
use crate::domain::services::mm_performance::{MmPerformanceRepository, MmPerformanceResult};
use crate::domain::value_objects::CounterpartyId;
use crate::domain::value_objects::timestamp::Timestamp;
use async_trait::async_trait;
use dashmap::DashMap;

/// In-memory implementation of the MM performance repository.
///
/// Stores performance events in a [`DashMap`] keyed by market maker ID.
/// Suitable for testing and development environments.
///
/// # Thread Safety
///
/// This implementation is thread-safe via `DashMap` and can be shared
/// across async tasks.
#[derive(Debug, Default)]
pub struct InMemoryMmPerformanceRepository {
    /// Events stored per market maker ID.
    events: DashMap<String, Vec<MmPerformanceEvent>>,
}

impl InMemoryMmPerformanceRepository {
    /// Creates a new empty in-memory repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of stored events across all market makers.
    #[must_use]
    pub fn total_event_count(&self) -> usize {
        self.events.iter().map(|entry| entry.value().len()).sum()
    }

    /// Returns the number of distinct market makers with recorded events.
    #[must_use]
    pub fn mm_count(&self) -> usize {
        self.events.len()
    }
}

#[async_trait]
impl MmPerformanceRepository for InMemoryMmPerformanceRepository {
    async fn record_event(&self, event: MmPerformanceEvent) -> MmPerformanceResult<()> {
        self.events
            .entry(event.mm_id().to_string())
            .or_default()
            .push(event);
        Ok(())
    }

    async fn get_events(
        &self,
        mm_id: &CounterpartyId,
        from: Timestamp,
        to: Timestamp,
    ) -> MmPerformanceResult<Vec<MmPerformanceEvent>> {
        let key = mm_id.to_string();
        match self.events.get(&key) {
            Some(events) => Ok(events
                .iter()
                .filter(|e| e.is_within_window(from, to))
                .cloned()
                .collect()),
            None => Ok(Vec::new()),
        }
    }

    async fn get_all_mm_ids(&self) -> MmPerformanceResult<Vec<CounterpartyId>> {
        Ok(self
            .events
            .iter()
            .map(|entry| CounterpartyId::new(entry.key().as_str()))
            .collect())
    }

    async fn trim_before(&self, before: Timestamp) -> MmPerformanceResult<u64> {
        let mut removed = 0u64;
        for mut entry in self.events.iter_mut() {
            let initial_len = entry.value().len() as u64;
            entry
                .value_mut()
                .retain(|e| !e.timestamp().is_before(&before));
            let final_len = entry.value().len() as u64;
            removed = removed.saturating_add(initial_len.saturating_sub(final_len));
        }
        Ok(removed)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entities::mm_performance::MmPerformanceEventKind;

    fn mm_id(name: &str) -> CounterpartyId {
        CounterpartyId::new(name)
    }

    fn make_event(
        mm: &CounterpartyId,
        kind: MmPerformanceEventKind,
        ts: Timestamp,
    ) -> MmPerformanceEvent {
        MmPerformanceEvent::new(mm.clone(), kind, ts)
    }

    fn now() -> Timestamp {
        Timestamp::from_secs(1_700_000_000).unwrap()
    }

    mod construction {
        use super::*;

        #[test]
        fn new_creates_empty_repo() {
            let repo = InMemoryMmPerformanceRepository::new();

            assert_eq!(repo.total_event_count(), 0);
            assert_eq!(repo.mm_count(), 0);
        }
    }

    mod record_event {
        use super::*;

        #[tokio::test]
        async fn stores_event_for_mm() {
            let repo = InMemoryMmPerformanceRepository::new();
            let id = mm_id("mm-a");
            let event = make_event(&id, MmPerformanceEventKind::RfqSent, now());

            let result = repo.record_event(event).await;
            assert!(result.is_ok());
            assert_eq!(repo.total_event_count(), 1);
            assert_eq!(repo.mm_count(), 1);
        }

        #[tokio::test]
        async fn stores_multiple_events_for_same_mm() {
            let repo = InMemoryMmPerformanceRepository::new();
            let id = mm_id("mm-b");

            assert!(
                repo.record_event(make_event(&id, MmPerformanceEventKind::RfqSent, now()))
                    .await
                    .is_ok()
            );
            assert!(
                repo.record_event(make_event(
                    &id,
                    MmPerformanceEventKind::QuoteReceived {
                        response_time_ms: 100,
                        rank: 1
                    },
                    now()
                ))
                .await
                .is_ok()
            );

            assert_eq!(repo.total_event_count(), 2);
            assert_eq!(repo.mm_count(), 1);
        }

        #[tokio::test]
        async fn stores_events_for_different_mms() {
            let repo = InMemoryMmPerformanceRepository::new();
            let id1 = mm_id("mm-1");
            let id2 = mm_id("mm-2");

            assert!(
                repo.record_event(make_event(&id1, MmPerformanceEventKind::RfqSent, now()))
                    .await
                    .is_ok()
            );
            assert!(
                repo.record_event(make_event(&id2, MmPerformanceEventKind::RfqSent, now()))
                    .await
                    .is_ok()
            );

            assert_eq!(repo.total_event_count(), 2);
            assert_eq!(repo.mm_count(), 2);
        }
    }

    mod get_events {
        use super::*;

        #[tokio::test]
        async fn returns_empty_for_unknown_mm() {
            let repo = InMemoryMmPerformanceRepository::new();
            let id = mm_id("mm-unknown");

            let events = repo
                .get_events(&id, now().sub_secs(86400), now())
                .await
                .unwrap();
            assert!(events.is_empty());
        }

        #[tokio::test]
        async fn filters_by_window() {
            let repo = InMemoryMmPerformanceRepository::new();
            let id = mm_id("mm-filter");

            // Old event (outside window)
            let old = make_event(
                &id,
                MmPerformanceEventKind::RfqSent,
                Timestamp::from_secs(1_000_000).unwrap(),
            );
            assert!(repo.record_event(old).await.is_ok());

            // Recent event (inside window)
            let recent = make_event(&id, MmPerformanceEventKind::RfqSent, now());
            assert!(repo.record_event(recent).await.is_ok());

            let events = repo
                .get_events(&id, now().sub_secs(86400), now())
                .await
                .unwrap();
            assert_eq!(events.len(), 1);
        }
    }

    mod get_all_mm_ids {
        use super::*;

        #[tokio::test]
        async fn returns_empty_when_no_events() {
            let repo = InMemoryMmPerformanceRepository::new();
            let ids = repo.get_all_mm_ids().await.unwrap();
            assert!(ids.is_empty());
        }

        #[tokio::test]
        async fn returns_all_distinct_ids() {
            let repo = InMemoryMmPerformanceRepository::new();

            assert!(
                repo.record_event(make_event(
                    &mm_id("mm-x"),
                    MmPerformanceEventKind::RfqSent,
                    now()
                ))
                .await
                .is_ok()
            );
            assert!(
                repo.record_event(make_event(
                    &mm_id("mm-y"),
                    MmPerformanceEventKind::RfqSent,
                    now()
                ))
                .await
                .is_ok()
            );
            assert!(
                repo.record_event(make_event(
                    &mm_id("mm-x"),
                    MmPerformanceEventKind::TradeExecuted,
                    now()
                ))
                .await
                .is_ok()
            );

            let ids = repo.get_all_mm_ids().await.unwrap();
            assert_eq!(ids.len(), 2);
        }
    }

    mod trim_before {
        use super::*;

        #[tokio::test]
        async fn removes_old_events() {
            let repo = InMemoryMmPerformanceRepository::new();
            let id = mm_id("mm-trim");

            // Old event
            let old = make_event(
                &id,
                MmPerformanceEventKind::RfqSent,
                Timestamp::from_secs(1_000_000).unwrap(),
            );
            assert!(repo.record_event(old).await.is_ok());

            // Recent event
            let recent = make_event(&id, MmPerformanceEventKind::RfqSent, now());
            assert!(repo.record_event(recent).await.is_ok());

            let removed = repo.trim_before(now().sub_secs(86400)).await.unwrap();
            assert_eq!(removed, 1);
            assert_eq!(repo.total_event_count(), 1);
        }

        #[tokio::test]
        async fn no_events_to_trim() {
            let repo = InMemoryMmPerformanceRepository::new();
            let id = mm_id("mm-none");

            let recent = make_event(&id, MmPerformanceEventKind::RfqSent, now());
            assert!(repo.record_event(recent).await.is_ok());

            let removed = repo
                .trim_before(Timestamp::from_secs(1_000_000).unwrap())
                .await
                .unwrap();
            assert_eq!(removed, 0);
        }

        #[tokio::test]
        async fn trims_across_multiple_mms() {
            let repo = InMemoryMmPerformanceRepository::new();

            let old_ts = Timestamp::from_secs(1_000_000).unwrap();

            assert!(
                repo.record_event(make_event(
                    &mm_id("mm-a"),
                    MmPerformanceEventKind::RfqSent,
                    old_ts
                ))
                .await
                .is_ok()
            );
            assert!(
                repo.record_event(make_event(
                    &mm_id("mm-b"),
                    MmPerformanceEventKind::RfqSent,
                    old_ts
                ))
                .await
                .is_ok()
            );
            assert!(
                repo.record_event(make_event(
                    &mm_id("mm-a"),
                    MmPerformanceEventKind::RfqSent,
                    now()
                ))
                .await
                .is_ok()
            );

            let removed = repo.trim_before(now().sub_secs(86400)).await.unwrap();
            assert_eq!(removed, 2);
            assert_eq!(repo.total_event_count(), 1);
        }
    }
}
