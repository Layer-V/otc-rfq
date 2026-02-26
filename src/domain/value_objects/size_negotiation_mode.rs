//! # Size Negotiation Mode
//!
//! Defines the fill semantics for RFQ quantity execution.
//!
//! This module provides the [`SizeNegotiationMode`] enum which controls
//! how an RFQ's requested quantity may be filled across one or multiple
//! market makers.
//!
//! # Modes
//!
//! | Mode | Behavior |
//! |------|----------|
//! | `AllOrNothing` | Reject unless the full quantity can be filled |
//! | `MinQuantity` | Reject if total fill < minimum threshold |
//! | `FillOrKill` | Fill the full quantity immediately or cancel |
//! | `BestEffort` | Fill as much as possible, partial fills allowed |
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::value_objects::size_negotiation_mode::SizeNegotiationMode;
//! use otc_rfq::domain::value_objects::Quantity;
//!
//! let mode = SizeNegotiationMode::AllOrNothing;
//! assert!(mode.requires_full_fill());
//!
//! let min = SizeNegotiationMode::MinQuantity(Quantity::new(0.5).unwrap());
//! assert!(!min.requires_full_fill());
//! ```

use crate::domain::value_objects::Quantity;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Controls how an RFQ's requested quantity may be filled.
///
/// Determines whether partial fills are allowed, whether a minimum
/// quantity threshold must be met, or whether the full quantity must
/// be filled atomically.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::size_negotiation_mode::SizeNegotiationMode;
/// use otc_rfq::domain::value_objects::Quantity;
///
/// let mode = SizeNegotiationMode::BestEffort;
/// assert!(mode.allows_partial_fill());
/// assert!(!mode.requires_full_fill());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SizeNegotiationMode {
    /// Reject unless the full requested quantity can be filled.
    AllOrNothing,

    /// Reject if the total filled quantity is below this minimum threshold.
    MinQuantity(Quantity),

    /// Fill the full quantity immediately or cancel the entire request.
    ///
    /// Similar to `AllOrNothing` but with immediate-execution semantics —
    /// the fill must happen in a single atomic pass with no waiting.
    FillOrKill,

    /// Fill as much as possible; partial fills are acceptable.
    BestEffort,
}

impl SizeNegotiationMode {
    /// Returns `true` if this mode requires the full target quantity to be filled.
    #[must_use]
    #[inline]
    pub fn requires_full_fill(&self) -> bool {
        matches!(self, Self::AllOrNothing | Self::FillOrKill)
    }

    /// Returns `true` if this mode allows partial fills.
    #[must_use]
    #[inline]
    pub fn allows_partial_fill(&self) -> bool {
        matches!(self, Self::BestEffort | Self::MinQuantity(_))
    }

    /// Returns the minimum quantity threshold, if any.
    ///
    /// - `AllOrNothing` / `FillOrKill`: returns `None` (full fill required instead)
    /// - `MinQuantity(q)`: returns `Some(q)`
    /// - `BestEffort`: returns `None` (no minimum)
    #[must_use]
    #[inline]
    pub fn min_quantity(&self) -> Option<Quantity> {
        match self {
            Self::MinQuantity(q) => Some(*q),
            _ => None,
        }
    }
}

impl Default for SizeNegotiationMode {
    /// Defaults to `AllOrNothing` — the safest mode.
    fn default() -> Self {
        Self::AllOrNothing
    }
}

impl fmt::Display for SizeNegotiationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllOrNothing => write!(f, "AllOrNothing"),
            Self::MinQuantity(q) => write!(f, "MinQuantity({q})"),
            Self::FillOrKill => write!(f, "FillOrKill"),
            Self::BestEffort => write!(f, "BestEffort"),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    mod construction {
        use super::*;

        #[test]
        fn all_or_nothing() {
            let mode = SizeNegotiationMode::AllOrNothing;
            assert!(mode.requires_full_fill());
            assert!(!mode.allows_partial_fill());
            assert!(mode.min_quantity().is_none());
        }

        #[test]
        fn fill_or_kill() {
            let mode = SizeNegotiationMode::FillOrKill;
            assert!(mode.requires_full_fill());
            assert!(!mode.allows_partial_fill());
            assert!(mode.min_quantity().is_none());
        }

        #[test]
        fn min_quantity() {
            let qty = Quantity::new(0.5).unwrap();
            let mode = SizeNegotiationMode::MinQuantity(qty);
            assert!(!mode.requires_full_fill());
            assert!(mode.allows_partial_fill());
            assert_eq!(mode.min_quantity(), Some(qty));
        }

        #[test]
        fn best_effort() {
            let mode = SizeNegotiationMode::BestEffort;
            assert!(!mode.requires_full_fill());
            assert!(mode.allows_partial_fill());
            assert!(mode.min_quantity().is_none());
        }

        #[test]
        fn default_is_all_or_nothing() {
            assert_eq!(
                SizeNegotiationMode::default(),
                SizeNegotiationMode::AllOrNothing
            );
        }
    }

    mod display {
        use super::*;

        #[test]
        fn display_all_variants() {
            assert_eq!(
                SizeNegotiationMode::AllOrNothing.to_string(),
                "AllOrNothing"
            );
            assert_eq!(SizeNegotiationMode::FillOrKill.to_string(), "FillOrKill");
            assert_eq!(SizeNegotiationMode::BestEffort.to_string(), "BestEffort");

            let qty = Quantity::new(1.5).unwrap();
            let display = SizeNegotiationMode::MinQuantity(qty).to_string();
            assert!(display.starts_with("MinQuantity("));
        }
    }

    mod serde_roundtrip {
        use super::*;

        #[test]
        fn all_or_nothing() {
            let mode = SizeNegotiationMode::AllOrNothing;
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: SizeNegotiationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, deserialized);
        }

        #[test]
        fn min_quantity() {
            let mode = SizeNegotiationMode::MinQuantity(Quantity::new(2.0).unwrap());
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: SizeNegotiationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, deserialized);
        }

        #[test]
        fn fill_or_kill() {
            let mode = SizeNegotiationMode::FillOrKill;
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: SizeNegotiationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, deserialized);
        }

        #[test]
        fn best_effort() {
            let mode = SizeNegotiationMode::BestEffort;
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: SizeNegotiationMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, deserialized);
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn same_variants_are_equal() {
            assert_eq!(
                SizeNegotiationMode::AllOrNothing,
                SizeNegotiationMode::AllOrNothing
            );
            assert_eq!(
                SizeNegotiationMode::FillOrKill,
                SizeNegotiationMode::FillOrKill
            );
            assert_eq!(
                SizeNegotiationMode::BestEffort,
                SizeNegotiationMode::BestEffort
            );
        }

        #[test]
        fn different_variants_are_not_equal() {
            assert_ne!(
                SizeNegotiationMode::AllOrNothing,
                SizeNegotiationMode::BestEffort
            );
            assert_ne!(
                SizeNegotiationMode::FillOrKill,
                SizeNegotiationMode::BestEffort
            );
        }

        #[test]
        fn min_quantity_equality_by_value() {
            let a = SizeNegotiationMode::MinQuantity(Quantity::new(1.0).unwrap());
            let b = SizeNegotiationMode::MinQuantity(Quantity::new(1.0).unwrap());
            let c = SizeNegotiationMode::MinQuantity(Quantity::new(2.0).unwrap());
            assert_eq!(a, b);
            assert_ne!(a, c);
        }
    }
}
