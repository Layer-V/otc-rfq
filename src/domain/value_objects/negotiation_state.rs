//! # Negotiation State
//!
//! Negotiation lifecycle state machine.
//!
//! This module provides the [`NegotiationState`] enum representing the lifecycle
//! of a counter-quote negotiation between a requester and a market maker.
//!
//! # State Machine
//!
//! ```text
//! Open → CounterPending → Open (loop up to max_rounds)
//!   ↓         ↓
//!   ├─────────┴→ Accepted
//!   ├─────────┴→ Rejected
//!   └─────────┴→ Expired
//! ```
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::value_objects::negotiation_state::NegotiationState;
//!
//! let state = NegotiationState::Open;
//! assert!(state.can_transition_to(NegotiationState::CounterPending));
//! assert!(!state.can_transition_to(NegotiationState::Open));
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Negotiation lifecycle state.
///
/// Represents the current state of a counter-quote negotiation session.
/// State transitions are enforced via [`can_transition_to`](NegotiationState::can_transition_to).
///
/// # Terminal States
///
/// - [`Accepted`](NegotiationState::Accepted) — both parties agreed on terms
/// - [`Rejected`](NegotiationState::Rejected) — one party rejected the negotiation
/// - [`Expired`](NegotiationState::Expired) — negotiation timed out
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::negotiation_state::NegotiationState;
///
/// let state = NegotiationState::Open;
/// assert!(!state.is_terminal());
///
/// let terminal = NegotiationState::Accepted;
/// assert!(terminal.is_terminal());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum NegotiationState {
    /// Negotiation is open, awaiting a counter-quote from either party.
    #[default]
    Open = 0,

    /// A counter-quote has been submitted, awaiting response.
    CounterPending = 1,

    /// Both parties agreed on terms (terminal).
    Accepted = 2,

    /// One party rejected the negotiation (terminal).
    Rejected = 3,

    /// The negotiation expired without resolution (terminal).
    Expired = 4,
}

impl NegotiationState {
    /// Returns true if this is a terminal state.
    ///
    /// Terminal states cannot transition to any other state.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::negotiation_state::NegotiationState;
    ///
    /// assert!(!NegotiationState::Open.is_terminal());
    /// assert!(NegotiationState::Accepted.is_terminal());
    /// assert!(NegotiationState::Rejected.is_terminal());
    /// assert!(NegotiationState::Expired.is_terminal());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Accepted | Self::Rejected | Self::Expired)
    }

    /// Returns true if this state can transition to the target state.
    ///
    /// Enforces the negotiation state machine rules:
    /// - Open → CounterPending, Accepted, Rejected, Expired
    /// - CounterPending → Open, Accepted, Rejected, Expired
    /// - Terminal states → (none)
    ///
    /// # Arguments
    ///
    /// * `target` - The target state to transition to
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::negotiation_state::NegotiationState;
    ///
    /// assert!(NegotiationState::Open.can_transition_to(NegotiationState::CounterPending));
    /// assert!(!NegotiationState::Accepted.can_transition_to(NegotiationState::Open));
    /// ```
    #[must_use]
    pub const fn can_transition_to(&self, target: Self) -> bool {
        matches!(
            (self, target),
            // From Open
            (Self::Open, Self::CounterPending)
                | (Self::Open, Self::Accepted)
                | (Self::Open, Self::Rejected)
                | (Self::Open, Self::Expired)
                // From CounterPending
                | (Self::CounterPending, Self::Open)
                | (Self::CounterPending, Self::Accepted)
                | (Self::CounterPending, Self::Rejected)
                | (Self::CounterPending, Self::Expired)
        )
    }

    /// Returns the valid next states from this state.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::negotiation_state::NegotiationState;
    ///
    /// let transitions = NegotiationState::Open.valid_transitions();
    /// assert!(transitions.contains(&NegotiationState::CounterPending));
    /// ```
    #[must_use]
    pub fn valid_transitions(&self) -> Vec<Self> {
        match self {
            Self::Open => vec![
                Self::CounterPending,
                Self::Accepted,
                Self::Rejected,
                Self::Expired,
            ],
            Self::CounterPending => {
                vec![Self::Open, Self::Accepted, Self::Rejected, Self::Expired]
            }
            Self::Accepted | Self::Rejected | Self::Expired => vec![],
        }
    }

    /// Returns true if this is an active (non-terminal) state.
    #[inline]
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !self.is_terminal()
    }

    /// Returns true if a counter-quote is pending response.
    #[inline]
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self, Self::CounterPending)
    }

    /// Returns the numeric value of this state.
    #[inline]
    #[must_use]
    pub const fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for NegotiationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Open => "OPEN",
            Self::CounterPending => "COUNTER_PENDING",
            Self::Accepted => "ACCEPTED",
            Self::Rejected => "REJECTED",
            Self::Expired => "EXPIRED",
        };
        write!(f, "{s}")
    }
}

/// Error returned when converting an invalid u8 to NegotiationState.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidNegotiationStateError(
    /// The invalid u8 value.
    pub u8,
);

impl fmt::Display for InvalidNegotiationStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid negotiation state value: {}", self.0)
    }
}

impl std::error::Error for InvalidNegotiationStateError {}

impl TryFrom<u8> for NegotiationState {
    type Error = InvalidNegotiationStateError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Open),
            1 => Ok(Self::CounterPending),
            2 => Ok(Self::Accepted),
            3 => Ok(Self::Rejected),
            4 => Ok(Self::Expired),
            _ => Err(InvalidNegotiationStateError(value)),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    mod terminal {
        use super::*;

        #[test]
        fn open_is_not_terminal() {
            assert!(!NegotiationState::Open.is_terminal());
        }

        #[test]
        fn counter_pending_is_not_terminal() {
            assert!(!NegotiationState::CounterPending.is_terminal());
        }

        #[test]
        fn accepted_is_terminal() {
            assert!(NegotiationState::Accepted.is_terminal());
        }

        #[test]
        fn rejected_is_terminal() {
            assert!(NegotiationState::Rejected.is_terminal());
        }

        #[test]
        fn expired_is_terminal() {
            assert!(NegotiationState::Expired.is_terminal());
        }
    }

    mod transitions {
        use super::*;

        #[test]
        fn open_to_counter_pending() {
            assert!(NegotiationState::Open.can_transition_to(NegotiationState::CounterPending));
        }

        #[test]
        fn open_to_accepted() {
            assert!(NegotiationState::Open.can_transition_to(NegotiationState::Accepted));
        }

        #[test]
        fn open_to_rejected() {
            assert!(NegotiationState::Open.can_transition_to(NegotiationState::Rejected));
        }

        #[test]
        fn open_to_expired() {
            assert!(NegotiationState::Open.can_transition_to(NegotiationState::Expired));
        }

        #[test]
        fn counter_pending_to_open() {
            assert!(NegotiationState::CounterPending.can_transition_to(NegotiationState::Open));
        }

        #[test]
        fn counter_pending_to_accepted() {
            assert!(NegotiationState::CounterPending.can_transition_to(NegotiationState::Accepted));
        }

        #[test]
        fn counter_pending_to_rejected() {
            assert!(NegotiationState::CounterPending.can_transition_to(NegotiationState::Rejected));
        }

        #[test]
        fn counter_pending_to_expired() {
            assert!(NegotiationState::CounterPending.can_transition_to(NegotiationState::Expired));
        }

        #[test]
        fn terminal_states_have_no_transitions() {
            for state in [
                NegotiationState::Accepted,
                NegotiationState::Rejected,
                NegotiationState::Expired,
            ] {
                assert!(state.valid_transitions().is_empty());
                for target in [
                    NegotiationState::Open,
                    NegotiationState::CounterPending,
                    NegotiationState::Accepted,
                    NegotiationState::Rejected,
                    NegotiationState::Expired,
                ] {
                    assert!(!state.can_transition_to(target));
                }
            }
        }

        #[test]
        fn open_cannot_self_transition() {
            assert!(!NegotiationState::Open.can_transition_to(NegotiationState::Open));
        }
    }

    mod display {
        use super::*;

        #[test]
        fn display_formats() {
            assert_eq!(NegotiationState::Open.to_string(), "OPEN");
            assert_eq!(
                NegotiationState::CounterPending.to_string(),
                "COUNTER_PENDING"
            );
            assert_eq!(NegotiationState::Accepted.to_string(), "ACCEPTED");
            assert_eq!(NegotiationState::Rejected.to_string(), "REJECTED");
            assert_eq!(NegotiationState::Expired.to_string(), "EXPIRED");
        }
    }

    mod try_from {
        use super::*;

        #[test]
        fn valid_values() {
            assert_eq!(
                NegotiationState::try_from(0u8).unwrap(),
                NegotiationState::Open
            );
            assert_eq!(
                NegotiationState::try_from(1u8).unwrap(),
                NegotiationState::CounterPending
            );
            assert_eq!(
                NegotiationState::try_from(2u8).unwrap(),
                NegotiationState::Accepted
            );
            assert_eq!(
                NegotiationState::try_from(3u8).unwrap(),
                NegotiationState::Rejected
            );
            assert_eq!(
                NegotiationState::try_from(4u8).unwrap(),
                NegotiationState::Expired
            );
        }

        #[test]
        fn invalid_value() {
            let result = NegotiationState::try_from(5u8);
            assert!(matches!(result, Err(InvalidNegotiationStateError(5))));
        }
    }

    mod serde {
        use super::*;

        #[test]
        fn serde_roundtrip() {
            for state in [
                NegotiationState::Open,
                NegotiationState::CounterPending,
                NegotiationState::Accepted,
                NegotiationState::Rejected,
                NegotiationState::Expired,
            ] {
                let json = serde_json::to_string(&state).unwrap();
                let deserialized: NegotiationState = serde_json::from_str(&json).unwrap();
                assert_eq!(state, deserialized);
            }
        }
    }

    mod helpers {
        use super::*;

        #[test]
        fn is_active() {
            assert!(NegotiationState::Open.is_active());
            assert!(NegotiationState::CounterPending.is_active());
            assert!(!NegotiationState::Accepted.is_active());
        }

        #[test]
        fn is_pending() {
            assert!(!NegotiationState::Open.is_pending());
            assert!(NegotiationState::CounterPending.is_pending());
            assert!(!NegotiationState::Accepted.is_pending());
        }

        #[test]
        fn as_u8() {
            assert_eq!(NegotiationState::Open.as_u8(), 0);
            assert_eq!(NegotiationState::CounterPending.as_u8(), 1);
            assert_eq!(NegotiationState::Accepted.as_u8(), 2);
            assert_eq!(NegotiationState::Rejected.as_u8(), 3);
            assert_eq!(NegotiationState::Expired.as_u8(), 4);
        }

        #[test]
        fn default_is_open() {
            assert_eq!(NegotiationState::default(), NegotiationState::Open);
        }
    }
}
