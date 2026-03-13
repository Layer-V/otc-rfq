//! # Domain Error Types
//!
//! Defines the core error types for domain operations.

use std::fmt;

/// Result type alias for domain operations.
pub type DomainResult<T> = Result<T, DomainError>;

/// Domain-level error types.
///
/// These errors represent failures in business logic and domain rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    // Validation errors (1000-1999)
    /// Invalid quantity value.
    InvalidQuantity(String),
    /// Invalid price value.
    InvalidPrice(String),
    /// General validation error.
    ValidationError(String),
    /// Quote has expired.
    QuoteExpired(String),
    /// Quote not found.
    QuoteNotFound(String),
    /// Insufficient liquidity for fill.
    InsufficientLiquidity {
        /// Requested quantity.
        requested: crate::domain::value_objects::Quantity,
        /// Available quantity.
        available: crate::domain::value_objects::Quantity,
    },
    /// Minimum quantity not met.
    MinQuantityNotMet {
        /// Actual fill quantity.
        filled: crate::domain::value_objects::Quantity,
        /// Minimum quantity required.
        minimum: crate::domain::value_objects::Quantity,
    },
    /// Allocation mismatch.
    AllocationMismatch {
        /// Allocated quantity.
        allocated: crate::domain::value_objects::Quantity,
        /// Target quantity.
        target: crate::domain::value_objects::Quantity,
    },
    /// No reference price available.
    NoReferencePrice,
    /// Division by zero.
    DivisionByZero,
    /// Price out of bounds.
    PriceOutOfBounds {
        /// Proposed price.
        proposed: crate::domain::value_objects::Price,
        /// Reference price.
        reference: crate::domain::value_objects::Price,
        /// Actual deviation percentage.
        deviation_pct: rust_decimal::Decimal,
        /// Maximum tolerance percentage.
        max_tolerance_pct: rust_decimal::Decimal,
    },

    // State errors (2000-2999)
    /// Invalid state transition for RFQ.
    InvalidStateTransition {
        /// Source state.
        from: crate::domain::value_objects::RfqState,
        /// Target state.
        to: crate::domain::value_objects::RfqState,
    },
    /// Generic state transition error (for non-RFQ entities).
    GenericStateTransitionError {
        /// Source state name.
        from: String,
        /// Target state name.
        to: String,
    },
    /// Invalid state for operation.
    InvalidState(String),
    /// Operation not allowed in current state.
    OperationNotAllowed(String),
    /// Trade not in correct state for off-book execution.
    InvalidTradeStateForExecution {
        /// Expected state.
        expected: String,
        /// Actual state.
        actual: String,
    },

    // Lock and concurrency errors
    /// Quote is already locked.
    QuoteLocked(String),
    /// Failed to acquire lock.
    LockAcquisitionFailed(String),
    /// Conflict detected during concurrent operation.
    ConflictDetected(String),

    // Risk and compliance errors (3000-3999)
    /// Risk check failed.
    RiskCheckFailed(String),
    /// Unauthorized counterparty.
    UnauthorizedCounterparty(String),
    /// Validation failed.
    ValidationFailed(String),
    /// Invalid negotiation state transition.
    InvalidNegotiationStateTransition {
        /// Source state.
        from: crate::domain::value_objects::NegotiationState,
        /// Target state.
        to: crate::domain::value_objects::NegotiationState,
    },
    /// Maximum negotiation rounds reached.
    MaxNegotiationRoundsReached {
        /// Maximum rounds allowed.
        max_rounds: u8,
    },
    /// No price improvement in counter-quote.
    NoPriceImprovement {
        /// Previous price.
        previous: crate::domain::value_objects::Price,
        /// Proposed price.
        proposed: crate::domain::value_objects::Price,
    },

    // Execution errors
    /// Last-look was rejected by market maker.
    LastLookRejected(String),
    /// Last-look timed out.
    LastLookTimeout(String),
    /// Acceptance flow timed out.
    AcceptanceTimeout(String),

    // Off-book execution errors
    /// Collateral lock failed.
    CollateralLockFailed(String),
    /// Settlement failed.
    SettlementFailed(String),
    /// Position update failed.
    PositionUpdateFailed(String),
    /// Price bounds verification failed (CRE check).
    PriceBoundsVerificationFailed(String),

    // Package quote errors
    /// Invalid package quote.
    InvalidPackageQuote(String),
    /// Inconsistent leg prices in package quote.
    InconsistentLegPrices {
        /// Index of the problematic leg.
        leg_index: usize,
        /// Reason for inconsistency.
        reason: String,
    },
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidQuantity(msg) => write!(f, "invalid quantity: {}", msg),
            Self::InvalidPrice(msg) => write!(f, "invalid price: {}", msg),
            Self::ValidationError(msg) => write!(f, "validation error: {}", msg),
            Self::QuoteExpired(msg) => write!(f, "quote expired: {}", msg),
            Self::QuoteNotFound(msg) => write!(f, "quote not found: {}", msg),
            Self::InsufficientLiquidity {
                requested,
                available,
            } => {
                write!(
                    f,
                    "insufficient liquidity: requested {}, available {}",
                    requested, available
                )
            }
            Self::MinQuantityNotMet { filled, minimum } => {
                write!(
                    f,
                    "minimum quantity not met: filled {}, minimum {}",
                    filled, minimum
                )
            }
            Self::AllocationMismatch { allocated, target } => {
                write!(
                    f,
                    "allocation mismatch: allocated {}, target {}",
                    allocated, target
                )
            }
            Self::NoReferencePrice => write!(f, "no reference price available"),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::PriceOutOfBounds {
                proposed,
                reference,
                deviation_pct,
                max_tolerance_pct,
            } => {
                write!(
                    f,
                    "price out of bounds: proposed {}, reference {}, deviation {}%, max tolerance {}%",
                    proposed, reference, deviation_pct, max_tolerance_pct
                )
            }
            Self::InvalidStateTransition { from, to } => {
                write!(f, "invalid state transition from {} to {}", from, to)
            }
            Self::GenericStateTransitionError { from, to } => {
                write!(f, "invalid state transition from {} to {}", from, to)
            }
            Self::InvalidState(msg) => write!(f, "invalid state: {}", msg),
            Self::OperationNotAllowed(msg) => write!(f, "operation not allowed: {}", msg),
            Self::InvalidTradeStateForExecution { expected, actual } => {
                write!(
                    f,
                    "invalid trade state for execution: expected {}, got {}",
                    expected, actual
                )
            }
            Self::QuoteLocked(msg) => write!(f, "quote locked: {}", msg),
            Self::LockAcquisitionFailed(msg) => write!(f, "lock acquisition failed: {}", msg),
            Self::ConflictDetected(msg) => write!(f, "conflict detected: {}", msg),
            Self::RiskCheckFailed(msg) => write!(f, "risk check failed: {}", msg),
            Self::UnauthorizedCounterparty(msg) => write!(f, "unauthorized counterparty: {}", msg),
            Self::ValidationFailed(msg) => write!(f, "validation failed: {}", msg),
            Self::InvalidNegotiationStateTransition { from, to } => {
                write!(
                    f,
                    "invalid negotiation state transition from {} to {}",
                    from, to
                )
            }
            Self::MaxNegotiationRoundsReached { max_rounds } => {
                write!(f, "maximum negotiation rounds ({}) reached", max_rounds)
            }
            Self::NoPriceImprovement { previous, proposed } => {
                write!(
                    f,
                    "no price improvement: previous {}, proposed {}",
                    previous, proposed
                )
            }
            Self::LastLookRejected(msg) => write!(f, "last-look rejected: {}", msg),
            Self::LastLookTimeout(msg) => write!(f, "last-look timeout: {}", msg),
            Self::AcceptanceTimeout(msg) => write!(f, "acceptance timeout: {}", msg),
            Self::CollateralLockFailed(msg) => write!(f, "collateral lock failed: {}", msg),
            Self::SettlementFailed(msg) => write!(f, "settlement failed: {}", msg),
            Self::PositionUpdateFailed(msg) => write!(f, "position update failed: {}", msg),
            Self::PriceBoundsVerificationFailed(msg) => {
                write!(f, "price bounds verification failed: {}", msg)
            }
            Self::InvalidPackageQuote(msg) => write!(f, "invalid package quote: {}", msg),
            Self::InconsistentLegPrices { leg_index, reason } => {
                write!(
                    f,
                    "inconsistent leg prices at index {}: {}",
                    leg_index, reason
                )
            }
        }
    }
}

impl std::error::Error for DomainError {}

impl From<crate::domain::value_objects::ArithmeticError> for DomainError {
    fn from(err: crate::domain::value_objects::ArithmeticError) -> Self {
        Self::ValidationError(err.to_string())
    }
}
