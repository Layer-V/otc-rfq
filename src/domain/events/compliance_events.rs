//! # Compliance Events
//!
//! Domain events for compliance checks.
//!
//! This module provides events that track compliance verification
//! during the RFQ lifecycle.

use crate::domain::events::domain_event::{DomainEvent, EventMetadata, EventType};
use crate::domain::value_objects::timestamp::Timestamp;
use crate::domain::value_objects::{CounterpartyId, EventId, RfqId};
use serde::{Deserialize, Serialize};

/// Type of compliance check performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceCheckType {
    /// Know Your Customer verification.
    Kyc,
    /// Anti-Money Laundering check.
    Aml,
    /// Sanctions screening.
    Sanctions,
    /// Trading limits check.
    TradingLimits,
    /// Instrument eligibility check.
    InstrumentEligibility,
    /// General compliance check.
    General,
}

impl std::fmt::Display for ComplianceCheckType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kyc => write!(f, "KYC"),
            Self::Aml => write!(f, "AML"),
            Self::Sanctions => write!(f, "SANCTIONS"),
            Self::TradingLimits => write!(f, "TRADING_LIMITS"),
            Self::InstrumentEligibility => write!(f, "INSTRUMENT_ELIGIBILITY"),
            Self::General => write!(f, "GENERAL"),
        }
    }
}

/// Event emitted when a compliance check passes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceCheckPassed {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The counterparty that was checked.
    pub counterparty_id: CounterpartyId,
    /// Type of compliance check.
    pub check_type: ComplianceCheckType,
    /// Optional details about the check.
    pub details: Option<String>,
}

impl ComplianceCheckPassed {
    /// Creates a new ComplianceCheckPassed event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        counterparty_id: CounterpartyId,
        check_type: ComplianceCheckType,
        details: Option<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            counterparty_id,
            check_type,
            details,
        }
    }

    /// Creates a KYC passed event.
    #[must_use]
    pub fn kyc(rfq_id: RfqId, counterparty_id: CounterpartyId) -> Self {
        Self::new(rfq_id, counterparty_id, ComplianceCheckType::Kyc, None)
    }

    /// Creates an AML passed event.
    #[must_use]
    pub fn aml(rfq_id: RfqId, counterparty_id: CounterpartyId) -> Self {
        Self::new(rfq_id, counterparty_id, ComplianceCheckType::Aml, None)
    }

    /// Creates a sanctions check passed event.
    #[must_use]
    pub fn sanctions(rfq_id: RfqId, counterparty_id: CounterpartyId) -> Self {
        Self::new(
            rfq_id,
            counterparty_id,
            ComplianceCheckType::Sanctions,
            None,
        )
    }

    /// Creates a trading limits check passed event.
    #[must_use]
    pub fn trading_limits(rfq_id: RfqId, counterparty_id: CounterpartyId) -> Self {
        Self::new(
            rfq_id,
            counterparty_id,
            ComplianceCheckType::TradingLimits,
            None,
        )
    }
}

impl DomainEvent for ComplianceCheckPassed {
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
        EventType::Compliance
    }

    fn event_name(&self) -> &'static str {
        "ComplianceCheckPassed"
    }
}

/// Event emitted when a compliance check fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceCheckFailed {
    /// Event metadata.
    pub metadata: EventMetadata,
    /// The counterparty that was checked.
    pub counterparty_id: CounterpartyId,
    /// Type of compliance check.
    pub check_type: ComplianceCheckType,
    /// Reason for failure.
    pub reason: String,
    /// Error code (if applicable).
    pub error_code: Option<String>,
}

impl ComplianceCheckFailed {
    /// Creates a new ComplianceCheckFailed event.
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        counterparty_id: CounterpartyId,
        check_type: ComplianceCheckType,
        reason: impl Into<String>,
        error_code: Option<String>,
    ) -> Self {
        Self {
            metadata: EventMetadata::for_rfq(rfq_id),
            counterparty_id,
            check_type,
            reason: reason.into(),
            error_code,
        }
    }

    /// Creates a KYC failed event.
    #[must_use]
    pub fn kyc(rfq_id: RfqId, counterparty_id: CounterpartyId, reason: impl Into<String>) -> Self {
        Self::new(
            rfq_id,
            counterparty_id,
            ComplianceCheckType::Kyc,
            reason,
            None,
        )
    }

    /// Creates an AML failed event.
    #[must_use]
    pub fn aml(rfq_id: RfqId, counterparty_id: CounterpartyId, reason: impl Into<String>) -> Self {
        Self::new(
            rfq_id,
            counterparty_id,
            ComplianceCheckType::Aml,
            reason,
            None,
        )
    }

    /// Creates a sanctions check failed event.
    #[must_use]
    pub fn sanctions(
        rfq_id: RfqId,
        counterparty_id: CounterpartyId,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(
            rfq_id,
            counterparty_id,
            ComplianceCheckType::Sanctions,
            reason,
            None,
        )
    }

    /// Creates a trading limits check failed event.
    #[must_use]
    pub fn trading_limits(
        rfq_id: RfqId,
        counterparty_id: CounterpartyId,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(
            rfq_id,
            counterparty_id,
            ComplianceCheckType::TradingLimits,
            reason,
            None,
        )
    }
}

impl DomainEvent for ComplianceCheckFailed {
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
        EventType::Compliance
    }

    fn event_name(&self) -> &'static str {
        "ComplianceCheckFailed"
    }
}

/// Enum containing all compliance events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ComplianceEvent {
    /// Compliance check passed.
    Passed(ComplianceCheckPassed),
    /// Compliance check failed.
    Failed(ComplianceCheckFailed),
}

impl DomainEvent for ComplianceEvent {
    fn event_id(&self) -> EventId {
        match self {
            Self::Passed(e) => e.event_id(),
            Self::Failed(e) => e.event_id(),
        }
    }

    fn rfq_id(&self) -> Option<RfqId> {
        match self {
            Self::Passed(e) => e.rfq_id(),
            Self::Failed(e) => e.rfq_id(),
        }
    }

    fn timestamp(&self) -> Timestamp {
        match self {
            Self::Passed(e) => e.timestamp(),
            Self::Failed(e) => e.timestamp(),
        }
    }

    fn event_type(&self) -> EventType {
        match self {
            Self::Passed(e) => e.event_type(),
            Self::Failed(e) => e.event_type(),
        }
    }

    fn event_name(&self) -> &'static str {
        match self {
            Self::Passed(e) => e.event_name(),
            Self::Failed(e) => e.event_name(),
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

    fn test_counterparty_id() -> CounterpartyId {
        CounterpartyId::new("test-client")
    }

    mod compliance_check_type {
        use super::*;

        #[test]
        fn display() {
            assert_eq!(ComplianceCheckType::Kyc.to_string(), "KYC");
            assert_eq!(ComplianceCheckType::Aml.to_string(), "AML");
            assert_eq!(ComplianceCheckType::Sanctions.to_string(), "SANCTIONS");
            assert_eq!(
                ComplianceCheckType::TradingLimits.to_string(),
                "TRADING_LIMITS"
            );
        }
    }

    mod compliance_check_passed {
        use super::*;

        #[test]
        fn creates_event() {
            let rfq_id = test_rfq_id();
            let event = ComplianceCheckPassed::kyc(rfq_id, test_counterparty_id());

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.check_type, ComplianceCheckType::Kyc);
            assert_eq!(event.event_name(), "ComplianceCheckPassed");
            assert_eq!(event.event_type(), EventType::Compliance);
        }

        #[test]
        fn aml_check() {
            let event = ComplianceCheckPassed::aml(test_rfq_id(), test_counterparty_id());
            assert_eq!(event.check_type, ComplianceCheckType::Aml);
        }

        #[test]
        fn sanctions_check() {
            let event = ComplianceCheckPassed::sanctions(test_rfq_id(), test_counterparty_id());
            assert_eq!(event.check_type, ComplianceCheckType::Sanctions);
        }

        #[test]
        fn trading_limits_check() {
            let event =
                ComplianceCheckPassed::trading_limits(test_rfq_id(), test_counterparty_id());
            assert_eq!(event.check_type, ComplianceCheckType::TradingLimits);
        }

        #[test]
        fn serde_roundtrip() {
            let event = ComplianceCheckPassed::new(
                test_rfq_id(),
                test_counterparty_id(),
                ComplianceCheckType::General,
                Some("All checks passed".to_string()),
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: ComplianceCheckPassed = serde_json::from_str(&json).unwrap();
            assert_eq!(event.check_type, deserialized.check_type);
            assert_eq!(event.details, deserialized.details);
        }
    }

    mod compliance_check_failed {
        use super::*;

        #[test]
        fn creates_event() {
            let rfq_id = test_rfq_id();
            let event =
                ComplianceCheckFailed::kyc(rfq_id, test_counterparty_id(), "KYC not approved");

            assert_eq!(event.rfq_id(), Some(rfq_id));
            assert_eq!(event.check_type, ComplianceCheckType::Kyc);
            assert_eq!(event.reason, "KYC not approved");
            assert_eq!(event.event_name(), "ComplianceCheckFailed");
        }

        #[test]
        fn aml_failed() {
            let event = ComplianceCheckFailed::aml(
                test_rfq_id(),
                test_counterparty_id(),
                "Suspicious activity",
            );
            assert_eq!(event.check_type, ComplianceCheckType::Aml);
            assert_eq!(event.reason, "Suspicious activity");
        }

        #[test]
        fn sanctions_failed() {
            let event = ComplianceCheckFailed::sanctions(
                test_rfq_id(),
                test_counterparty_id(),
                "On sanctions list",
            );
            assert_eq!(event.check_type, ComplianceCheckType::Sanctions);
        }

        #[test]
        fn trading_limits_failed() {
            let event = ComplianceCheckFailed::trading_limits(
                test_rfq_id(),
                test_counterparty_id(),
                "Daily limit exceeded",
            );
            assert_eq!(event.check_type, ComplianceCheckType::TradingLimits);
        }

        #[test]
        fn with_error_code() {
            let event = ComplianceCheckFailed::new(
                test_rfq_id(),
                test_counterparty_id(),
                ComplianceCheckType::General,
                "Check failed",
                Some("ERR_001".to_string()),
            );

            assert_eq!(event.error_code, Some("ERR_001".to_string()));
        }

        #[test]
        fn serde_roundtrip() {
            let event = ComplianceCheckFailed::new(
                test_rfq_id(),
                test_counterparty_id(),
                ComplianceCheckType::Aml,
                "Failed",
                Some("AML_001".to_string()),
            );

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: ComplianceCheckFailed = serde_json::from_str(&json).unwrap();
            assert_eq!(event.reason, deserialized.reason);
            assert_eq!(event.error_code, deserialized.error_code);
        }
    }

    mod compliance_event_enum {
        use super::*;

        #[test]
        fn serde_roundtrip() {
            let event = ComplianceEvent::Passed(ComplianceCheckPassed::kyc(
                test_rfq_id(),
                test_counterparty_id(),
            ));

            let json = serde_json::to_string(&event).unwrap();
            let deserialized: ComplianceEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event.event_name(), deserialized.event_name());
        }

        #[test]
        fn domain_event_trait() {
            let event = ComplianceEvent::Failed(ComplianceCheckFailed::kyc(
                test_rfq_id(),
                test_counterparty_id(),
                "Failed",
            ));

            assert_eq!(event.event_name(), "ComplianceCheckFailed");
            assert_eq!(event.event_type(), EventType::Compliance);
        }
    }
}
