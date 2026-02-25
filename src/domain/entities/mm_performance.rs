//! # Market Maker Performance Entity
//!
//! Tracks per-market-maker performance metrics over a rolling window.
//!
//! This module provides the [`MmPerformanceMetrics`] computed metrics struct
//! and [`MmPerformanceEvent`] for recording individual performance data points
//! such as RFQ sends, quote receipts, trade executions, and last-look rejects.
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::entities::mm_performance::{
//!     MmPerformanceEvent, MmPerformanceEventKind, MmPerformanceMetrics,
//! };
//! use otc_rfq::domain::value_objects::CounterpartyId;
//! use otc_rfq::domain::value_objects::timestamp::Timestamp;
//!
//! let mm_id = CounterpartyId::new("mm-citadel");
//! let now = Timestamp::now();
//!
//! let events = vec![
//!     MmPerformanceEvent::new(mm_id.clone(), MmPerformanceEventKind::RfqSent, now),
//!     MmPerformanceEvent::new(
//!         mm_id.clone(),
//!         MmPerformanceEventKind::QuoteReceived { response_time_ms: 150, rank: 1 },
//!         now,
//!     ),
//! ];
//!
//! let metrics = MmPerformanceMetrics::compute(&mm_id, &events, now.sub_secs(86400 * 7), now);
//! assert_eq!(metrics.total_rfqs_received(), 1);
//! ```

use crate::domain::value_objects::CounterpartyId;
use crate::domain::value_objects::timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Default rolling window size in days.
pub const DEFAULT_WINDOW_DAYS: u32 = 7;

/// Default minimum response rate percentage for MM eligibility.
pub const DEFAULT_MIN_RESPONSE_RATE_PCT: f64 = 80.0;

/// Kind of market maker performance event.
///
/// Represents the different types of interactions tracked for
/// computing performance metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum MmPerformanceEventKind {
    /// An RFQ was sent to this market maker.
    RfqSent = 0,

    /// A quote was received from this market maker.
    ///
    /// Fields:
    /// - `response_time_ms`: time from RFQ broadcast to quote receipt in milliseconds
    /// - `rank`: position in the quote ranking (1 = best, lower = more competitive)
    QuoteReceived {
        /// Time from RFQ broadcast to quote receipt in milliseconds.
        response_time_ms: u64,
        /// Position in the quote ranking (1 = best).
        rank: u64,
    } = 1,

    /// A trade was executed from this market maker's quote.
    TradeExecuted = 2,

    /// A last-look reject was recorded for this market maker.
    LastLookReject = 3,

    /// An accept was requested from this market maker (for reject rate denominator).
    AcceptRequested = 4,
}

impl MmPerformanceEventKind {
    /// Returns the numeric value of this event kind.
    #[inline]
    #[must_use]
    pub const fn as_u8(&self) -> u8 {
        match self {
            Self::RfqSent => 0,
            Self::QuoteReceived { .. } => 1,
            Self::TradeExecuted => 2,
            Self::LastLookReject => 3,
            Self::AcceptRequested => 4,
        }
    }
}

impl fmt::Display for MmPerformanceEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RfqSent => write!(f, "RFQ_SENT"),
            Self::QuoteReceived {
                response_time_ms,
                rank,
            } => {
                write!(f, "QUOTE_RECEIVED({}ms, rank={})", response_time_ms, rank)
            }
            Self::TradeExecuted => write!(f, "TRADE_EXECUTED"),
            Self::LastLookReject => write!(f, "LAST_LOOK_REJECT"),
            Self::AcceptRequested => write!(f, "ACCEPT_REQUESTED"),
        }
    }
}

/// A single performance data point for a market maker.
///
/// Records one event in the MM's performance history, used to compute
/// aggregated [`MmPerformanceMetrics`] over a rolling window.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::entities::mm_performance::{MmPerformanceEvent, MmPerformanceEventKind};
/// use otc_rfq::domain::value_objects::CounterpartyId;
/// use otc_rfq::domain::value_objects::timestamp::Timestamp;
///
/// let event = MmPerformanceEvent::new(
///     CounterpartyId::new("mm-jump"),
///     MmPerformanceEventKind::RfqSent,
///     Timestamp::now(),
/// );
///
/// assert_eq!(event.kind().as_u8(), 0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MmPerformanceEvent {
    /// Market maker identifier.
    mm_id: CounterpartyId,
    /// Type of event recorded.
    kind: MmPerformanceEventKind,
    /// When this event occurred.
    timestamp: Timestamp,
}

impl MmPerformanceEvent {
    /// Creates a new performance event.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    /// * `kind` - Type of event
    /// * `timestamp` - When the event occurred
    #[must_use]
    pub fn new(mm_id: CounterpartyId, kind: MmPerformanceEventKind, timestamp: Timestamp) -> Self {
        Self {
            mm_id,
            kind,
            timestamp,
        }
    }

    /// Returns the market maker identifier.
    #[inline]
    #[must_use]
    pub fn mm_id(&self) -> &CounterpartyId {
        &self.mm_id
    }

    /// Returns the event kind.
    #[inline]
    #[must_use]
    pub fn kind(&self) -> &MmPerformanceEventKind {
        &self.kind
    }

    /// Returns the event timestamp.
    #[inline]
    #[must_use]
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Returns true if this event is within the given time window.
    #[inline]
    #[must_use]
    pub fn is_within_window(&self, window_start: Timestamp, window_end: Timestamp) -> bool {
        !self.timestamp.is_before(&window_start) && !self.timestamp.is_after(&window_end)
    }
}

impl fmt::Display for MmPerformanceEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MmEvent({} {} @ {})",
            self.mm_id, self.kind, self.timestamp
        )
    }
}

/// Computed performance metrics for a market maker over a rolling window.
///
/// All percentage fields are in the range 0.0–100.0.
/// The `competitiveness_score` is an average rank (lower = more competitive).
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::entities::mm_performance::{
///     MmPerformanceEvent, MmPerformanceEventKind, MmPerformanceMetrics,
/// };
/// use otc_rfq::domain::value_objects::CounterpartyId;
/// use otc_rfq::domain::value_objects::timestamp::Timestamp;
///
/// let mm_id = CounterpartyId::new("mm-wintermute");
/// let now = Timestamp::now();
/// let window_start = now.sub_secs(86400 * 7);
///
/// // No events → zero metrics
/// let metrics = MmPerformanceMetrics::compute(&mm_id, &[], window_start, now);
/// assert_eq!(metrics.total_rfqs_received(), 0);
/// assert!(metrics.response_rate_pct().is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MmPerformanceMetrics {
    /// Market maker identifier.
    mm_id: CounterpartyId,
    /// Response rate: (quotes_provided / rfqs_sent) × 100. Percentage (0-100).
    response_rate_pct: Option<f64>,
    /// Average response time from RFQ broadcast to quote receipt in milliseconds.
    avg_response_time_ms: Option<f64>,
    /// Quote-to-trade conversion: (trades_executed / quotes_provided) × 100. Percentage (0-100).
    quote_to_trade_pct: Option<f64>,
    /// Average rank across all quotes (lower = more competitive).
    competitiveness_score: Option<f64>,
    /// Reject rate: (last_look_rejects / accepts_requested) × 100. Percentage (0-100).
    reject_rate_pct: Option<f64>,
    /// Total RFQs sent to this MM within the window.
    total_rfqs_received: u64,
    /// Total quotes provided by this MM within the window.
    total_quotes_provided: u64,
    /// Total trades executed from this MM's quotes within the window.
    total_trades_executed: u64,
    /// Total accepts requested from this MM within the window.
    total_accepts_requested: u64,
    /// Total last-look rejects from this MM within the window.
    total_last_look_rejects: u64,
    /// Start of the rolling window.
    window_start: Timestamp,
    /// End of the rolling window.
    window_end: Timestamp,
}

impl MmPerformanceMetrics {
    /// Computes metrics from a set of events within the specified window.
    ///
    /// Events outside the `[window_start, window_end]` range are ignored.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    /// * `events` - Slice of performance events (may include events outside the window)
    /// * `window_start` - Start of the rolling window (inclusive)
    /// * `window_end` - End of the rolling window (inclusive)
    #[must_use]
    pub fn compute(
        mm_id: &CounterpartyId,
        events: &[MmPerformanceEvent],
        window_start: Timestamp,
        window_end: Timestamp,
    ) -> Self {
        let mut total_rfqs_received: u64 = 0;
        let mut total_quotes_provided: u64 = 0;
        let mut total_trades_executed: u64 = 0;
        let mut total_accepts_requested: u64 = 0;
        let mut total_last_look_rejects: u64 = 0;
        let mut total_response_time_ms: u64 = 0;
        let mut total_rank: u64 = 0;

        for event in events {
            if !event.is_within_window(window_start, window_end) {
                continue;
            }

            match &event.kind {
                MmPerformanceEventKind::RfqSent => {
                    total_rfqs_received = total_rfqs_received.saturating_add(1);
                }
                MmPerformanceEventKind::QuoteReceived {
                    response_time_ms,
                    rank,
                } => {
                    total_quotes_provided = total_quotes_provided.saturating_add(1);
                    total_response_time_ms =
                        total_response_time_ms.saturating_add(*response_time_ms);
                    total_rank = total_rank.saturating_add(*rank);
                }
                MmPerformanceEventKind::TradeExecuted => {
                    total_trades_executed = total_trades_executed.saturating_add(1);
                }
                MmPerformanceEventKind::LastLookReject => {
                    total_last_look_rejects = total_last_look_rejects.saturating_add(1);
                }
                MmPerformanceEventKind::AcceptRequested => {
                    total_accepts_requested = total_accepts_requested.saturating_add(1);
                }
            }
        }

        // response_rate_pct = (quotes_provided / rfqs_sent) × 100
        let response_rate_pct = if total_rfqs_received > 0 {
            Some((total_quotes_provided as f64 / total_rfqs_received as f64) * 100.0)
        } else {
            None
        };

        // avg_response_time_ms = total_response_time / quotes_provided
        let avg_response_time_ms = if total_quotes_provided > 0 {
            Some(total_response_time_ms as f64 / total_quotes_provided as f64)
        } else {
            None
        };

        // quote_to_trade_pct = (trades_executed / quotes_provided) × 100
        let quote_to_trade_pct = if total_quotes_provided > 0 {
            Some((total_trades_executed as f64 / total_quotes_provided as f64) * 100.0)
        } else {
            None
        };

        // competitiveness_score = average rank
        let competitiveness_score = if total_quotes_provided > 0 {
            Some(total_rank as f64 / total_quotes_provided as f64)
        } else {
            None
        };

        // reject_rate_pct = (last_look_rejects / accepts_requested) × 100
        let reject_rate_pct = if total_accepts_requested > 0 {
            Some((total_last_look_rejects as f64 / total_accepts_requested as f64) * 100.0)
        } else {
            None
        };

        Self {
            mm_id: mm_id.clone(),
            response_rate_pct,
            avg_response_time_ms,
            quote_to_trade_pct,
            competitiveness_score,
            reject_rate_pct,
            total_rfqs_received,
            total_quotes_provided,
            total_trades_executed,
            total_accepts_requested,
            total_last_look_rejects,
            window_start,
            window_end,
        }
    }

    /// Returns the market maker identifier.
    #[inline]
    #[must_use]
    pub fn mm_id(&self) -> &CounterpartyId {
        &self.mm_id
    }

    /// Returns the response rate as a percentage (0-100), or `None` if no RFQs were sent.
    #[inline]
    #[must_use]
    pub fn response_rate_pct(&self) -> Option<f64> {
        self.response_rate_pct
    }

    /// Returns the average response time in milliseconds, or `None` if no quotes were received.
    #[inline]
    #[must_use]
    pub fn avg_response_time_ms(&self) -> Option<f64> {
        self.avg_response_time_ms
    }

    /// Returns the quote-to-trade conversion percentage (0-100), or `None` if no quotes were provided.
    #[inline]
    #[must_use]
    pub fn quote_to_trade_pct(&self) -> Option<f64> {
        self.quote_to_trade_pct
    }

    /// Returns the competitiveness score (average rank, lower = better), or `None` if no quotes.
    #[inline]
    #[must_use]
    pub fn competitiveness_score(&self) -> Option<f64> {
        self.competitiveness_score
    }

    /// Returns the reject rate percentage (0-100), or `None` if no accepts were requested.
    #[inline]
    #[must_use]
    pub fn reject_rate_pct(&self) -> Option<f64> {
        self.reject_rate_pct
    }

    /// Returns the total number of RFQs sent to this MM in the window.
    #[inline]
    #[must_use]
    pub fn total_rfqs_received(&self) -> u64 {
        self.total_rfqs_received
    }

    /// Returns the total number of quotes provided by this MM in the window.
    #[inline]
    #[must_use]
    pub fn total_quotes_provided(&self) -> u64 {
        self.total_quotes_provided
    }

    /// Returns the total number of trades executed from this MM's quotes in the window.
    #[inline]
    #[must_use]
    pub fn total_trades_executed(&self) -> u64 {
        self.total_trades_executed
    }

    /// Returns the total number of accepts requested from this MM in the window.
    #[inline]
    #[must_use]
    pub fn total_accepts_requested(&self) -> u64 {
        self.total_accepts_requested
    }

    /// Returns the total number of last-look rejects from this MM in the window.
    #[inline]
    #[must_use]
    pub fn total_last_look_rejects(&self) -> u64 {
        self.total_last_look_rejects
    }

    /// Returns the start of the rolling window.
    #[inline]
    #[must_use]
    pub fn window_start(&self) -> Timestamp {
        self.window_start
    }

    /// Returns the end of the rolling window.
    #[inline]
    #[must_use]
    pub fn window_end(&self) -> Timestamp {
        self.window_end
    }

    /// Returns true if this MM meets the minimum response rate for eligibility.
    ///
    /// # Arguments
    ///
    /// * `min_response_rate_pct` - Minimum response rate percentage (0-100)
    #[must_use]
    pub fn is_eligible(&self, min_response_rate_pct: f64) -> bool {
        match self.response_rate_pct {
            Some(rate) => rate >= min_response_rate_pct,
            // If no RFQs sent, consider eligible (new MM)
            None => true,
        }
    }
}

impl fmt::Display for MmPerformanceMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MmMetrics({} response={:.1}% trades={:.1}% rank={:.1} reject={:.1}%)",
            self.mm_id,
            self.response_rate_pct.unwrap_or(0.0),
            self.quote_to_trade_pct.unwrap_or(0.0),
            self.competitiveness_score.unwrap_or(0.0),
            self.reject_rate_pct.unwrap_or(0.0),
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn mm_id() -> CounterpartyId {
        CounterpartyId::new("mm-test")
    }

    fn now() -> Timestamp {
        Timestamp::from_secs(1_700_000_000).unwrap()
    }

    fn window_start() -> Timestamp {
        now().sub_secs(86400 * 7) // 7 days ago
    }

    fn make_event(kind: MmPerformanceEventKind) -> MmPerformanceEvent {
        MmPerformanceEvent::new(mm_id(), kind, now().sub_secs(3600))
    }

    fn make_event_at(kind: MmPerformanceEventKind, ts: Timestamp) -> MmPerformanceEvent {
        MmPerformanceEvent::new(mm_id(), kind, ts)
    }

    mod event_kind {
        use super::*;

        #[test]
        fn as_u8_values() {
            assert_eq!(MmPerformanceEventKind::RfqSent.as_u8(), 0);
            assert_eq!(
                MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1
                }
                .as_u8(),
                1
            );
            assert_eq!(MmPerformanceEventKind::TradeExecuted.as_u8(), 2);
            assert_eq!(MmPerformanceEventKind::LastLookReject.as_u8(), 3);
            assert_eq!(MmPerformanceEventKind::AcceptRequested.as_u8(), 4);
        }

        #[test]
        fn display_formatting() {
            assert_eq!(MmPerformanceEventKind::RfqSent.to_string(), "RFQ_SENT");
            assert_eq!(
                MmPerformanceEventKind::TradeExecuted.to_string(),
                "TRADE_EXECUTED"
            );
            let quote = MmPerformanceEventKind::QuoteReceived {
                response_time_ms: 150,
                rank: 2,
            };
            assert!(quote.to_string().contains("150ms"));
            assert!(quote.to_string().contains("rank=2"));
        }
    }

    mod event_construction {
        use super::*;

        #[test]
        fn new_creates_event() {
            let event = make_event(MmPerformanceEventKind::RfqSent);

            assert_eq!(event.mm_id(), &mm_id());
            assert_eq!(event.kind().as_u8(), 0);
        }

        #[test]
        fn is_within_window_inside() {
            let ts = now().sub_secs(3600);
            let event = make_event_at(MmPerformanceEventKind::RfqSent, ts);

            assert!(event.is_within_window(window_start(), now()));
        }

        #[test]
        fn is_within_window_outside_before() {
            let ts = window_start().sub_secs(1);
            let event = make_event_at(MmPerformanceEventKind::RfqSent, ts);

            assert!(!event.is_within_window(window_start(), now()));
        }

        #[test]
        fn is_within_window_outside_after() {
            let ts = now().add_secs(1);
            let event = make_event_at(MmPerformanceEventKind::RfqSent, ts);

            assert!(!event.is_within_window(window_start(), now()));
        }

        #[test]
        fn is_within_window_exact_boundary() {
            let event_start = make_event_at(MmPerformanceEventKind::RfqSent, window_start());
            let event_end = make_event_at(MmPerformanceEventKind::RfqSent, now());

            assert!(event_start.is_within_window(window_start(), now()));
            assert!(event_end.is_within_window(window_start(), now()));
        }

        #[test]
        fn display_format() {
            let event = make_event(MmPerformanceEventKind::RfqSent);
            let display = event.to_string();

            assert!(display.contains("mm-test"));
            assert!(display.contains("RFQ_SENT"));
        }
    }

    mod metrics_computation {
        use super::*;

        #[test]
        fn no_events_returns_zero_metrics() {
            let metrics = MmPerformanceMetrics::compute(&mm_id(), &[], window_start(), now());

            assert_eq!(metrics.total_rfqs_received(), 0);
            assert_eq!(metrics.total_quotes_provided(), 0);
            assert_eq!(metrics.total_trades_executed(), 0);
            assert!(metrics.response_rate_pct().is_none());
            assert!(metrics.avg_response_time_ms().is_none());
            assert!(metrics.quote_to_trade_pct().is_none());
            assert!(metrics.competitiveness_score().is_none());
            assert!(metrics.reject_rate_pct().is_none());
        }

        #[test]
        fn response_rate_pct_computed_correctly() {
            let events = vec![
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 200,
                    rank: 2,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 150,
                    rank: 1,
                }),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            assert_eq!(metrics.total_rfqs_received(), 4);
            assert_eq!(metrics.total_quotes_provided(), 3);

            let rate = metrics.response_rate_pct().unwrap();
            assert!((rate - 75.0).abs() < f64::EPSILON);
        }

        #[test]
        fn avg_response_time_ms_computed_correctly() {
            let events = vec![
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 200,
                    rank: 2,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 300,
                    rank: 3,
                }),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            let avg = metrics.avg_response_time_ms().unwrap();
            assert!((avg - 200.0).abs() < f64::EPSILON);
        }

        #[test]
        fn quote_to_trade_pct_computed_correctly() {
            let events = vec![
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 200,
                    rank: 2,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 150,
                    rank: 1,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 120,
                    rank: 3,
                }),
                make_event(MmPerformanceEventKind::TradeExecuted),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            let pct = metrics.quote_to_trade_pct().unwrap();
            assert!((pct - 25.0).abs() < f64::EPSILON);
        }

        #[test]
        fn competitiveness_score_computed_correctly() {
            let events = vec![
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 200,
                    rank: 3,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 150,
                    rank: 2,
                }),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            let score = metrics.competitiveness_score().unwrap();
            assert!((score - 2.0).abs() < f64::EPSILON);
        }

        #[test]
        fn reject_rate_pct_computed_correctly() {
            let events = vec![
                make_event(MmPerformanceEventKind::AcceptRequested),
                make_event(MmPerformanceEventKind::AcceptRequested),
                make_event(MmPerformanceEventKind::AcceptRequested),
                make_event(MmPerformanceEventKind::AcceptRequested),
                make_event(MmPerformanceEventKind::AcceptRequested),
                make_event(MmPerformanceEventKind::LastLookReject),
                make_event(MmPerformanceEventKind::LastLookReject),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            assert_eq!(metrics.total_accepts_requested(), 5);
            assert_eq!(metrics.total_last_look_rejects(), 2);

            let rate = metrics.reject_rate_pct().unwrap();
            assert!((rate - 40.0).abs() < f64::EPSILON);
        }

        #[test]
        fn events_outside_window_are_excluded() {
            let old_ts = window_start().sub_secs(3600); // 1 hour before window start
            let events = vec![
                make_event_at(MmPerformanceEventKind::RfqSent, old_ts),
                make_event_at(
                    MmPerformanceEventKind::QuoteReceived {
                        response_time_ms: 100,
                        rank: 1,
                    },
                    old_ts,
                ),
                // This one is inside the window
                make_event(MmPerformanceEventKind::RfqSent),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            assert_eq!(metrics.total_rfqs_received(), 1);
            assert_eq!(metrics.total_quotes_provided(), 0);
        }

        #[test]
        fn all_events_outside_window_returns_empty_metrics() {
            let old_ts = window_start().sub_secs(86400); // 1 day before window
            let events = vec![
                make_event_at(MmPerformanceEventKind::RfqSent, old_ts),
                make_event_at(
                    MmPerformanceEventKind::QuoteReceived {
                        response_time_ms: 100,
                        rank: 1,
                    },
                    old_ts,
                ),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            assert_eq!(metrics.total_rfqs_received(), 0);
            assert_eq!(metrics.total_quotes_provided(), 0);
            assert!(metrics.response_rate_pct().is_none());
        }

        #[test]
        fn single_rfq_with_quote_gives_100_percent_response() {
            let events = vec![
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 50,
                    rank: 1,
                }),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            let rate = metrics.response_rate_pct().unwrap();
            assert!((rate - 100.0).abs() < f64::EPSILON);
        }

        #[test]
        fn window_boundaries_are_correct() {
            let metrics = MmPerformanceMetrics::compute(&mm_id(), &[], window_start(), now());

            assert_eq!(metrics.window_start(), window_start());
            assert_eq!(metrics.window_end(), now());
        }

        #[test]
        fn mm_id_is_preserved() {
            let custom_id = CounterpartyId::new("mm-custom");
            let metrics = MmPerformanceMetrics::compute(&custom_id, &[], window_start(), now());

            assert_eq!(metrics.mm_id(), &custom_id);
        }
    }

    mod eligibility {
        use super::*;

        #[test]
        fn eligible_when_above_threshold() {
            let events = vec![
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 200,
                    rank: 2,
                }),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            // 100% response rate
            assert!(metrics.is_eligible(80.0));
        }

        #[test]
        fn not_eligible_when_below_threshold() {
            let events = vec![
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1,
                }),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            // 20% response rate
            assert!(!metrics.is_eligible(80.0));
        }

        #[test]
        fn eligible_when_no_rfqs_sent_new_mm() {
            let metrics = MmPerformanceMetrics::compute(&mm_id(), &[], window_start(), now());

            // New MM with no data should be eligible
            assert!(metrics.is_eligible(80.0));
        }

        #[test]
        fn eligible_at_exact_threshold() {
            // 4 out of 5 = 80%
            let events = vec![
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 200,
                    rank: 2,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 150,
                    rank: 1,
                }),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 120,
                    rank: 3,
                }),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());

            // 80% exact
            assert!(metrics.is_eligible(80.0));
        }
    }

    mod display {
        use super::*;

        #[test]
        fn metrics_display_format() {
            let events = vec![
                make_event(MmPerformanceEventKind::RfqSent),
                make_event(MmPerformanceEventKind::QuoteReceived {
                    response_time_ms: 100,
                    rank: 1,
                }),
            ];

            let metrics = MmPerformanceMetrics::compute(&mm_id(), &events, window_start(), now());
            let display = metrics.to_string();

            assert!(display.contains("mm-test"));
            assert!(display.contains("response="));
        }
    }
}
