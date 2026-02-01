//! # Domain Events
//!
//! Events emitted during domain operations for event sourcing and audit trail.
//!
//! ## RFQ Events
//!
//! - [`RfqCreated`]: New RFQ initiated
//! - [`QuoteCollectionStarted`]: Quote collection begins
//! - [`QuoteRequested`]: Quote requested from venue
//! - [`QuoteReceived`]: Quote received from venue
//! - [`QuoteRequestFailed`]: Quote request failed
//! - [`QuoteCollectionCompleted`]: Quote collection finished
//! - [`QuoteSelected`]: Client selected a quote
//! - [`ExecutionStarted`]: Trade execution begins
//! - [`ExecutionFailed`]: Trade execution failed
//! - [`RfqCancelled`]: RFQ was cancelled
//! - [`RfqExpired`]: RFQ expired
//!
//! ## Trade Events
//!
//! - [`TradeExecuted`]: Trade successfully executed
//! - [`SettlementInitiated`]: Settlement process started
//! - [`SettlementConfirmed`]: Settlement completed successfully
//! - [`SettlementFailed`]: Settlement failed
//!
//! ## Compliance Events
//!
//! - [`ComplianceCheckPassed`]: Compliance check passed
//! - [`ComplianceCheckFailed`]: Compliance check failed

pub mod compliance_events;
pub mod domain_event;
pub mod rfq_events;
pub mod trade_events;

pub use compliance_events::{
    ComplianceCheckFailed, ComplianceCheckPassed, ComplianceCheckType, ComplianceEvent,
};
pub use domain_event::{DomainEvent, EventMetadata, EventType};
pub use rfq_events::{
    ExecutionFailed, ExecutionStarted, QuoteCollectionCompleted, QuoteCollectionStarted,
    QuoteReceived, QuoteRequestFailed, QuoteRequested, QuoteSelected, RfqCancelled, RfqCreated,
    RfqEvent, RfqExpired,
};
pub use trade_events::{
    SettlementConfirmed, SettlementFailed, SettlementInitiated, TradeEvent, TradeExecuted,
};
