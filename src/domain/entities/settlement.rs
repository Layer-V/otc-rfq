//! # Incentive Settlement
//!
//! Domain entities for monthly MM incentive settlement cycles.
//!
//! Implements an event-sourced aggregate root pattern where settlements
//! accumulate incentive events over a period and finalize for payout.

use crate::domain::entities::mm_incentive::IncentiveResult;
use crate::domain::value_objects::timestamp::Timestamp;
use crate::domain::value_objects::{CounterpartyId, TradeId};
use chrono::{TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// ============================================================================
// SettlementId
// ============================================================================

/// Unique identifier for a settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SettlementId(Uuid);

impl SettlementId {
    /// Creates a new random settlement ID.
    #[must_use]
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a settlement ID from a UUID.
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID.
    #[inline]
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Display for SettlementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// SettlementPeriod
// ============================================================================

/// Represents a settlement period (typically one calendar month).
///
/// # Invariants
///
/// - Start timestamp must be before end timestamp
/// - Period boundaries are inclusive [start, end]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementPeriod {
    /// Start of the period (inclusive).
    start: Timestamp,
    /// End of the period (inclusive).
    end: Timestamp,
}

impl SettlementPeriod {
    /// Creates a new settlement period.
    ///
    /// # Arguments
    ///
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (inclusive)
    ///
    /// # Returns
    ///
    /// A new `SettlementPeriod` if start < end, otherwise `None`.
    #[must_use]
    pub fn new(start: Timestamp, end: Timestamp) -> Option<Self> {
        if start < end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Creates a settlement period for a specific month and year.
    ///
    /// # Arguments
    ///
    /// * `year` - Year (e.g., 2026)
    /// * `month` - Month (1-12)
    ///
    /// # Returns
    ///
    /// A `SettlementPeriod` covering the entire month, or `None` if invalid month.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let period = SettlementPeriod::from_month_year(2026, 3)?;
    /// // Covers March 1, 2026 00:00:00 to March 31, 2026 23:59:59
    /// ```
    #[must_use]
    pub fn from_month_year(year: i32, month: u32) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }

        // First day of month at 00:00:00
        let start_dt = Utc
            .with_ymd_and_hms(year, month, 1, 0, 0, 0)
            .single()
            .and_then(|dt| Timestamp::from_millis(dt.timestamp_millis()))?;

        // Last day of month at 23:59:59
        let days_in_month = Self::days_in_month(year, month)?;
        let end_dt = Utc
            .with_ymd_and_hms(year, month, days_in_month, 23, 59, 59)
            .single()
            .and_then(|dt| Timestamp::from_millis(dt.timestamp_millis()))?;

        let start = start_dt;
        let end = end_dt;

        Some(Self { start, end })
    }

    /// Returns the number of days in a given month.
    #[must_use]
    fn days_in_month(year: i32, month: u32) -> Option<u32> {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
            4 | 6 | 9 | 11 => Some(30),
            2 => {
                // Leap year calculation
                let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
                Some(if is_leap { 29 } else { 28 })
            }
            _ => None,
        }
    }

    /// Returns the start timestamp.
    #[inline]
    #[must_use]
    pub const fn start(&self) -> Timestamp {
        self.start
    }

    /// Returns the end timestamp.
    #[inline]
    #[must_use]
    pub const fn end(&self) -> Timestamp {
        self.end
    }

    /// Checks if a timestamp falls within this period.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The timestamp to check
    ///
    /// # Returns
    ///
    /// `true` if the timestamp is within [start, end] inclusive.
    #[inline]
    #[must_use]
    pub fn contains(&self, timestamp: Timestamp) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }

    /// Checks if the period has ended (current time is after end).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        Timestamp::now() > self.end
    }
}

// ============================================================================
// SettlementStatus
// ============================================================================

/// Status of a settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementStatus {
    /// Settlement is open and accepting events.
    Open,
    /// Settlement is being finalized (transitional state).
    Finalizing,
    /// Settlement has been finalized and is ready for payout.
    Finalized,
    /// Payout has been processed (external system).
    Paid,
}

impl fmt::Display for SettlementStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open => write!(f, "OPEN"),
            Self::Finalizing => write!(f, "FINALIZING"),
            Self::Finalized => write!(f, "FINALIZED"),
            Self::Paid => write!(f, "PAID"),
        }
    }
}

// ============================================================================
// IncentiveEvent
// ============================================================================

/// Domain event representing an incentive earned by an MM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncentiveEvent {
    /// A trade incentive was earned.
    TradeIncentiveEarned {
        /// Trade identifier.
        trade_id: TradeId,
        /// Market maker identifier.
        mm_id: CounterpartyId,
        /// Incentive calculation result.
        result: IncentiveResult,
        /// Trade notional in USD.
        notional: Decimal,
        /// Timestamp when the incentive was earned.
        timestamp: Timestamp,
    },
}

impl IncentiveEvent {
    /// Creates a new trade incentive earned event.
    #[must_use]
    pub fn trade_incentive_earned(
        trade_id: TradeId,
        mm_id: CounterpartyId,
        result: IncentiveResult,
        notional: Decimal,
        timestamp: Timestamp,
    ) -> Self {
        Self::TradeIncentiveEarned {
            trade_id,
            mm_id,
            result,
            notional,
            timestamp,
        }
    }

    /// Returns the market maker ID from the event.
    #[must_use]
    pub fn mm_id(&self) -> &CounterpartyId {
        match self {
            Self::TradeIncentiveEarned { mm_id, .. } => mm_id,
        }
    }

    /// Returns the timestamp from the event.
    #[must_use]
    pub fn timestamp(&self) -> Timestamp {
        match self {
            Self::TradeIncentiveEarned { timestamp, .. } => *timestamp,
        }
    }
}

// ============================================================================
// IncentiveSettlement
// ============================================================================

/// Aggregate root for MM incentive settlement.
///
/// Accumulates incentive events over a period and finalizes for payout.
/// Implements event sourcing pattern for idempotency and audit trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncentiveSettlement {
    /// Unique settlement identifier.
    id: SettlementId,
    /// Market maker identifier.
    mm_id: CounterpartyId,
    /// Settlement period.
    period: SettlementPeriod,
    /// Current status.
    status: SettlementStatus,
    /// Total number of trades in this settlement.
    total_trades: u64,
    /// Total trade volume in USD.
    total_volume_usd: Decimal,
    /// Total base rebates earned (negative value = payment to MM).
    total_base_rebates_usd: Decimal,
    /// Total spread bonuses earned (negative value = payment to MM).
    total_bonuses_usd: Decimal,
    /// Net payout amount (negative value = payment to MM).
    net_payout_usd: Decimal,
    /// Events that have been applied to this settlement.
    events: Vec<IncentiveEvent>,
    /// Version number (incremented with each event).
    version: u64,
}

impl IncentiveSettlement {
    /// Creates a new settlement for a market maker and period.
    ///
    /// # Arguments
    ///
    /// * `mm_id` - Market maker identifier
    /// * `period` - Settlement period
    #[must_use]
    pub fn new(mm_id: CounterpartyId, period: SettlementPeriod) -> Self {
        Self {
            id: SettlementId::new_v4(),
            mm_id,
            period,
            status: SettlementStatus::Open,
            total_trades: 0,
            total_volume_usd: Decimal::ZERO,
            total_base_rebates_usd: Decimal::ZERO,
            total_bonuses_usd: Decimal::ZERO,
            net_payout_usd: Decimal::ZERO,
            events: Vec::new(),
            version: 0,
        }
    }

    /// Returns the settlement ID.
    #[inline]
    #[must_use]
    pub const fn id(&self) -> SettlementId {
        self.id
    }

    /// Returns the market maker ID.
    #[inline]
    #[must_use]
    pub fn mm_id(&self) -> &CounterpartyId {
        &self.mm_id
    }

    /// Returns the settlement period.
    #[inline]
    #[must_use]
    pub const fn period(&self) -> SettlementPeriod {
        self.period
    }

    /// Returns the current status.
    #[inline]
    #[must_use]
    pub const fn status(&self) -> SettlementStatus {
        self.status
    }

    /// Returns the total number of trades.
    #[inline]
    #[must_use]
    pub const fn total_trades(&self) -> u64 {
        self.total_trades
    }

    /// Returns the total volume in USD.
    #[inline]
    #[must_use]
    pub fn total_volume_usd(&self) -> Decimal {
        self.total_volume_usd
    }

    /// Returns the total base rebates in USD.
    #[inline]
    #[must_use]
    pub fn total_base_rebates_usd(&self) -> Decimal {
        self.total_base_rebates_usd
    }

    /// Returns the total bonuses in USD.
    #[inline]
    #[must_use]
    pub fn total_bonuses_usd(&self) -> Decimal {
        self.total_bonuses_usd
    }

    /// Returns the net payout amount in USD.
    #[inline]
    #[must_use]
    pub fn net_payout_usd(&self) -> Decimal {
        self.net_payout_usd
    }

    /// Returns the version number.
    #[inline]
    #[must_use]
    pub const fn version(&self) -> u64 {
        self.version
    }

    /// Returns a reference to the events.
    #[inline]
    #[must_use]
    pub fn events(&self) -> &[IncentiveEvent] {
        &self.events
    }

    /// Applies an incentive event to this settlement.
    ///
    /// Updates aggregated totals based on the event type.
    /// This is a pure function that modifies the settlement state.
    ///
    /// # Arguments
    ///
    /// * `event` - The incentive event to apply
    pub fn apply_event(&mut self, event: IncentiveEvent) {
        match &event {
            IncentiveEvent::TradeIncentiveEarned {
                result, notional, ..
            } => {
                // Increment trade count (checked)
                self.total_trades = self.total_trades.saturating_add(1);

                // Add to total volume (checked)
                self.total_volume_usd = self.total_volume_usd.saturating_add(*notional);

                // Add base rebate (negative value)
                let base_rebate = result.rebate_amount();
                self.total_base_rebates_usd =
                    self.total_base_rebates_usd.saturating_add(base_rebate);

                // Add spread bonus if any
                let bonus_amount = if !result.spread_bonus_bps().is_zero() {
                    // Calculate bonus amount: notional * spread_bonus_bps / 10000
                    let bps_divisor = Decimal::from(10000);
                    let product = notional.saturating_mul(result.spread_bonus_bps());
                    product.checked_div(bps_divisor).unwrap_or(Decimal::ZERO)
                } else {
                    Decimal::ZERO
                };
                self.total_bonuses_usd = self.total_bonuses_usd.saturating_add(bonus_amount);
            }
        }

        // Store event and increment version
        self.events.push(event);
        self.version = self.version.saturating_add(1);
    }

    /// Finalizes the settlement, calculating net payout and transitioning status.
    ///
    /// # Errors
    ///
    /// Returns an error if the settlement is already finalized.
    pub fn finalize(&mut self) -> Result<(), SettlementError> {
        if self.status != SettlementStatus::Open {
            return Err(SettlementError::AlreadyFinalized);
        }

        // Calculate net payout (sum of base rebates and bonuses)
        self.net_payout_usd = self
            .total_base_rebates_usd
            .saturating_add(self.total_bonuses_usd);

        // Transition to finalized status
        self.status = SettlementStatus::Finalized;

        Ok(())
    }

    /// Marks the settlement as paid (external payout completed).
    ///
    /// # Errors
    ///
    /// Returns an error if the settlement is not finalized.
    pub fn mark_as_paid(&mut self) -> Result<(), SettlementError> {
        if self.status != SettlementStatus::Finalized {
            return Err(SettlementError::NotFinalized);
        }

        self.status = SettlementStatus::Paid;
        Ok(())
    }
}

// ============================================================================
// SettlementError
// ============================================================================

/// Errors that can occur during settlement operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SettlementError {
    /// Settlement is already finalized.
    #[error("settlement is already finalized")]
    AlreadyFinalized,

    /// Settlement is not finalized.
    #[error("settlement is not finalized")]
    NotFinalized,

    /// Settlement not found.
    #[error("settlement not found for MM {0} and period")]
    NotFound(String),

    /// Invalid settlement period.
    #[error("invalid settlement period: {0}")]
    InvalidPeriod(String),

    /// Repository error.
    #[error("repository error: {0}")]
    Repository(String),
}
