//! # Application Errors
//!
//! Error types for the application layer.
//!
//! These errors represent failures that can occur during use case execution,
//! including validation failures, business rule violations, and infrastructure errors.
//!
//! # Error Hierarchy
//!
//! ```text
//! ApplicationError
//! ├── Domain(DomainError)         - Business rule violations
//! ├── Infrastructure(InfrastructureError) - External system failures
//! ├── Venue(VenueError)           - Venue-specific errors
//! ├── Validation(String)          - Input validation failures
//! ├── NotFound(String)            - Resource not found
//! ├── Unauthorized                - Authentication/authorization failures
//! └── ... (specific error variants)
//! ```
//!
//! # Examples
//!
//! ```
//! use otc_rfq::application::error::{ApplicationError, InfrastructureError};
//!
//! // Create validation error
//! let err = ApplicationError::validation("quantity must be positive");
//!
//! // Create not found error
//! let err = ApplicationError::not_found("RFQ", "rfq-123");
//!
//! // Create infrastructure error
//! let infra_err = InfrastructureError::database("connection timeout");
//! let app_err: ApplicationError = infra_err.into();
//! ```

use crate::domain::errors::DomainError;
use crate::infrastructure::persistence::RepositoryError;
use crate::infrastructure::venues::error::VenueError;
use std::fmt;
use thiserror::Error;

/// Infrastructure layer error.
///
/// Represents errors from external systems and infrastructure components
/// such as databases, message queues, and external services.
#[derive(Debug, Error)]
pub enum InfrastructureError {
    /// Database error.
    #[error("database error: {0}")]
    Database(String),

    /// Network error.
    #[error("network error: {0}")]
    Network(String),

    /// Message queue error.
    #[error("message queue error: {0}")]
    MessageQueue(String),

    /// Cache error.
    #[error("cache error: {0}")]
    Cache(String),

    /// External service error.
    #[error("external service error: {service} - {message}")]
    ExternalService {
        /// Service name.
        service: String,
        /// Error message.
        message: String,
    },

    /// Configuration error.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Timeout error.
    #[error("timeout: {0}")]
    Timeout(String),

    /// Repository error.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
}

impl InfrastructureError {
    /// Creates a database error.
    #[must_use]
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database(message.into())
    }

    /// Creates a network error.
    #[must_use]
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    /// Creates a message queue error.
    #[must_use]
    pub fn message_queue(message: impl Into<String>) -> Self {
        Self::MessageQueue(message.into())
    }

    /// Creates a cache error.
    #[must_use]
    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache(message.into())
    }

    /// Creates an external service error.
    #[must_use]
    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Creates a configuration error.
    #[must_use]
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration(message.into())
    }

    /// Creates a serialization error.
    #[must_use]
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization(message.into())
    }

    /// Creates a timeout error.
    #[must_use]
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout(message.into())
    }

    /// Returns true if this error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::Timeout(_) | Self::MessageQueue(_)
        )
    }
}

/// Application layer error.
///
/// Wraps domain, infrastructure, and venue errors with application-specific
/// context for use case execution failures.
#[derive(Debug, Error)]
pub enum ApplicationError {
    /// Domain error from business logic.
    #[error("domain error: {0}")]
    Domain(#[from] DomainError),

    /// Infrastructure error from external systems.
    #[error("infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),

    /// Venue error from liquidity providers.
    #[error("venue error: {0}")]
    Venue(#[from] VenueError),

    /// Request validation failed.
    #[error("validation error: {0}")]
    Validation(String),

    /// Resource not found.
    #[error("not found: {resource_type} with id {id}")]
    NotFound {
        /// Type of resource.
        resource_type: String,
        /// Resource identifier.
        id: String,
    },

    /// Authentication or authorization failure.
    #[error("unauthorized")]
    Unauthorized,

    /// Client not found.
    #[error("client not found: {0}")]
    ClientNotFound(String),

    /// Client is not active.
    #[error("client not active: {0}")]
    ClientNotActive(String),

    /// Instrument not supported.
    #[error("instrument not supported: {0}")]
    InstrumentNotSupported(String),

    /// Compliance check failed.
    #[error("compliance check failed: {0}")]
    ComplianceFailed(String),

    /// Repository error.
    #[error("repository error: {0}")]
    RepositoryError(String),

    /// Event publishing error.
    #[error("event publishing error: {0}")]
    EventPublishError(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// RFQ not found.
    #[error("rfq not found: {0}")]
    RfqNotFound(String),

    /// Quote not found.
    #[error("quote not found: {0}")]
    QuoteNotFound(String),

    /// Quote expired.
    #[error("quote expired: {0}")]
    QuoteExpired(String),

    /// Invalid state for operation.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Venue not available.
    #[error("venue not available: {0}")]
    VenueNotAvailable(String),

    /// Trade execution failed.
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
}

impl ApplicationError {
    /// Creates a client not found error.
    #[must_use]
    pub fn client_not_found(client_id: impl Into<String>) -> Self {
        Self::ClientNotFound(client_id.into())
    }

    /// Creates a client not active error.
    #[must_use]
    pub fn client_not_active(client_id: impl Into<String>) -> Self {
        Self::ClientNotActive(client_id.into())
    }

    /// Creates an instrument not supported error.
    #[must_use]
    pub fn instrument_not_supported(instrument: impl fmt::Display) -> Self {
        Self::InstrumentNotSupported(instrument.to_string())
    }

    /// Creates a compliance failed error.
    #[must_use]
    pub fn compliance_failed(reason: impl Into<String>) -> Self {
        Self::ComplianceFailed(reason.into())
    }

    /// Creates a validation error.
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Creates a not found error.
    #[must_use]
    pub fn not_found(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// Creates an unauthorized error.
    #[must_use]
    pub fn unauthorized() -> Self {
        Self::Unauthorized
    }

    /// Creates a repository error.
    #[must_use]
    pub fn repository(message: impl Into<String>) -> Self {
        Self::RepositoryError(message.into())
    }

    /// Creates an event publish error.
    #[must_use]
    pub fn event_publish(message: impl Into<String>) -> Self {
        Self::EventPublishError(message.into())
    }

    /// Creates an internal error.
    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Returns true if this error is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Infrastructure(e) => e.is_retryable(),
            Self::Venue(e) => e.is_retryable(),
            _ => false,
        }
    }

    /// Returns true if this is a not found error.
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self::NotFound { .. }
                | Self::ClientNotFound(_)
                | Self::RfqNotFound(_)
                | Self::QuoteNotFound(_)
        )
    }

    /// Returns true if this is a validation error.
    #[must_use]
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Validation(_))
    }

    /// Returns true if this is an authorization error.
    #[must_use]
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Self::Unauthorized)
    }
}

/// Result type for application operations.
pub type ApplicationResult<T> = Result<T, ApplicationError>;

#[cfg(test)]
mod tests {
    use super::*;

    // InfrastructureError tests

    #[test]
    fn infrastructure_error_database() {
        let err = InfrastructureError::database("connection timeout");
        assert!(err.to_string().contains("database"));
        assert!(err.to_string().contains("connection timeout"));
    }

    #[test]
    fn infrastructure_error_network() {
        let err = InfrastructureError::network("connection refused");
        assert!(err.to_string().contains("network"));
        assert!(err.is_retryable());
    }

    #[test]
    fn infrastructure_error_message_queue() {
        let err = InfrastructureError::message_queue("broker unavailable");
        assert!(err.to_string().contains("message queue"));
        assert!(err.is_retryable());
    }

    #[test]
    fn infrastructure_error_cache() {
        let err = InfrastructureError::cache("redis timeout");
        assert!(err.to_string().contains("cache"));
        assert!(!err.is_retryable());
    }

    #[test]
    fn infrastructure_error_external_service() {
        let err = InfrastructureError::external_service("pricing-api", "rate limited");
        assert!(err.to_string().contains("pricing-api"));
        assert!(err.to_string().contains("rate limited"));
    }

    #[test]
    fn infrastructure_error_configuration() {
        let err = InfrastructureError::configuration("missing API key");
        assert!(err.to_string().contains("configuration"));
    }

    #[test]
    fn infrastructure_error_serialization() {
        let err = InfrastructureError::serialization("invalid JSON");
        assert!(err.to_string().contains("serialization"));
    }

    #[test]
    fn infrastructure_error_timeout() {
        let err = InfrastructureError::timeout("request exceeded 5s");
        assert!(err.to_string().contains("timeout"));
        assert!(err.is_retryable());
    }

    #[test]
    fn infrastructure_error_from_repository_error() {
        let repo_err = RepositoryError::not_found("RFQ", "rfq-123");
        let infra_err: InfrastructureError = repo_err.into();
        assert!(infra_err.to_string().contains("rfq-123"));
    }

    // ApplicationError tests

    #[test]
    fn application_error_client_not_found() {
        let err = ApplicationError::client_not_found("client-123");
        assert!(err.to_string().contains("client-123"));
        assert!(err.is_not_found());
    }

    #[test]
    fn application_error_client_not_active() {
        let err = ApplicationError::client_not_active("client-456");
        assert!(err.to_string().contains("client-456"));
    }

    #[test]
    fn application_error_instrument_not_supported() {
        let err = ApplicationError::instrument_not_supported("BTC/XYZ");
        assert!(err.to_string().contains("BTC/XYZ"));
    }

    #[test]
    fn application_error_compliance_failed() {
        let err = ApplicationError::compliance_failed("KYC not verified");
        assert!(err.to_string().contains("KYC not verified"));
    }

    #[test]
    fn application_error_validation() {
        let err = ApplicationError::validation("quantity must be positive");
        assert!(err.to_string().contains("quantity must be positive"));
        assert!(err.is_validation());
    }

    #[test]
    fn application_error_not_found() {
        let err = ApplicationError::not_found("RFQ", "rfq-123");
        assert!(err.to_string().contains("RFQ"));
        assert!(err.to_string().contains("rfq-123"));
        assert!(err.is_not_found());
    }

    #[test]
    fn application_error_unauthorized() {
        let err = ApplicationError::unauthorized();
        assert!(err.to_string().contains("unauthorized"));
        assert!(err.is_unauthorized());
    }

    #[test]
    fn application_error_repository() {
        let err = ApplicationError::repository("database connection failed");
        assert!(err.to_string().contains("database connection failed"));
    }

    #[test]
    fn application_error_from_domain_error() {
        let domain_err = DomainError::InvalidQuantity("negative".to_string());
        let app_err: ApplicationError = domain_err.into();
        assert!(app_err.to_string().contains("negative"));
    }

    #[test]
    fn application_error_from_infrastructure_error() {
        let infra_err = InfrastructureError::database("connection failed");
        let app_err: ApplicationError = infra_err.into();
        assert!(app_err.to_string().contains("infrastructure"));
        assert!(app_err.to_string().contains("database"));
    }

    #[test]
    fn application_error_from_venue_error() {
        let venue_err = VenueError::timeout("request timed out");
        let app_err: ApplicationError = venue_err.into();
        assert!(app_err.to_string().contains("venue"));
        assert!(app_err.is_retryable());
    }

    #[test]
    fn application_error_retryable_from_infrastructure() {
        let infra_err = InfrastructureError::network("connection refused");
        let app_err: ApplicationError = infra_err.into();
        assert!(app_err.is_retryable());
    }

    #[test]
    fn application_error_retryable_from_venue() {
        let venue_err = VenueError::connection("network error");
        let app_err: ApplicationError = venue_err.into();
        assert!(app_err.is_retryable());
    }

    #[test]
    fn application_error_not_retryable() {
        let err = ApplicationError::validation("invalid input");
        assert!(!err.is_retryable());

        let err = ApplicationError::unauthorized();
        assert!(!err.is_retryable());
    }

    #[test]
    fn application_error_is_not_found_variants() {
        assert!(ApplicationError::not_found("RFQ", "123").is_not_found());
        assert!(ApplicationError::ClientNotFound("123".to_string()).is_not_found());
        assert!(ApplicationError::RfqNotFound("123".to_string()).is_not_found());
        assert!(ApplicationError::QuoteNotFound("123".to_string()).is_not_found());
        assert!(!ApplicationError::validation("test").is_not_found());
    }
}
