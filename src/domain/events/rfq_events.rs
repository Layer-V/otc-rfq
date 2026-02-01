//! # RFQ Events
//!
//! Domain events for RFQ lifecycle.
//!
//! This module provides events that track the lifecycle of an RFQ from
//! creation through quote collection, selection, and execution.
//!
//! # Event Flow
//!
//! ```text
//! RfqCreated -> QuoteCollectionStarted -> QuoteRequested* -> QuoteReceived*
//!            -> QuoteCollectionCompleted -> QuoteSelected -> ExecutionStarted
//!            -> TradeExecuted | ExecutionFailed
//!
//! At any point: RfqCancelled | RfqExpired
//! ```

use crate::domain::events::domain_event::{DomainEvent, EventMetadata, EventType};
use crate::domain::value_objects::timestamp::Timestamp;
use crate::domain::value_objects::{
    CounterpartyId, EventId, Instrument, OrderSide, Price, Quantity, QuoteId, RfqId, RfqState,
    VenueId,
};
use serde::{Deserialize, Serialize};

/// Event emitted when a new RFQ is created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RfqCreated {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The client who created the RFQ.
    pub client_id: CounterpartyId,
    /// The instrument being traded.
    pub instrument: Instrument,
    /// Buy or sell.
    pub side: OrderSide,
    /// Requested quantity.
    pub quantity: Quantity,
    /// When the RFQ expires.
    pub expires_at: Timestamp,
}

impl RfqCreated {
    /// Creates a new RfqCreated event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        client_id: CounterpartyId,
        instrument: Instrument,
        side: OrderSide,
        quantity: Quantity,
        expires_at: Timestamp,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            client_id,
            instrument,
            side,
            quantity,
            expires_at,
        }
    }
}

impl DomainEvent for RfqCreated {
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
        EventType::Rfq
    }

    fn event_name(&self) -> &'static str {
        "RfqCreated"
    }
}

/// Event emitted when quote collection starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteCollectionStarted {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Venues being queried.
    pub venue_ids: Vec<VenueId>,
}

impl QuoteCollectionStarted {
    /// Creates a new QuoteCollectionStarted event.
    #[must_use]
    pub fn new(rfq_id: RfqId, venue_ids: Vec<VenueId>) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            venue_ids,
        }
    }
}

impl DomainEvent for QuoteCollectionStarted {
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
        EventType::Quote
    }

    fn event_name(&self) -> &'static str {
        "QuoteCollectionStarted"
    }
}

/// Event emitted when a quote is requested from a venue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteRequested {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The venue being queried.
    pub venue_id: VenueId,
}

impl QuoteRequested {
    /// Creates a new QuoteRequested event.
    #[must_use]
    pub fn new(rfq_id: RfqId, venue_id: VenueId) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            venue_id,
        }
    }
}

impl DomainEvent for QuoteRequested {
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
        EventType::Quote
    }

    fn event_name(&self) -> &'static str {
        "QuoteRequested"
    }
}

/// Event emitted when a quote is received from a venue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteReceived {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The quote ID.
    pub quote_id: QuoteId,
    /// The venue that provided the quote.
    pub venue_id: VenueId,
    /// The quoted price.
    pub price: Price,
    /// The quoted quantity.
    pub quantity: Quantity,
    /// When the quote expires.
    pub valid_until: Timestamp,
}

impl QuoteReceived {
    /// Creates a new QuoteReceived event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        quote_id: QuoteId,
        venue_id: VenueId,
        price: Price,
        quantity: Quantity,
        valid_until: Timestamp,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            quote_id,
            venue_id,
            price,
            quantity,
            valid_until,
        }
    }
}

impl DomainEvent for QuoteReceived {
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
        EventType::Quote
    }

    fn event_name(&self) -> &'static str {
        "QuoteReceived"
    }
}

/// Event emitted when a quote request fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteRequestFailed {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The venue that failed.
    pub venue_id: VenueId,
    /// Reason for failure.
    pub reason: String,
}

impl QuoteRequestFailed {
    /// Creates a new QuoteRequestFailed event.
    #[must_use]
    pub fn new(rfq_id: RfqId, venue_id: VenueId, reason: impl Into<String>) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            venue_id,
            reason: reason.into(),
        }
    }
}

impl DomainEvent for QuoteRequestFailed {
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
        EventType::Quote
    }

    fn event_name(&self) -> &'static str {
        "QuoteRequestFailed"
    }
}

/// Event emitted when quote collection is complete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteCollectionCompleted {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// Number of quotes received.
    pub quotes_received: u32,
    /// Number of venues that failed.
    pub venues_failed: u32,
}

impl QuoteCollectionCompleted {
    /// Creates a new QuoteCollectionCompleted event.
    #[must_use]
    pub fn new(rfq_id: RfqId, quotes_received: u32, venues_failed: u32) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            quotes_received,
            venues_failed,
        }
    }
}

impl DomainEvent for QuoteCollectionCompleted {
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
        EventType::Quote
    }

    fn event_name(&self) -> &'static str {
        "QuoteCollectionCompleted"
    }
}

/// Event emitted when a quote is selected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteSelected {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The selected quote ID.
    pub quote_id: QuoteId,
    /// The venue providing the selected quote.
    pub venue_id: VenueId,
    /// The selected price.
    pub price: Price,
}

impl QuoteSelected {
    /// Creates a new QuoteSelected event.
    #[must_use]
    pub fn new(rfq_id: RfqId, quote_id: QuoteId, venue_id: VenueId, price: Price) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            quote_id,
            venue_id,
            price,
        }
    }
}

impl DomainEvent for QuoteSelected {
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
        EventType::Quote
    }

    fn event_name(&self) -> &'static str {
        "QuoteSelected"
    }
}

/// Event emitted when execution starts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionStarted {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The quote being executed.
    pub quote_id: QuoteId,
    /// The venue executing the trade.
    pub venue_id: VenueId,
}

impl ExecutionStarted {
    /// Creates a new ExecutionStarted event.
    #[must_use]
    pub fn new(rfq_id: RfqId, quote_id: QuoteId, venue_id: VenueId) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            quote_id,
            venue_id,
        }
    }
}

impl DomainEvent for ExecutionStarted {
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
        "ExecutionStarted"
    }
}

/// Event emitted when execution fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionFailed {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The quote that failed to execute.
    pub quote_id: QuoteId,
    /// Reason for failure.
    pub reason: String,
}

impl ExecutionFailed {
    /// Creates a new ExecutionFailed event.
    #[must_use]
    pub fn new(rfq_id: RfqId, quote_id: QuoteId, reason: impl Into<String>) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            quote_id,
            reason: reason.into(),
        }
    }
}

impl DomainEvent for ExecutionFailed {
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
        "ExecutionFailed"
    }
}

/// Event emitted when an RFQ is cancelled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RfqCancelled {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The state the RFQ was in when cancelled.
    pub previous_state: RfqState,
    /// Reason for cancellation.
    pub reason: Option<String>,
}

impl RfqCancelled {
    /// Creates a new RfqCancelled event.
    #[must_use]
    pub fn new(rfq_id: RfqId, previous_state: RfqState, reason: Option<String>) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            previous_state,
            reason,
        }
    }
}

impl DomainEvent for RfqCancelled {
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
        EventType::Rfq
    }

    fn event_name(&self) -> &'static str {
        "RfqCancelled"
    }
}

/// Event emitted when an RFQ expires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RfqExpired {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The state the RFQ was in when it expired.
    pub previous_state: RfqState,
}

impl RfqExpired {
    /// Creates a new RfqExpired event.
    #[must_use]
    pub fn new(rfq_id: RfqId, previous_state: RfqState) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            previous_state,
        }
    }
}

impl DomainEvent for RfqExpired {
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
        EventType::Rfq
    }

    fn event_name(&self) -> &'static str {
        "RfqExpired"
    }
}

/// Enum containing all RFQ-related events.
///
/// This enum allows for type-safe handling of all RFQ events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RfqEvent {
    /// RFQ was created.
    Created(RfqCreated),
    /// Quote collection started.
    QuoteCollectionStarted(QuoteCollectionStarted),
    /// Quote was requested from a venue.
    QuoteRequested(QuoteRequested),
    /// Quote was received from a venue.
    QuoteReceived(QuoteReceived),
    /// Quote request failed.
    QuoteRequestFailed(QuoteRequestFailed),
    /// Quote collection completed.
    QuoteCollectionCompleted(QuoteCollectionCompleted),
    /// Quote was selected.
    QuoteSelected(QuoteSelected),
    /// Execution started.
    ExecutionStarted(ExecutionStarted),
    /// Execution failed.
    ExecutionFailed(ExecutionFailed),
    /// RFQ was cancelled.
    Cancelled(RfqCancelled),
    /// RFQ expired.
    Expired(RfqExpired),
}

impl DomainEvent for RfqEvent {
    fn event_id(&self) -> EventId {
        match self {
            Self::Created(e) => e.event_id(),
            Self::QuoteCollectionStarted(e) => e.event_id(),
            Self::QuoteRequested(e) => e.event_id(),
            Self::QuoteReceived(e) => e.event_id(),
            Self::QuoteRequestFailed(e) => e.event_id(),
            Self::QuoteCollectionCompleted(e) => e.event_id(),
            Self::QuoteSelected(e) => e.event_id(),
            Self::ExecutionStarted(e) => e.event_id(),
            Self::ExecutionFailed(e) => e.event_id(),
            Self::Cancelled(e) => e.event_id(),
            Self::Expired(e) => e.event_id(),
        }
    }

    fn rfq_id(&self) -> Option<RfqId> {
        match self {
            Self::Created(e) => e.rfq_id(),
            Self::QuoteCollectionStarted(e) => e.rfq_id(),
            Self::QuoteRequested(e) => e.rfq_id(),
            Self::QuoteReceived(e) => e.rfq_id(),
            Self::QuoteRequestFailed(e) => e.rfq_id(),
            Self::QuoteCollectionCompleted(e) => e.rfq_id(),
            Self::QuoteSelected(e) => e.rfq_id(),
            Self::ExecutionStarted(e) => e.rfq_id(),
            Self::ExecutionFailed(e) => e.rfq_id(),
            Self::Cancelled(e) => e.rfq_id(),
            Self::Expired(e) => e.rfq_id(),
        }
    }

    fn timestamp(&self) -> Timestamp {
        match self {
            Self::Created(e) => e.timestamp(),
            Self::QuoteCollectionStarted(e) => e.timestamp(),
            Self::QuoteRequested(e) => e.timestamp(),
            Self::QuoteReceived(e) => e.timestamp(),
            Self::QuoteRequestFailed(e) => e.timestamp(),
            Self::QuoteCollectionCompleted(e) => e.timestamp(),
            Self::QuoteSelected(e) => e.timestamp(),
            Self::ExecutionStarted(e) => e.timestamp(),
            Self::ExecutionFailed(e) => e.timestamp(),
            Self::Cancelled(e) => e.timestamp(),
            Self::Expired(e) => e.timestamp(),
        }
    }

    fn event_type(&self) -> EventType {
        match self {
            Self::Created(e) => e.event_type(),
            Self::QuoteCollectionStarted(e) => e.event_type(),
            Self::QuoteRequested(e) => e.event_type(),
            Self::QuoteReceived(e) => e.event_type(),
            Self::QuoteRequestFailed(e) => e.event_type(),
            Self::QuoteCollectionCompleted(e) => e.event_type(),
            Self::QuoteSelected(e) => e.event_type(),
            Self::ExecutionStarted(e) => e.event_type(),
            Self::ExecutionFailed(e) => e.event_type(),
            Self::Cancelled(e) => e.event_type(),
            Self::Expired(e) => e.event_type(),
        }
    }

    fn event_name(&self) -> &'static str {
        match self {
            Self::Created(e) => e.event_name(),
            Self::QuoteCollectionStarted(e) => e.event_name(),
            Self::QuoteRequested(e) => e.event_name(),
            Self::QuoteReceived(e) => e.event_name(),
            Self::QuoteRequestFailed(e) => e.event_name(),
            Self::QuoteCollectionCompleted(e) => e.event_name(),
            Self::QuoteSelected(e) => e.event_name(),
            Self::ExecutionStarted(e) => e.event_name(),
            Self::ExecutionFailed(e) => e.event_name(),
            Self::Cancelled(e) => e.event_name(),
            Self::Expired(e) => e.event_name(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{AssetClass, Symbol};

    fn test_rfq_id() -> RfqId {
        RfqId::new_v4()
    }

    fn test_venue_id() -> VenueId {
        VenueId::new("test-venue")
    }

    fn test_client_id() -> CounterpartyId {
        CounterpartyId::new("test-client")
    }

    fn test_instrument() -> Instrument {
        Instrument::builder(Symbol::new("BTC/USD").unwrap(), AssetClass::CryptoSpot).build()
    }

    mod rfq_created {
        use super::*;

        #[test]
        fn creates_event() {
            let rfq_id = test_rfq_id();
            let event = RfqCreated::new(
                rfq_id,
                test_client_id(),
                test_instrument(),
                OrderSide::Buy,
                Quantity::new(100.0).unwrap(),
                Timestamp::now().add_secs(300),
            );

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.event_name(), "RfqCreated");
            assert_eq!(event.event_type(), EventType::Rfq);
        }

        #[test]
        fn serde_roundtrip() {
            let event = RfqCreated::new(
                test_rfq_id(),
                test_client_id(),
                test_instrument(),
                OrderSide::Buy,
                Quantity::new(100.0).unwrap(),
                Timestamp::now().add_secs(300),
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: RfqCreated = serde_json::from_str(&json).unwrap();
            assert_eq!(event.metadata.event_id, deserialized.metadata.event_id);
        }
    }

    mod quote_events {
        use super::*;

        #[test]
        fn quote_collection_started() {
            let rfq_id = test_rfq_id();
            let event = QuoteCollectionStarted::new(rfq_id, vec![test_venue_id()]);

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.event_name(), "QuoteCollectionStarted");
            assert_eq!(event.venue_ids.len(), 1);
        }

        #[test]
        fn quote_requested() {
            let rfq_id = test_rfq_id();
            let venue_id = test_venue_id();
            let event = QuoteRequested::new(rfq_id, venue_id.clone());

            assert_eq!(event.venue_id, venue_id);
            assert_eq!(event.event_name(), "QuoteRequested");
        }

        #[test]
        fn quote_received() {
            let rfq_id = test_rfq_id();
            let quote_id = QuoteId::new_v4();
            let event = QuoteReceived::new(
                rfq_id,
                quote_id,
                test_venue_id(),
                Price::new(50000.0).unwrap(),
                Quantity::new(1.0).unwrap(),
                Timestamp::now().add_secs(60),
            );

            assert_eq!(event.quote_id, quote_id);
            assert_eq!(event.event_name(), "QuoteReceived");
        }

        #[test]
        fn quote_request_failed() {
            let event = QuoteRequestFailed::new(test_rfq_id(), test_venue_id(), "Timeout");

            assert_eq!(event.reason, "Timeout");
            assert_eq!(event.event_name(), "QuoteRequestFailed");
        }

        #[test]
        fn quote_collection_completed() {
            let event = QuoteCollectionCompleted::new(test_rfq_id(), 3, 1);

            assert_eq!(event.quotes_received, 3);
            assert_eq!(event.venues_failed, 1);
            assert_eq!(event.event_name(), "QuoteCollectionCompleted");
        }

        #[test]
        fn quote_selected() {
            let quote_id = QuoteId::new_v4();
            let event = QuoteSelected::new(
                test_rfq_id(),
                quote_id,
                test_venue_id(),
                Price::new(50000.0).unwrap(),
            );

            assert_eq!(event.quote_id, quote_id);
            assert_eq!(event.event_name(), "QuoteSelected");
        }
    }

    mod execution_events {
        use super::*;

        #[test]
        fn execution_started() {
            let quote_id = QuoteId::new_v4();
            let event = ExecutionStarted::new(test_rfq_id(), quote_id, test_venue_id());

            assert_eq!(event.quote_id, quote_id);
            assert_eq!(event.event_name(), "ExecutionStarted");
            assert_eq!(event.event_type(), EventType::Trade);
        }

        #[test]
        fn execution_failed() {
            let quote_id = QuoteId::new_v4();
            let event = ExecutionFailed::new(test_rfq_id(), quote_id, "Insufficient liquidity");

            assert_eq!(event.reason, "Insufficient liquidity");
            assert_eq!(event.event_name(), "ExecutionFailed");
        }
    }

    mod rfq_lifecycle {
        use super::*;

        #[test]
        fn rfq_cancelled() {
            let event = RfqCancelled::new(
                test_rfq_id(),
                RfqState::QuotesReceived,
                Some("Client requested".to_string()),
            );

            assert_eq!(event.previous_state, RfqState::QuotesReceived);
            assert_eq!(event.reason, Some("Client requested".to_string()));
            assert_eq!(event.event_name(), "RfqCancelled");
        }

        #[test]
        fn rfq_expired() {
            let event = RfqExpired::new(test_rfq_id(), RfqState::QuoteRequesting);

            assert_eq!(event.previous_state, RfqState::QuoteRequesting);
            assert_eq!(event.event_name(), "RfqExpired");
        }
    }

    mod rfq_event_enum {
        use super::*;

        #[test]
        fn serde_roundtrip() {
            let event = RfqEvent::Created(RfqCreated::new(
                test_rfq_id(),
                test_client_id(),
                test_instrument(),
                OrderSide::Buy,
                Quantity::new(100.0).unwrap(),
                Timestamp::now().add_secs(300),
            ));

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: RfqEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event.event_name(), deserialized.event_name());
        }

        #[test]
        fn domain_event_trait() {
            let event = RfqEvent::QuoteCollectionStarted(QuoteCollectionStarted::new(
                test_rfq_id(),
                vec![test_venue_id()],
            ));

            assert_eq!(event.event_name(), "QuoteCollectionStarted");
            assert_eq!(event.event_type(), EventType::Quote);
        }
    }
}
