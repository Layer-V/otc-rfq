//! # Liquidity Classification
//!
//! Classification of instrument liquidity for price bounds validation.
//!
//! This module provides the [`LiquidityClassification`] enum used to
//! determine which tolerance band applies when validating block trade
//! prices against reference prices.
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::value_objects::liquidity_classification::LiquidityClassification;
//!
//! let liq = LiquidityClassification::Liquid;
//! assert_eq!(liq.to_string(), "Liquid");
//! assert!(liq.is_liquid());
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Classification of an instrument's liquidity profile.
///
/// Determines which tolerance percentage applies when validating
/// proposed block trade prices against reference prices.
///
/// - **Liquid**: Tight tolerance (e.g., ±5%)
/// - **SemiLiquid**: Medium tolerance (e.g., ±7.5%)
/// - **Illiquid**: Wide tolerance (e.g., ±10%)
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::liquidity_classification::LiquidityClassification;
///
/// let default = LiquidityClassification::default();
/// assert_eq!(default, LiquidityClassification::Liquid);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
#[derive(Default)]
pub enum LiquidityClassification {
    /// High liquidity — tight price tolerance band.
    #[default]
    Liquid = 0,
    /// Medium liquidity — moderate price tolerance band.
    SemiLiquid = 1,
    /// Low liquidity — wide price tolerance band.
    Illiquid = 2,
}

impl LiquidityClassification {
    /// Returns `true` if this is a liquid classification.
    #[inline]
    #[must_use]
    pub const fn is_liquid(self) -> bool {
        matches!(self, Self::Liquid)
    }

    /// Returns `true` if this is a semi-liquid classification.
    #[inline]
    #[must_use]
    pub const fn is_semi_liquid(self) -> bool {
        matches!(self, Self::SemiLiquid)
    }

    /// Returns `true` if this is an illiquid classification.
    #[inline]
    #[must_use]
    pub const fn is_illiquid(self) -> bool {
        matches!(self, Self::Illiquid)
    }
}

impl fmt::Display for LiquidityClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Liquid => write!(f, "Liquid"),
            Self::SemiLiquid => write!(f, "SemiLiquid"),
            Self::Illiquid => write!(f, "Illiquid"),
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
        fn all_variants_exist() {
            let _ = LiquidityClassification::Liquid;
            let _ = LiquidityClassification::SemiLiquid;
            let _ = LiquidityClassification::Illiquid;
        }

        #[test]
        fn default_is_liquid() {
            assert_eq!(
                LiquidityClassification::default(),
                LiquidityClassification::Liquid
            );
        }
    }

    mod predicates {
        use super::*;

        #[test]
        fn is_liquid() {
            assert!(LiquidityClassification::Liquid.is_liquid());
            assert!(!LiquidityClassification::SemiLiquid.is_liquid());
            assert!(!LiquidityClassification::Illiquid.is_liquid());
        }

        #[test]
        fn is_semi_liquid() {
            assert!(!LiquidityClassification::Liquid.is_semi_liquid());
            assert!(LiquidityClassification::SemiLiquid.is_semi_liquid());
            assert!(!LiquidityClassification::Illiquid.is_semi_liquid());
        }

        #[test]
        fn is_illiquid() {
            assert!(!LiquidityClassification::Liquid.is_illiquid());
            assert!(!LiquidityClassification::SemiLiquid.is_illiquid());
            assert!(LiquidityClassification::Illiquid.is_illiquid());
        }
    }

    mod display {
        use super::*;

        #[test]
        fn formats_correctly() {
            assert_eq!(LiquidityClassification::Liquid.to_string(), "Liquid");
            assert_eq!(
                LiquidityClassification::SemiLiquid.to_string(),
                "SemiLiquid"
            );
            assert_eq!(LiquidityClassification::Illiquid.to_string(), "Illiquid");
        }
    }

    mod serde_tests {
        use super::*;

        #[test]
        fn roundtrip() {
            for variant in [
                LiquidityClassification::Liquid,
                LiquidityClassification::SemiLiquid,
                LiquidityClassification::Illiquid,
            ] {
                let json = serde_json::to_string(&variant).unwrap();
                let deserialized: LiquidityClassification = serde_json::from_str(&json).unwrap();
                assert_eq!(variant, deserialized);
            }
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn same_variants_equal() {
            assert_eq!(
                LiquidityClassification::Liquid,
                LiquidityClassification::Liquid
            );
        }

        #[test]
        fn different_variants_not_equal() {
            assert_ne!(
                LiquidityClassification::Liquid,
                LiquidityClassification::Illiquid
            );
        }
    }
}
