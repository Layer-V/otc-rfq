//! # Allocation Events
//!
//! Domain events for multi-MM fill allocation lifecycle.
//!
//! This module provides events that track the allocation of an RFQ's
//! target quantity across multiple market makers.
//!
//! # Event Flow
//!
//! ```text
//! MultiMmFillAllocated -> AllocationExecuted* -> (all done)
//!                      -> AllocationRolledBack* (on failure)
//! ```

use crate::domain::entities::allocation::Allocation;
use crate::domain::events::domain_event::{DomainEvent, EventMetadata, EventType};
use crate::domain::value_objects::size_negotiation_mode::SizeNegotiationMode;
use crate::domain::value_objects::timestamp::Timestamp;
use crate::domain::value_objects::{EventId, Quantity, QuoteId, RfqId, VenueId};
use serde::{Deserialize, Serialize};

/// Event emitted when a multi-MM fill allocation plan is created.
///
/// Contains the full set of allocations and the mode used to compute them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiMmFillAllocated {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The allocations produced by the fill strategy.
    pub allocations: Vec<Allocation>,
    /// The size negotiation mode used.
    pub mode: SizeNegotiationMode,
    /// The target quantity for the fill.
    pub target_quantity: Quantity,
}

impl MultiMmFillAllocated {
    /// Creates a new `MultiMmFillAllocated` event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        allocations: Vec<Allocation>,
        mode: SizeNegotiationMode,
        target_quantity: Quantity,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            allocations,
            mode,
            target_quantity,
        }
    }
}

impl DomainEvent for MultiMmFillAllocated {
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
        "MultiMmFillAllocated"
    }
}

/// Event emitted when a single allocation leg is successfully executed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationExecuted {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The quote ID identifying the executed allocation.
    pub quote_id: QuoteId,
    /// The venue that executed this allocation.
    pub venue_id: VenueId,
    /// The quantity that was executed.
    pub executed_quantity: Quantity,
}

impl AllocationExecuted {
    /// Creates a new `AllocationExecuted` event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        quote_id: QuoteId,
        venue_id: VenueId,
        executed_quantity: Quantity,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            quote_id,
            venue_id,
            executed_quantity,
        }
    }
}

impl DomainEvent for AllocationExecuted {
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
        "AllocationExecuted"
    }
}

/// Event emitted when a single allocation leg is rolled back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllocationRolledBack {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The quote ID identifying the rolled-back allocation.
    pub quote_id: QuoteId,
    /// The venue whose allocation was rolled back.
    pub venue_id: VenueId,
    /// The reason for the rollback.
    pub reason: String,
}

impl AllocationRolledBack {
    /// Creates a new `AllocationRolledBack` event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        quote_id: QuoteId,
        venue_id: VenueId,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            quote_id,
            venue_id,
            reason: reason.into(),
        }
    }
}

impl DomainEvent for AllocationRolledBack {
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
        "AllocationRolledBack"
    }
}

/// Wrapper enum for all allocation-related events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationEvent {
    /// A multi-MM fill allocation plan was created.
    FillAllocated(MultiMmFillAllocated),
    /// A single allocation leg was executed.
    Executed(AllocationExecuted),
    /// A single allocation leg was rolled back.
    RolledBack(AllocationRolledBack),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{Price, Quantity, QuoteId, RfqId, VenueId};

    fn test_rfq_id() -> RfqId {
        RfqId::new_v4()
    }

    fn test_venue() -> VenueId {
        VenueId::new("test-venue")
    }

    fn test_quote_id() -> QuoteId {
        QuoteId::new_v4()
    }

    mod multi_mm_fill_allocated {
        use super::*;

        #[test]
        fn construction() {
            let rfq_id = test_rfq_id();
            let alloc = Allocation::new(
                test_venue(),
                test_quote_id(),
                Quantity::new(5.0).unwrap(),
                Price::new(100.0).unwrap(),
            )
            .unwrap();

            let event = MultiMmFillAllocated::new(
                rfq_id,
                vec![alloc],
                SizeNegotiationMode::AllOrNothing,
                Quantity::new(5.0).unwrap(),
            );

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.event_type(), EventType::Trade);
            assert_eq!(event.event_name(), "MultiMmFillAllocated");
            assert_eq!(event.allocations.len(), 1);
            assert_eq!(event.mode, SizeNegotiationMode::AllOrNothing);
        }

        #[test]
        fn serde_roundtrip() {
            let alloc = Allocation::new(
                test_venue(),
                test_quote_id(),
                Quantity::new(3.0).unwrap(),
                Price::new(50.0).unwrap(),
            )
            .unwrap();

            let event = MultiMmFillAllocated::new(
                test_rfq_id(),
                vec![alloc],
                SizeNegotiationMode::BestEffort,
                Quantity::new(3.0).unwrap(),
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: MultiMmFillAllocated = serde_json::from_str(&json).unwrap();
            assert_eq!(event, deserialized);
        }
    }

    mod allocation_executed {
        use super::*;

        #[test]
        fn construction() {
            let rfq_id = test_rfq_id();
            let quote_id = test_quote_id();
            let venue = test_venue();

            let event = AllocationExecuted::new(
                rfq_id,
                quote_id,
                venue.clone(),
                Quantity::new(2.0).unwrap(),
            );

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.quote_id, quote_id);
            assert_eq!(event.venue_id, venue);
            assert_eq!(event.event_type(), EventType::Trade);
            assert_eq!(event.event_name(), "AllocationExecuted");
        }

        #[test]
        fn serde_roundtrip() {
            let event = AllocationExecuted::new(
                test_rfq_id(),
                test_quote_id(),
                test_venue(),
                Quantity::new(1.0).unwrap(),
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: AllocationExecuted = serde_json::from_str(&json).unwrap();
            assert_eq!(event, deserialized);
        }
    }

    mod allocation_rolled_back {
        use super::*;

        #[test]
        fn construction() {
            let rfq_id = test_rfq_id();
            let quote_id = test_quote_id();
            let venue = test_venue();

            let event = AllocationRolledBack::new(rfq_id, quote_id, venue.clone(), "venue timeout");

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.quote_id, quote_id);
            assert_eq!(event.venue_id, venue);
            assert_eq!(event.reason, "venue timeout");
            assert_eq!(event.event_type(), EventType::Trade);
            assert_eq!(event.event_name(), "AllocationRolledBack");
        }

        #[test]
        fn serde_roundtrip() {
            let event = AllocationRolledBack::new(
                test_rfq_id(),
                test_quote_id(),
                test_venue(),
                "execution failed",
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: AllocationRolledBack = serde_json::from_str(&json).unwrap();
            assert_eq!(event, deserialized);
        }
    }

    mod allocation_event_enum {
        use super::*;

        #[test]
        fn fill_allocated_variant() {
            let alloc = Allocation::new(
                test_venue(),
                test_quote_id(),
                Quantity::new(1.0).unwrap(),
                Price::new(10.0).unwrap(),
            )
            .unwrap();

            let event = AllocationEvent::FillAllocated(MultiMmFillAllocated::new(
                test_rfq_id(),
                vec![alloc],
                SizeNegotiationMode::AllOrNothing,
                Quantity::new(1.0).unwrap(),
            ));

            assert!(matches!(event, AllocationEvent::FillAllocated(_)));
        }

        #[test]
        fn executed_variant() {
            let event = AllocationEvent::Executed(AllocationExecuted::new(
                test_rfq_id(),
                test_quote_id(),
                test_venue(),
                Quantity::new(1.0).unwrap(),
            ));

            assert!(matches!(event, AllocationEvent::Executed(_)));
        }

        #[test]
        fn rolled_back_variant() {
            let event = AllocationEvent::RolledBack(AllocationRolledBack::new(
                test_rfq_id(),
                test_quote_id(),
                test_venue(),
                "timeout",
            ));

            assert!(matches!(event, AllocationEvent::RolledBack(_)));
        }
    }
}
