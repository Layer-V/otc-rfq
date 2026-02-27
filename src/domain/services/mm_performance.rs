//! # Market Maker Performance Service
//!
//! Domain service for tracking and computing market maker performance metrics.
//!
//! The [`MmPerformanceTracker`] records individual performance events and
//! computes aggregated metrics over a configurable rolling window (default 7 days).
//!
//! # Examples
//!
//! ```ignore
//! use otc_rfq::domain::services::mm_performance::MmPerformanceTracker;
//! use std::sync::Arc;
//!
//! let repo: Arc<dyn MmPerformanceRepository> = /* ... */;
//! let tracker = MmPerformanceTracker::new(repo, 7);
//!
//! tracker.record_rfq_sent(&mm_id).await?;
//! let metrics = tracker.get_metrics(&mm_id).await?;
//! ```

use crate::domain::entities::mm_performance::{
    DEFAULT_WINDOW_DAYS, MmPerformanceEvent, MmPerformanceEventKind, MmPerformanceMetrics,
};
use crate::domain::value_objects::CounterpartyId;
use crate::domain::value_objects::timestamp::Timestamp;
use async_trait::async_trait;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;

/// Error type for MM performance operations.
#[derive(Debug, Error)]
pub enum MmPerformanceError {
    /// Repository error.
    #[error("repository error: {0}")]
    Repository(String),

    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

impl MmPerformanceError {
    /// Creates a repository error.
    #[must_use]
    pub fn repository(msg: impl Into<String>) -> Self {
        Self::Repository(msg.into())
    }

    /// Creates an invalid configuration error.
    #[must_use]
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }
}

/// Result type for MM performance operations.
pub type MmPerformanceResult<T> = Result<T, MmPerformanceError>;

/// Repository trait for storing and retrieving MM performance events.
///
/// Implementations may use in-memory storage, Redis, PostgreSQL, or any
/// other backend suitable for time-series event data.
#[async_trait]
pub trait MmPerformanceRepository: Send + Sync + fmt::Debug {
    /// Records a performance event for a market maker.
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if the event cannot be stored.
    async fn record_event(&self, event: MmPerformanceEvent) -> MmPerformanceResult<()>;

    /// Retrieves all events for a market maker within a time window.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    /// * `from` - Start of the time window (inclusive)
    /// * `to` - End of the time window (inclusive)
    async fn get_events(
        &self,
        mm_id: &CounterpartyId,
        from: Timestamp,
        to: Timestamp,
    ) -> MmPerformanceResult<Vec<MmPerformanceEvent>>;

    /// Returns all distinct market maker IDs that have recorded events.
    async fn get_all_mm_ids(&self) -> MmPerformanceResult<Vec<CounterpartyId>>;

    /// Trims events older than the given timestamp for all market makers.
    ///
    /// # Arguments
    ///
    /// * `before` - Remove events with timestamps strictly before this value
    ///
    /// # Returns
    ///
    /// The number of events removed.
    async fn trim_before(&self, before: Timestamp) -> MmPerformanceResult<u64>;
}

/// Service for tracking market maker performance over a rolling window.
///
/// Records individual events (RFQ sent, quote received, trade executed,
/// last-look reject) and computes aggregated metrics per market maker.
///
/// # Examples
///
/// ```ignore
/// use otc_rfq::domain::services::mm_performance::MmPerformanceTracker;
///
/// let tracker = MmPerformanceTracker::new(repo, 7);
///
/// // Record events
/// tracker.record_rfq_sent(&mm_id).await?;
/// tracker.record_quote_received(&mm_id, 150, 1).await?;
/// tracker.record_trade_executed(&mm_id).await?;
///
/// // Get computed metrics
/// let metrics = tracker.get_metrics(&mm_id).await?;
/// ```
#[derive(Debug)]
pub struct MmPerformanceTracker {
    /// Repository for persisting performance events.
    repository: Arc<dyn MmPerformanceRepository>,
    /// Rolling window size in days.
    window_days: u32,
}

impl MmPerformanceTracker {
    /// Creates a new performance tracker.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository for persisting performance events
    /// * `window_days` - Rolling window size in days (default: 7)
    #[must_use]
    pub fn new(repository: Arc<dyn MmPerformanceRepository>, window_days: u32) -> Self {
        Self {
            repository,
            window_days,
        }
    }

    /// Creates a new tracker with default 7-day window.
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository for persisting performance events
    #[must_use]
    pub fn with_defaults(repository: Arc<dyn MmPerformanceRepository>) -> Self {
        Self::new(repository, DEFAULT_WINDOW_DAYS)
    }

    /// Returns the configured window size in days.
    #[inline]
    #[must_use]
    pub fn window_days(&self) -> u32 {
        self.window_days
    }

    /// Records that an RFQ was sent to a market maker.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if the event cannot be stored.
    pub async fn record_rfq_sent(&self, mm_id: &CounterpartyId) -> MmPerformanceResult<()> {
        let event = MmPerformanceEvent::new(
            mm_id.clone(),
            MmPerformanceEventKind::RfqSent,
            Timestamp::now(),
        );
        self.repository.record_event(event).await
    }

    /// Records that a quote was received from a market maker.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    /// * `response_time_ms` - Time from RFQ broadcast to quote receipt in milliseconds
    /// * `rank` - Position in the quote ranking (1 = best)
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if the event cannot be stored.
    pub async fn record_quote_received(
        &self,
        mm_id: &CounterpartyId,
        response_time_ms: u64,
        rank: u64,
    ) -> MmPerformanceResult<()> {
        let event = MmPerformanceEvent::new(
            mm_id.clone(),
            MmPerformanceEventKind::QuoteReceived {
                response_time_ms,
                rank,
            },
            Timestamp::now(),
        );
        self.repository.record_event(event).await
    }

    /// Records that a trade was executed from a market maker's quote.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if the event cannot be stored.
    pub async fn record_trade_executed(&self, mm_id: &CounterpartyId) -> MmPerformanceResult<()> {
        let event = MmPerformanceEvent::new(
            mm_id.clone(),
            MmPerformanceEventKind::TradeExecuted,
            Timestamp::now(),
        );
        self.repository.record_event(event).await
    }

    /// Records that a last-look reject was received from a market maker.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if the event cannot be stored.
    pub async fn record_last_look_reject(&self, mm_id: &CounterpartyId) -> MmPerformanceResult<()> {
        let event = MmPerformanceEvent::new(
            mm_id.clone(),
            MmPerformanceEventKind::LastLookReject,
            Timestamp::now(),
        );
        self.repository.record_event(event).await
    }

    /// Records that an accept was requested from a market maker.
    ///
    /// This is the denominator for the reject rate calculation.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if the event cannot be stored.
    pub async fn record_accept_requested(&self, mm_id: &CounterpartyId) -> MmPerformanceResult<()> {
        let event = MmPerformanceEvent::new(
            mm_id.clone(),
            MmPerformanceEventKind::AcceptRequested,
            Timestamp::now(),
        );
        self.repository.record_event(event).await
    }

    /// Computes performance metrics for a specific market maker.
    ///
    /// Metrics are computed over the configured rolling window ending at the
    /// current time.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if events cannot be retrieved.
    pub async fn get_metrics(
        &self,
        mm_id: &CounterpartyId,
    ) -> MmPerformanceResult<MmPerformanceMetrics> {
        let now = Timestamp::now();
        let window_start = now.sub_secs(i64::from(self.window_days) * 86400);
        let events = self.repository.get_events(mm_id, window_start, now).await?;

        Ok(MmPerformanceMetrics::compute(
            mm_id,
            &events,
            window_start,
            now,
        ))
    }

    /// Returns market makers meeting the minimum response rate threshold.
    ///
    /// # Arguments
    ///
    /// * `min_response_rate_pct` - Minimum response rate percentage (0-100)
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if data cannot be retrieved.
    pub async fn get_eligible_mms(
        &self,
        min_response_rate_pct: f64,
    ) -> MmPerformanceResult<Vec<CounterpartyId>> {
        let all_ids = self.repository.get_all_mm_ids().await?;
        let mut eligible = Vec::new();

        for mm_id in &all_ids {
            let metrics = self.get_metrics(mm_id).await?;
            if metrics.is_eligible(min_response_rate_pct) {
                eligible.push(mm_id.clone());
            }
        }

        Ok(eligible)
    }

    /// Returns all market maker metrics.
    ///
    /// Computes metrics for every MM that has recorded events.
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if data cannot be retrieved.
    pub async fn get_all_metrics(&self) -> MmPerformanceResult<Vec<MmPerformanceMetrics>> {
        let all_ids = self.repository.get_all_mm_ids().await?;
        let now = Timestamp::now();
        let window_start = now.sub_secs(i64::from(self.window_days) * 86400);
        let mut all_metrics = Vec::with_capacity(all_ids.len());

        for mm_id in &all_ids {
            let events = self.repository.get_events(mm_id, window_start, now).await?;
            all_metrics.push(MmPerformanceMetrics::compute(
                mm_id,
                &events,
                window_start,
                now,
            ));
        }

        Ok(all_metrics)
    }

    /// Trims events older than the rolling window for all market makers.
    ///
    /// Should be called periodically to prevent unbounded storage growth.
    ///
    /// # Returns
    ///
    /// The number of events removed.
    ///
    /// # Errors
    ///
    /// Returns `MmPerformanceError::Repository` if trimming fails.
    pub async fn trim_old_events(&self) -> MmPerformanceResult<u64> {
        let cutoff = Timestamp::now().sub_secs(i64::from(self.window_days) * 86400);
        self.repository.trim_before(cutoff).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use dashmap::DashMap;

    /// In-memory implementation for testing.
    #[derive(Debug, Default)]
    struct MockMmPerformanceRepo {
        events: DashMap<String, Vec<MmPerformanceEvent>>,
    }

    #[async_trait]
    impl MmPerformanceRepository for MockMmPerformanceRepo {
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

    fn mm_id(name: &str) -> CounterpartyId {
        CounterpartyId::new(name)
    }

    fn create_tracker() -> (MmPerformanceTracker, Arc<MockMmPerformanceRepo>) {
        let repo = Arc::new(MockMmPerformanceRepo::default());
        let tracker =
            MmPerformanceTracker::new(Arc::clone(&repo) as Arc<dyn MmPerformanceRepository>, 7);
        (tracker, repo)
    }

    mod construction {
        use super::*;

        #[test]
        fn new_sets_window_days() {
            let repo = Arc::new(MockMmPerformanceRepo::default());
            let tracker = MmPerformanceTracker::new(repo, 14);

            assert_eq!(tracker.window_days(), 14);
        }

        #[test]
        fn with_defaults_uses_7_days() {
            let repo = Arc::new(MockMmPerformanceRepo::default());
            let tracker =
                MmPerformanceTracker::with_defaults(repo as Arc<dyn MmPerformanceRepository>);

            assert_eq!(tracker.window_days(), DEFAULT_WINDOW_DAYS);
        }
    }

    mod record_events {
        use super::*;

        #[tokio::test]
        async fn record_rfq_sent() {
            let (tracker, repo) = create_tracker();
            let id = mm_id("mm-a");

            let result = tracker.record_rfq_sent(&id).await;
            assert!(result.is_ok());

            let events = repo.events.get("mm-a");
            assert!(events.is_some());
            assert_eq!(events.unwrap().len(), 1);
        }

        #[tokio::test]
        async fn record_quote_received() {
            let (tracker, repo) = create_tracker();
            let id = mm_id("mm-b");

            let result = tracker.record_quote_received(&id, 150, 2).await;
            assert!(result.is_ok());

            let events = repo.events.get("mm-b").unwrap();
            assert_eq!(events.len(), 1);

            let first = events.first().unwrap();
            assert!(
                matches!(
                    first.kind(),
                    MmPerformanceEventKind::QuoteReceived {
                        response_time_ms: 150,
                        rank: 2,
                    }
                ),
                "expected QuoteReceived(150ms, rank=2), got {:?}",
                first.kind()
            );
        }

        #[tokio::test]
        async fn record_trade_executed() {
            let (tracker, repo) = create_tracker();
            let id = mm_id("mm-c");

            let result = tracker.record_trade_executed(&id).await;
            assert!(result.is_ok());

            let events = repo.events.get("mm-c").unwrap();
            assert_eq!(
                events.first().unwrap().kind().as_u8(),
                MmPerformanceEventKind::TradeExecuted.as_u8()
            );
        }

        #[tokio::test]
        async fn record_last_look_reject() {
            let (tracker, repo) = create_tracker();
            let id = mm_id("mm-d");

            let result = tracker.record_last_look_reject(&id).await;
            assert!(result.is_ok());

            let events = repo.events.get("mm-d").unwrap();
            assert_eq!(
                events.first().unwrap().kind().as_u8(),
                MmPerformanceEventKind::LastLookReject.as_u8()
            );
        }

        #[tokio::test]
        async fn record_accept_requested() {
            let (tracker, repo) = create_tracker();
            let id = mm_id("mm-e");

            let result = tracker.record_accept_requested(&id).await;
            assert!(result.is_ok());

            let events = repo.events.get("mm-e").unwrap();
            assert_eq!(
                events.first().unwrap().kind().as_u8(),
                MmPerformanceEventKind::AcceptRequested.as_u8()
            );
        }
    }

    mod get_metrics {
        use super::*;

        #[tokio::test]
        async fn no_events_returns_empty_metrics() {
            let (tracker, _) = create_tracker();
            let id = mm_id("mm-empty");

            let metrics = tracker.get_metrics(&id).await;
            assert!(metrics.is_ok());

            let metrics = metrics.unwrap();
            assert_eq!(metrics.total_rfqs_received(), 0);
            assert!(metrics.response_rate_pct().is_none());
        }

        #[tokio::test]
        async fn metrics_computed_after_events() {
            let (tracker, _) = create_tracker();
            let id = mm_id("mm-active");

            // Record events
            assert!(tracker.record_rfq_sent(&id).await.is_ok());
            assert!(tracker.record_rfq_sent(&id).await.is_ok());
            assert!(tracker.record_quote_received(&id, 100, 1).await.is_ok());
            assert!(tracker.record_trade_executed(&id).await.is_ok());

            let metrics = tracker.get_metrics(&id).await.unwrap();

            assert_eq!(metrics.total_rfqs_received(), 2);
            assert_eq!(metrics.total_quotes_provided(), 1);
            assert_eq!(metrics.total_trades_executed(), 1);
            assert!(metrics.response_rate_pct().is_some());
        }
    }

    mod eligible_mms {
        use super::*;
        use crate::domain::entities::mm_performance::DEFAULT_MIN_RESPONSE_RATE_PCT;

        #[tokio::test]
        async fn eligible_mms_filters_correctly() {
            let (tracker, _) = create_tracker();

            // mm-good: 100% response rate
            let good = mm_id("mm-good");
            assert!(tracker.record_rfq_sent(&good).await.is_ok());
            assert!(tracker.record_quote_received(&good, 100, 1).await.is_ok());

            // mm-bad: 0% response rate (only RFQs, no quotes)
            let bad = mm_id("mm-bad");
            assert!(tracker.record_rfq_sent(&bad).await.is_ok());
            assert!(tracker.record_rfq_sent(&bad).await.is_ok());

            let eligible = tracker
                .get_eligible_mms(DEFAULT_MIN_RESPONSE_RATE_PCT)
                .await
                .unwrap();

            // mm-good should be eligible, mm-bad should not
            assert!(eligible.iter().any(|id| id.as_str() == "mm-good"));
            assert!(!eligible.iter().any(|id| id.as_str() == "mm-bad"));
        }
    }

    mod get_all_metrics {
        use super::*;

        #[tokio::test]
        async fn returns_metrics_for_all_mms() {
            let (tracker, _) = create_tracker();

            let mm1 = mm_id("mm-1");
            let mm2 = mm_id("mm-2");

            assert!(tracker.record_rfq_sent(&mm1).await.is_ok());
            assert!(tracker.record_rfq_sent(&mm2).await.is_ok());

            let all_metrics = tracker.get_all_metrics().await.unwrap();

            assert_eq!(all_metrics.len(), 2);
        }
    }

    mod trim {
        use super::*;

        #[tokio::test]
        async fn trim_old_events_removes_expired() {
            let (tracker, repo) = create_tracker();
            let id = mm_id("mm-trim");

            // Insert an old event directly
            let old_event = MmPerformanceEvent::new(
                id.clone(),
                MmPerformanceEventKind::RfqSent,
                Timestamp::from_secs(1_000_000).unwrap(), // very old
            );
            assert!(repo.record_event(old_event).await.is_ok());

            // Insert a recent event
            assert!(tracker.record_rfq_sent(&id).await.is_ok());

            let removed = tracker.trim_old_events().await.unwrap();
            assert_eq!(removed, 1);

            // Only the recent event should remain
            let events = repo.events.get("mm-trim").unwrap();
            assert_eq!(events.len(), 1);
        }
    }

    mod errors {
        use super::*;

        #[test]
        fn error_display() {
            let err = MmPerformanceError::repository("connection failed");
            assert!(err.to_string().contains("repository error"));
            assert!(err.to_string().contains("connection failed"));

            let err = MmPerformanceError::invalid_config("window must be > 0");
            assert!(err.to_string().contains("invalid configuration"));
        }
    }
}
