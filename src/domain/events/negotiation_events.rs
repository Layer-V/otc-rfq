//! # Negotiation Events
//!
//! Domain events for the counter-quote negotiation lifecycle.
//!
//! This module provides events that track the lifecycle of a negotiation
//! from counter-quote submission through acceptance, rejection, or expiry.
//!
//! # Event Flow
//!
//! ```text
//! CounterQuoteSent -> CounterQuoteReceived -> (repeat)
//!                  -> NegotiationCompleted (accepted | rejected | expired)
//! ```

use crate::domain::events::domain_event::{DomainEvent, EventMetadata, EventType};
use crate::domain::value_objects::negotiation_state::NegotiationState;
use crate::domain::value_objects::timestamp::Timestamp;
use crate::domain::value_objects::{
    CounterpartyId, EventId, NegotiationId, Price, Quantity, QuoteId, RfqId,
};
use serde::{Deserialize, Serialize};

/// Event emitted when a counter-quote is submitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterQuoteSent {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The negotiation ID.
    pub negotiation_id: NegotiationId,
    /// The counter-quote ID.
    pub counter_quote_id: QuoteId,
    /// The original quote being countered.
    pub original_quote_id: QuoteId,
    /// Who submitted the counter.
    pub from_account: CounterpartyId,
    /// The proposed price.
    pub price: Price,
    /// The proposed quantity.
    pub quantity: Quantity,
    /// The negotiation round number.
    pub round: u8,
}

impl CounterQuoteSent {
    /// Creates a new CounterQuoteSent event.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rfq_id: RfqId,
        negotiation_id: NegotiationId,
        counter_quote_id: QuoteId,
        original_quote_id: QuoteId,
        from_account: CounterpartyId,
        price: Price,
        quantity: Quantity,
        round: u8,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            negotiation_id,
            counter_quote_id,
            original_quote_id,
            from_account,
            price,
            quantity,
            round,
        }
    }
}

impl DomainEvent for CounterQuoteSent {
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
        "CounterQuoteSent"
    }
}

/// Event emitted when a counter-quote is received by the other party.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterQuoteReceived {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The negotiation ID.
    pub negotiation_id: NegotiationId,
    /// The counter-quote ID.
    pub counter_quote_id: QuoteId,
    /// Who received the counter.
    pub to_account: CounterpartyId,
    /// The proposed price.
    pub price: Price,
    /// The negotiation round number.
    pub round: u8,
}

impl CounterQuoteReceived {
    /// Creates a new CounterQuoteReceived event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        negotiation_id: NegotiationId,
        counter_quote_id: QuoteId,
        to_account: CounterpartyId,
        price: Price,
        round: u8,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            negotiation_id,
            counter_quote_id,
            to_account,
            price,
            round,
        }
    }
}

impl DomainEvent for CounterQuoteReceived {
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
        "CounterQuoteReceived"
    }
}

/// Outcome of a negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NegotiationOutcome {
    /// Both parties agreed on terms.
    Accepted,
    /// One party rejected the negotiation.
    Rejected,
    /// The negotiation timed out.
    Expired,
}

impl std::fmt::Display for NegotiationOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accepted => write!(f, "ACCEPTED"),
            Self::Rejected => write!(f, "REJECTED"),
            Self::Expired => write!(f, "EXPIRED"),
        }
    }
}

/// Event emitted when a negotiation is completed (accepted, rejected, or expired).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiationCompleted {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The negotiation ID.
    pub negotiation_id: NegotiationId,
    /// The outcome of the negotiation.
    pub outcome: NegotiationOutcome,
    /// The final negotiation state.
    pub final_state: NegotiationState,
    /// The final agreed price, if accepted.
    pub final_price: Option<Price>,
    /// Total rounds completed.
    pub total_rounds: u8,
}

impl NegotiationCompleted {
    /// Creates a new NegotiationCompleted event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        negotiation_id: NegotiationId,
        outcome: NegotiationOutcome,
        final_state: NegotiationState,
        final_price: Option<Price>,
        total_rounds: u8,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            negotiation_id,
            outcome,
            final_state,
            final_price,
            total_rounds,
        }
    }
}

impl DomainEvent for NegotiationCompleted {
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
        "NegotiationCompleted"
    }
}

/// Enum containing all negotiation-related events.
///
/// This enum allows for type-safe handling of all negotiation events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NegotiationEvent {
    /// A counter-quote was sent.
    CounterQuoteSent(CounterQuoteSent),
    /// A counter-quote was received by the other party.
    CounterQuoteReceived(CounterQuoteReceived),
    /// The negotiation was completed.
    NegotiationCompleted(NegotiationCompleted),
}

impl DomainEvent for NegotiationEvent {
    fn event_id(&self) -> EventId {
        match self {
            Self::CounterQuoteSent(e) => e.event_id(),
            Self::CounterQuoteReceived(e) => e.event_id(),
            Self::NegotiationCompleted(e) => e.event_id(),
        }
    }

    fn rfq_id(&self) -> Option<RfqId> {
        match self {
            Self::CounterQuoteSent(e) => e.rfq_id(),
            Self::CounterQuoteReceived(e) => e.rfq_id(),
            Self::NegotiationCompleted(e) => e.rfq_id(),
        }
    }

    fn timestamp(&self) -> Timestamp {
        match self {
            Self::CounterQuoteSent(e) => e.timestamp(),
            Self::CounterQuoteReceived(e) => e.timestamp(),
            Self::NegotiationCompleted(e) => e.timestamp(),
        }
    }

    fn event_type(&self) -> EventType {
        match self {
            Self::CounterQuoteSent(e) => e.event_type(),
            Self::CounterQuoteReceived(e) => e.event_type(),
            Self::NegotiationCompleted(e) => e.event_type(),
        }
    }

    fn event_name(&self) -> &'static str {
        match self {
            Self::CounterQuoteSent(e) => e.event_name(),
            Self::CounterQuoteReceived(e) => e.event_name(),
            Self::NegotiationCompleted(e) => e.event_name(),
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

    fn test_negotiation_id() -> NegotiationId {
        NegotiationId::new_v4()
    }

    mod counter_quote_sent {
        use super::*;

        #[test]
        fn creates_event() {
            let rfq_id = test_rfq_id();
            let neg_id = test_negotiation_id();
            let event = CounterQuoteSent::new(
                rfq_id,
                neg_id,
                QuoteId::new_v4(),
                QuoteId::new_v4(),
                CounterpartyId::new("mm-1"),
                Price::new(50000.0).unwrap(),
                Quantity::new(1.0).unwrap(),
                1,
            );

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.negotiation_id, neg_id);
            assert_eq!(event.round, 1);
            assert_eq!(event.event_name(), "CounterQuoteSent");
            assert_eq!(event.event_type(), EventType::Quote);
        }

        #[test]
        fn serde_roundtrip() {
            let event = CounterQuoteSent::new(
                test_rfq_id(),
                test_negotiation_id(),
                QuoteId::new_v4(),
                QuoteId::new_v4(),
                CounterpartyId::new("mm-1"),
                Price::new(50000.0).unwrap(),
                Quantity::new(1.0).unwrap(),
                1,
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: CounterQuoteSent = serde_json::from_str(&json).unwrap();
            assert_eq!(event.metadata.event_id, deserialized.metadata.event_id);
        }
    }

    mod counter_quote_received {
        use super::*;

        #[test]
        fn creates_event() {
            let rfq_id = test_rfq_id();
            let event = CounterQuoteReceived::new(
                rfq_id,
                test_negotiation_id(),
                QuoteId::new_v4(),
                CounterpartyId::new("client-1"),
                Price::new(49500.0).unwrap(),
                2,
            );

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.round, 2);
            assert_eq!(event.event_name(), "CounterQuoteReceived");
        }
    }

    mod negotiation_completed {
        use super::*;

        #[test]
        fn creates_accepted_event() {
            let rfq_id = test_rfq_id();
            let event = NegotiationCompleted::new(
                rfq_id,
                test_negotiation_id(),
                NegotiationOutcome::Accepted,
                NegotiationState::Accepted,
                Some(Price::new(49500.0).unwrap()),
                3,
            );

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.outcome, NegotiationOutcome::Accepted);
            assert!(event.final_price.is_some());
            assert_eq!(event.total_rounds, 3);
            assert_eq!(event.event_name(), "NegotiationCompleted");
            assert_eq!(event.event_type(), EventType::Rfq);
        }

        #[test]
        fn creates_rejected_event() {
            let event = NegotiationCompleted::new(
                test_rfq_id(),
                test_negotiation_id(),
                NegotiationOutcome::Rejected,
                NegotiationState::Rejected,
                None,
                1,
            );

            assert_eq!(event.outcome, NegotiationOutcome::Rejected);
            assert!(event.final_price.is_none());
        }

        #[test]
        fn creates_expired_event() {
            let event = NegotiationCompleted::new(
                test_rfq_id(),
                test_negotiation_id(),
                NegotiationOutcome::Expired,
                NegotiationState::Expired,
                None,
                2,
            );

            assert_eq!(event.outcome, NegotiationOutcome::Expired);
        }

        #[test]
        fn serde_roundtrip() {
            let event = NegotiationCompleted::new(
                test_rfq_id(),
                test_negotiation_id(),
                NegotiationOutcome::Accepted,
                NegotiationState::Accepted,
                Some(Price::new(49500.0).unwrap()),
                3,
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: NegotiationCompleted = serde_json::from_str(&json).unwrap();
            assert_eq!(event.outcome, deserialized.outcome);
            assert_eq!(event.total_rounds, deserialized.total_rounds);
        }
    }

    mod negotiation_event_enum {
        use super::*;

        #[test]
        fn serde_roundtrip() {
            let event = NegotiationEvent::CounterQuoteSent(CounterQuoteSent::new(
                test_rfq_id(),
                test_negotiation_id(),
                QuoteId::new_v4(),
                QuoteId::new_v4(),
                CounterpartyId::new("mm-1"),
                Price::new(50000.0).unwrap(),
                Quantity::new(1.0).unwrap(),
                1,
            ));

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: NegotiationEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event.event_name(), deserialized.event_name());
        }

        #[test]
        fn domain_event_trait() {
            let event = NegotiationEvent::NegotiationCompleted(NegotiationCompleted::new(
                test_rfq_id(),
                test_negotiation_id(),
                NegotiationOutcome::Accepted,
                NegotiationState::Accepted,
                Some(Price::new(49500.0).unwrap()),
                3,
            ));

            assert_eq!(event.event_name(), "NegotiationCompleted");
            assert_eq!(event.event_type(), EventType::Rfq);
        }
    }

    mod outcome_display {
        use super::*;

        #[test]
        fn display_formats() {
            assert_eq!(NegotiationOutcome::Accepted.to_string(), "ACCEPTED");
            assert_eq!(NegotiationOutcome::Rejected.to_string(), "REJECTED");
            assert_eq!(NegotiationOutcome::Expired.to_string(), "EXPIRED");
        }
    }
}
