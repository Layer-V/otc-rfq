//! # Trade Events
//!
//! Domain events for trade and settlement lifecycle.
//!
//! This module provides events that track trade execution and settlement.
//!
//! # Event Flow
//!
//! ```text
//! TradeExecuted -> SettlementInitiated -> SettlementConfirmed | SettlementFailed
//! ```

use crate::domain::events::domain_event::{DomainEvent, EventMetadata, EventType};
use crate::domain::value_objects::timestamp::Timestamp;
use crate::domain::value_objects::{
    Blockchain, CounterpartyId, EventId, Price, Quantity, QuoteId, RfqId, SettlementMethod,
    TradeId, VenueId,
};
use serde::{Deserialize, Serialize};

/// Event emitted when a trade is executed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeExecuted {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The trade ID.
    pub trade_id: TradeId,
    /// The quote that was executed.
    pub quote_id: QuoteId,
    /// The venue where the trade was executed.
    pub venue_id: VenueId,
    /// The counterparty (client).
    pub counterparty_id: CounterpartyId,
    /// The execution price.
    pub price: Price,
    /// The executed quantity.
    pub quantity: Quantity,
    /// The settlement method.
    pub settlement_method: SettlementMethod,
}

impl TradeExecuted {
    /// Creates a new TradeExecuted event.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rfq_id: RfqId,
        trade_id: TradeId,
        quote_id: QuoteId,
        venue_id: VenueId,
        counterparty_id: CounterpartyId,
        price: Price,
        quantity: Quantity,
        settlement_method: SettlementMethod,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            trade_id,
            quote_id,
            venue_id,
            counterparty_id,
            price,
            quantity,
            settlement_method,
        }
    }
}

impl DomainEvent for TradeExecuted {
    fn event_id(&self) -> EventId {
        self.metadata.event_id
    }

    fn rfq_id(&self) -> Option<RfqId> {
        self.metadata.rfq_id
    }

    fn timestamp(&self) -> Timestamp {
        self.metadata.timestamp
    }

    fn event_type(&self) -> EventType {
        EventType::Trade
    }

    fn event_name(&self) -> &'static str {
        "TradeExecuted"
    }
}

/// Event emitted when settlement is initiated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementInitiated {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The trade being settled.
    pub trade_id: TradeId,
    /// The settlement method.
    pub settlement_method: SettlementMethod,
    /// Transaction hash for on-chain settlement (if applicable).
    pub tx_hash: Option<String>,
}

impl SettlementInitiated {
    /// Creates a new SettlementInitiated event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        trade_id: TradeId,
        settlement_method: SettlementMethod,
        tx_hash: Option<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            trade_id,
            settlement_method,
            tx_hash,
        }
    }

    /// Creates a new SettlementInitiated event for on-chain settlement.
    #[must_use]
    pub fn on_chain(
        rfq_id: RfqId,
        trade_id: TradeId,
        blockchain: Blockchain,
        tx_hash: String,
    ) -> Self {
        Self::new(
            rfq_id,
            trade_id,
            SettlementMethod::OnChain(blockchain),
            Some(tx_hash),
        )
    }

    /// Creates a new SettlementInitiated event for off-chain settlement.
    #[must_use]
    pub fn off_chain(rfq_id: RfqId, trade_id: TradeId) -> Self {
        Self::new(rfq_id, trade_id, SettlementMethod::OffChain, None)
    }
}

impl DomainEvent for SettlementInitiated {
    fn event_id(&self) -> EventId {
        self.metadata.event_id
    }

    fn rfq_id(&self) -> Option<RfqId> {
        self.metadata.rfq_id
    }

    fn timestamp(&self) -> Timestamp {
        self.metadata.timestamp
    }

    fn event_type(&self) -> EventType {
        EventType::Settlement
    }

    fn event_name(&self) -> &'static str {
        "SettlementInitiated"
    }
}

/// Event emitted when settlement is confirmed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementConfirmed {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The trade that was settled.
    pub trade_id: TradeId,
    /// Transaction hash for on-chain settlement (if applicable).
    pub tx_hash: Option<String>,
    /// Block number for on-chain settlement (if applicable).
    pub block_number: Option<u64>,
}

impl SettlementConfirmed {
    /// Creates a new SettlementConfirmed event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        trade_id: TradeId,
        tx_hash: Option<String>,
        block_number: Option<u64>,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            trade_id,
            tx_hash,
            block_number,
        }
    }

    /// Creates a new SettlementConfirmed event for on-chain settlement.
    #[must_use]
    pub fn on_chain(rfq_id: RfqId, trade_id: TradeId, tx_hash: String, block_number: u64) -> Self {
        Self::new(rfq_id, trade_id, Some(tx_hash), Some(block_number))
    }

    /// Creates a new SettlementConfirmed event for off-chain settlement.
    #[must_use]
    pub fn off_chain(rfq_id: RfqId, trade_id: TradeId) -> Self {
        Self::new(rfq_id, trade_id, None, None)
    }
}

impl DomainEvent for SettlementConfirmed {
    fn event_id(&self) -> EventId {
        self.metadata.event_id
    }

    fn rfq_id(&self) -> Option<RfqId> {
        self.metadata.rfq_id
    }

    fn timestamp(&self) -> Timestamp {
        self.metadata.timestamp
    }

    fn event_type(&self) -> EventType {
        EventType::Settlement
    }

    fn event_name(&self) -> &'static str {
        "SettlementConfirmed"
    }
}

/// Event emitted when settlement fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementFailed {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The trade that failed to settle.
    pub trade_id: TradeId,
    /// Reason for failure.
    pub reason: String,
    /// Transaction hash if the failure was on-chain.
    pub tx_hash: Option<String>,
}

impl SettlementFailed {
    /// Creates a new SettlementFailed event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        trade_id: TradeId,
        reason: impl Into<String>,
        tx_hash: Option<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            trade_id,
            reason: reason.into(),
            tx_hash,
        }
    }
}

impl DomainEvent for SettlementFailed {
    fn event_id(&self) -> EventId {
        self.metadata.event_id
    }

    fn rfq_id(&self) -> Option<RfqId> {
        self.metadata.rfq_id
    }

    fn timestamp(&self) -> Timestamp {
        self.metadata.timestamp
    }

    fn event_type(&self) -> EventType {
        EventType::Settlement
    }

    fn event_name(&self) -> &'static str {
        "SettlementFailed"
    }
}

/// Enum containing all trade and settlement events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TradeEvent {
    /// Trade was executed.
    Executed(TradeExecuted),
    /// Settlement was initiated.
    SettlementInitiated(SettlementInitiated),
    /// Settlement was confirmed.
    SettlementConfirmed(SettlementConfirmed),
    /// Settlement failed.
    SettlementFailed(SettlementFailed),
}

impl DomainEvent for TradeEvent {
    fn event_id(&self) -> EventId {
        match self {
            Self::Executed(e) => e.event_id(),
            Self::SettlementInitiated(e) => e.event_id(),
            Self::SettlementConfirmed(e) => e.event_id(),
            Self::SettlementFailed(e) => e.event_id(),
        }
    }

    fn rfq_id(&self) -> Option<RfqId> {
        match self {
            Self::Executed(e) => e.rfq_id(),
            Self::SettlementInitiated(e) => e.rfq_id(),
            Self::SettlementConfirmed(e) => e.rfq_id(),
            Self::SettlementFailed(e) => e.rfq_id(),
        }
    }

    fn timestamp(&self) -> Timestamp {
        match self {
            Self::Executed(e) => e.timestamp(),
            Self::SettlementInitiated(e) => e.timestamp(),
            Self::SettlementConfirmed(e) => e.timestamp(),
            Self::SettlementFailed(e) => e.timestamp(),
        }
    }

    fn event_type(&self) -> EventType {
        match self {
            Self::Executed(e) => e.event_type(),
            Self::SettlementInitiated(e) => e.event_type(),
            Self::SettlementConfirmed(e) => e.event_type(),
            Self::SettlementFailed(e) => e.event_type(),
        }
    }

    fn event_name(&self) -> &'static str {
        match self {
            Self::Executed(e) => e.event_name(),
            Self::SettlementInitiated(e) => e.event_name(),
            Self::SettlementConfirmed(e) => e.event_name(),
            Self::SettlementFailed(e) => e.event_name(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_rfq_id() -> RfqId {
        RfqId::new_v4()
    }

    fn test_trade_id() -> TradeId {
        TradeId::new_v4()
    }

    fn test_quote_id() -> QuoteId {
        QuoteId::new_v4()
    }

    fn test_venue_id() -> VenueId {
        VenueId::new("test-venue")
    }

    fn test_counterparty_id() -> CounterpartyId {
        CounterpartyId::new("test-client")
    }

    mod trade_executed {
        use super::*;

        #[test]
        fn creates_event() {
            let rfq_id = test_rfq_id();
            let trade_id = test_trade_id();
            let event = TradeExecuted::new(
                rfq_id,
                trade_id,
                test_quote_id(),
                test_venue_id(),
                test_counterparty_id(),
                Price::new(50000.0).unwrap(),
                Quantity::new(1.0).unwrap(),
                SettlementMethod::OnChain(Blockchain::Ethereum),
            );

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.trade_id, trade_id);
            assert_eq!(event.event_name(), "TradeExecuted");
            assert_eq!(event.event_type(), EventType::Trade);
        }

        #[test]
        fn serde_roundtrip() {
            let event = TradeExecuted::new(
                test_rfq_id(),
                test_trade_id(),
                test_quote_id(),
                test_venue_id(),
                test_counterparty_id(),
                Price::new(50000.0).unwrap(),
                Quantity::new(1.0).unwrap(),
                SettlementMethod::OffChain,
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: TradeExecuted = serde_json::from_str(&json).unwrap();
            assert_eq!(event.trade_id, deserialized.trade_id);
        }
    }

    mod settlement_initiated {
        use super::*;

        #[test]
        fn on_chain() {
            let event = SettlementInitiated::on_chain(
                test_rfq_id(),
                test_trade_id(),
                Blockchain::Ethereum,
                "0xabc123".to_string(),
            );

            assert_eq!(event.tx_hash, Some("0xabc123".to_string()));
            assert_eq!(
                event.settlement_method,
                SettlementMethod::OnChain(Blockchain::Ethereum)
            );
            assert_eq!(event.event_name(), "SettlementInitiated");
            assert_eq!(event.event_type(), EventType::Settlement);
        }

        #[test]
        fn off_chain() {
            let event = SettlementInitiated::off_chain(test_rfq_id(), test_trade_id());

            assert!(event.tx_hash.is_none());
            assert_eq!(event.settlement_method, SettlementMethod::OffChain);
        }
    }

    mod settlement_confirmed {
        use super::*;

        #[test]
        fn on_chain() {
            let event = SettlementConfirmed::on_chain(
                test_rfq_id(),
                test_trade_id(),
                "0xdef456".to_string(),
                12345678,
            );

            assert_eq!(event.tx_hash, Some("0xdef456".to_string()));
            assert_eq!(event.block_number, Some(12345678));
            assert_eq!(event.event_name(), "SettlementConfirmed");
        }

        #[test]
        fn off_chain() {
            let event = SettlementConfirmed::off_chain(test_rfq_id(), test_trade_id());

            assert!(event.tx_hash.is_none());
            assert!(event.block_number.is_none());
        }
    }

    mod settlement_failed {
        use super::*;

        #[test]
        fn creates_event() {
            let event = SettlementFailed::new(
                test_rfq_id(),
                test_trade_id(),
                "Insufficient funds",
                Some("0xfailed".to_string()),
            );

            assert_eq!(event.reason, "Insufficient funds");
            assert_eq!(event.tx_hash, Some("0xfailed".to_string()));
            assert_eq!(event.event_name(), "SettlementFailed");
        }
    }

    mod trade_event_enum {
        use super::*;

        #[test]
        fn serde_roundtrip() {
            let event = TradeEvent::SettlementConfirmed(SettlementConfirmed::on_chain(
                test_rfq_id(),
                test_trade_id(),
                "0xabc".to_string(),
                100,
            ));

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: TradeEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event.event_name(), deserialized.event_name());
        }

        #[test]
        fn domain_event_trait() {
            let event = TradeEvent::SettlementFailed(SettlementFailed::new(
                test_rfq_id(),
                test_trade_id(),
                "Error",
                None,
            ));

            assert_eq!(event.event_name(), "SettlementFailed");
            assert_eq!(event.event_type(), EventType::Settlement);
        }
    }
}
