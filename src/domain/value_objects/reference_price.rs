//! # Reference Price Types
//!
//! Value objects for price bounds validation against reference prices.
//!
//! This module provides:
//! - [`ReferencePriceSource`]: Origin of a reference price (CLOB mid, theoretical, Chainlink)
//! - [`PriceBoundsConfig`]: Tolerance percentages per liquidity tier
//! - [`PriceBoundsResult`]: Successful validation outcome with deviation details
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::value_objects::reference_price::{
//!     ReferencePriceSource, PriceBoundsConfig, PriceBoundsResult,
//! };
//! use otc_rfq::domain::value_objects::Price;
//! use rust_decimal::Decimal;
//!
//! let config = PriceBoundsConfig::default();
//! assert_eq!(config.liquid_tolerance_pct(), Decimal::new(5, 2));
//!
//! let result = PriceBoundsResult::new(
//!     Price::new(50000.0).unwrap(),
//!     ReferencePriceSource::ClobMid,
//!     Decimal::new(2, 2),
//! );
//! assert_eq!(result.source(), ReferencePriceSource::ClobMid);
//! ```

use crate::domain::value_objects::liquidity_classification::LiquidityClassification;
use crate::domain::value_objects::price::Price;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The origin of a reference price used for bounds validation.
///
/// Prices are checked in priority order: CLOB mid → Theoretical → Chainlink index.
/// The first available source is used.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::reference_price::ReferencePriceSource;
///
/// let source = ReferencePriceSource::ClobMid;
/// assert_eq!(source.to_string(), "ClobMid");
/// assert_eq!(source.priority(), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ReferencePriceSource {
    /// Central Limit Order Book mid-price — highest priority.
    ClobMid = 0,
    /// Theoretical (model-derived) price — second priority.
    Theoretical = 1,
    /// Chainlink on-chain index price — lowest priority / fallback.
    ChainlinkIndex = 2,
}

impl ReferencePriceSource {
    /// Returns the priority of this source (lower is higher priority).
    #[inline]
    #[must_use]
    pub const fn priority(self) -> u8 {
        self as u8
    }
}

impl fmt::Display for ReferencePriceSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClobMid => write!(f, "ClobMid"),
            Self::Theoretical => write!(f, "Theoretical"),
            Self::ChainlinkIndex => write!(f, "ChainlinkIndex"),
        }
    }
}

/// Configuration for price bounds validation tolerances.
///
/// Each field is a fractional percentage (e.g., `0.05` = ±5%).
/// All tolerances must be non-negative.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::reference_price::PriceBoundsConfig;
/// use rust_decimal::Decimal;
///
/// let config = PriceBoundsConfig::new(
///     Decimal::new(5, 2),
///     Decimal::new(75, 3),
///     Decimal::new(10, 2),
/// ).unwrap();
/// assert_eq!(config.liquid_tolerance_pct(), Decimal::new(5, 2));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PriceBoundsConfig {
    /// Maximum allowed deviation for liquid instruments (fractional, e.g., 0.05 = ±5%).
    liquid_tolerance_pct: Decimal,
    /// Maximum allowed deviation for semi-liquid instruments (fractional, e.g., 0.075 = ±7.5%).
    semi_liquid_tolerance_pct: Decimal,
    /// Maximum allowed deviation for illiquid instruments (fractional, e.g., 0.10 = ±10%).
    illiquid_tolerance_pct: Decimal,
}

impl PriceBoundsConfig {
    /// Creates a new configuration with the given tolerance percentages.
    ///
    /// # Arguments
    ///
    /// * `liquid_tolerance_pct` - Tolerance for liquid instruments (fractional)
    /// * `semi_liquid_tolerance_pct` - Tolerance for semi-liquid instruments (fractional)
    /// * `illiquid_tolerance_pct` - Tolerance for illiquid instruments (fractional)
    ///
    /// # Errors
    ///
    /// Returns `None` if any tolerance is negative.
    /// A zero tolerance is valid and means exact price match is required
    /// (useful for fixed-price or pegged instruments).
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::reference_price::PriceBoundsConfig;
    /// use rust_decimal::Decimal;
    ///
    /// let config = PriceBoundsConfig::new(
    ///     Decimal::new(5, 2),
    ///     Decimal::new(75, 3),
    ///     Decimal::new(10, 2),
    /// ).unwrap();
    /// assert_eq!(config.illiquid_tolerance_pct(), Decimal::new(10, 2));
    ///
    /// // Negative tolerance is rejected
    /// assert!(PriceBoundsConfig::new(
    ///     Decimal::new(-1, 2),
    ///     Decimal::new(5, 2),
    ///     Decimal::new(10, 2),
    /// ).is_none());
    /// ```
    #[must_use]
    pub fn new(
        liquid_tolerance_pct: Decimal,
        semi_liquid_tolerance_pct: Decimal,
        illiquid_tolerance_pct: Decimal,
    ) -> Option<Self> {
        if liquid_tolerance_pct.is_sign_negative()
            || semi_liquid_tolerance_pct.is_sign_negative()
            || illiquid_tolerance_pct.is_sign_negative()
        {
            return None;
        }
        Some(Self {
            liquid_tolerance_pct,
            semi_liquid_tolerance_pct,
            illiquid_tolerance_pct,
        })
    }

    /// Returns the tolerance for liquid instruments (fractional).
    #[inline]
    #[must_use]
    pub const fn liquid_tolerance_pct(&self) -> Decimal {
        self.liquid_tolerance_pct
    }

    /// Returns the tolerance for semi-liquid instruments (fractional).
    #[inline]
    #[must_use]
    pub const fn semi_liquid_tolerance_pct(&self) -> Decimal {
        self.semi_liquid_tolerance_pct
    }

    /// Returns the tolerance for illiquid instruments (fractional).
    #[inline]
    #[must_use]
    pub const fn illiquid_tolerance_pct(&self) -> Decimal {
        self.illiquid_tolerance_pct
    }

    /// Returns the tolerance for the given liquidity classification.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::reference_price::PriceBoundsConfig;
    /// use otc_rfq::domain::value_objects::liquidity_classification::LiquidityClassification;
    /// use rust_decimal::Decimal;
    ///
    /// let config = PriceBoundsConfig::default();
    /// assert_eq!(
    ///     config.tolerance_for(LiquidityClassification::Liquid),
    ///     Decimal::new(5, 2),
    /// );
    /// assert_eq!(
    ///     config.tolerance_for(LiquidityClassification::Illiquid),
    ///     Decimal::new(10, 2),
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub const fn tolerance_for(&self, liquidity: LiquidityClassification) -> Decimal {
        match liquidity {
            LiquidityClassification::Liquid => self.liquid_tolerance_pct,
            LiquidityClassification::SemiLiquid => self.semi_liquid_tolerance_pct,
            LiquidityClassification::Illiquid => self.illiquid_tolerance_pct,
        }
    }
}

impl Default for PriceBoundsConfig {
    /// Returns the default configuration: ±5% liquid, ±7.5% semi-liquid, ±10% illiquid.
    fn default() -> Self {
        Self {
            liquid_tolerance_pct: Decimal::new(5, 2),
            semi_liquid_tolerance_pct: Decimal::new(75, 3),
            illiquid_tolerance_pct: Decimal::new(10, 2),
        }
    }
}

impl fmt::Display for PriceBoundsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PriceBoundsConfig(liquid={}%, semi={}%, illiquid={}%)",
            self.liquid_tolerance_pct * Decimal::ONE_HUNDRED,
            self.semi_liquid_tolerance_pct * Decimal::ONE_HUNDRED,
            self.illiquid_tolerance_pct * Decimal::ONE_HUNDRED,
        )
    }
}

/// Result of a successful price bounds validation.
///
/// Contains the reference price used, its source, and the computed
/// deviation percentage.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::reference_price::{PriceBoundsResult, ReferencePriceSource};
/// use otc_rfq::domain::value_objects::Price;
/// use rust_decimal::Decimal;
///
/// let result = PriceBoundsResult::new(
///     Price::new(50000.0).unwrap(),
///     ReferencePriceSource::ClobMid,
///     Decimal::new(2, 2),
/// );
/// assert_eq!(result.source(), ReferencePriceSource::ClobMid);
/// assert_eq!(result.deviation_pct(), Decimal::new(2, 2));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceBoundsResult {
    /// The reference price used for comparison.
    reference: Price,
    /// The source from which the reference price was obtained.
    source: ReferencePriceSource,
    /// The absolute deviation of the proposed price from the reference (fractional).
    deviation_pct: Decimal,
}

impl PriceBoundsResult {
    /// Creates a new price bounds result.
    #[must_use]
    pub const fn new(
        reference: Price,
        source: ReferencePriceSource,
        deviation_pct: Decimal,
    ) -> Self {
        Self {
            reference,
            source,
            deviation_pct,
        }
    }

    /// Returns the reference price.
    #[inline]
    #[must_use]
    pub const fn reference(&self) -> Price {
        self.reference
    }

    /// Returns the reference price source.
    #[inline]
    #[must_use]
    pub const fn source(&self) -> ReferencePriceSource {
        self.source
    }

    /// Returns the absolute deviation percentage (fractional).
    #[inline]
    #[must_use]
    pub const fn deviation_pct(&self) -> Decimal {
        self.deviation_pct
    }
}

impl fmt::Display for PriceBoundsResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PriceBoundsResult(ref={}, source={}, deviation={}%)",
            self.reference,
            self.source,
            self.deviation_pct * Decimal::ONE_HUNDRED,
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    mod reference_price_source {
        use super::*;

        #[test]
        fn priority_ordering() {
            assert_eq!(ReferencePriceSource::ClobMid.priority(), 0);
            assert_eq!(ReferencePriceSource::Theoretical.priority(), 1);
            assert_eq!(ReferencePriceSource::ChainlinkIndex.priority(), 2);
        }

        #[test]
        fn display() {
            assert_eq!(ReferencePriceSource::ClobMid.to_string(), "ClobMid");
            assert_eq!(ReferencePriceSource::Theoretical.to_string(), "Theoretical");
            assert_eq!(
                ReferencePriceSource::ChainlinkIndex.to_string(),
                "ChainlinkIndex"
            );
        }

        #[test]
        fn serde_roundtrip() {
            for variant in [
                ReferencePriceSource::ClobMid,
                ReferencePriceSource::Theoretical,
                ReferencePriceSource::ChainlinkIndex,
            ] {
                let json = serde_json::to_string(&variant).unwrap();
                let deserialized: ReferencePriceSource = serde_json::from_str(&json).unwrap();
                assert_eq!(variant, deserialized);
            }
        }

        #[test]
        fn equality() {
            assert_eq!(ReferencePriceSource::ClobMid, ReferencePriceSource::ClobMid);
            assert_ne!(
                ReferencePriceSource::ClobMid,
                ReferencePriceSource::Theoretical
            );
        }
    }

    mod price_bounds_config {
        use super::*;

        #[test]
        fn construction() {
            let config = PriceBoundsConfig::new(
                Decimal::new(5, 2),
                Decimal::new(75, 3),
                Decimal::new(10, 2),
            )
            .unwrap();
            assert_eq!(config.liquid_tolerance_pct(), Decimal::new(5, 2));
            assert_eq!(config.semi_liquid_tolerance_pct(), Decimal::new(75, 3));
            assert_eq!(config.illiquid_tolerance_pct(), Decimal::new(10, 2));
        }

        #[test]
        fn rejects_negative_liquid() {
            assert!(
                PriceBoundsConfig::new(
                    Decimal::new(-1, 2),
                    Decimal::new(5, 2),
                    Decimal::new(10, 2),
                )
                .is_none()
            );
        }

        #[test]
        fn rejects_negative_semi_liquid() {
            assert!(
                PriceBoundsConfig::new(
                    Decimal::new(5, 2),
                    Decimal::new(-1, 2),
                    Decimal::new(10, 2),
                )
                .is_none()
            );
        }

        #[test]
        fn rejects_negative_illiquid() {
            assert!(
                PriceBoundsConfig::new(
                    Decimal::new(5, 2),
                    Decimal::new(75, 3),
                    Decimal::new(-1, 2),
                )
                .is_none()
            );
        }

        #[test]
        fn allows_zero_tolerance() {
            let config =
                PriceBoundsConfig::new(Decimal::ZERO, Decimal::ZERO, Decimal::ZERO).unwrap();
            assert_eq!(config.liquid_tolerance_pct(), Decimal::ZERO);
        }

        #[test]
        fn default_values() {
            let config = PriceBoundsConfig::default();
            assert_eq!(config.liquid_tolerance_pct(), Decimal::new(5, 2));
            assert_eq!(config.semi_liquid_tolerance_pct(), Decimal::new(75, 3));
            assert_eq!(config.illiquid_tolerance_pct(), Decimal::new(10, 2));
        }

        #[test]
        fn tolerance_for_liquidity() {
            let config = PriceBoundsConfig::default();
            assert_eq!(
                config.tolerance_for(LiquidityClassification::Liquid),
                Decimal::new(5, 2)
            );
            assert_eq!(
                config.tolerance_for(LiquidityClassification::SemiLiquid),
                Decimal::new(75, 3)
            );
            assert_eq!(
                config.tolerance_for(LiquidityClassification::Illiquid),
                Decimal::new(10, 2)
            );
        }

        #[test]
        fn display() {
            let config = PriceBoundsConfig::default();
            let s = config.to_string();
            assert!(s.contains("5"));
            assert!(s.contains("7.5"));
            assert!(s.contains("10"));
        }

        #[test]
        fn serde_roundtrip() {
            let config = PriceBoundsConfig::default();
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: PriceBoundsConfig = serde_json::from_str(&json).unwrap();
            assert_eq!(config, deserialized);
        }
    }

    mod price_bounds_result {
        use super::*;

        #[test]
        fn construction_and_accessors() {
            let reference = Price::new(50000.0).unwrap();
            let result = PriceBoundsResult::new(
                reference,
                ReferencePriceSource::ClobMid,
                Decimal::new(2, 2),
            );

            assert_eq!(result.reference(), reference);
            assert_eq!(result.source(), ReferencePriceSource::ClobMid);
            assert_eq!(result.deviation_pct(), Decimal::new(2, 2));
        }

        #[test]
        fn display() {
            let result = PriceBoundsResult::new(
                Price::new(100.0).unwrap(),
                ReferencePriceSource::Theoretical,
                Decimal::new(3, 2),
            );
            let s = result.to_string();
            assert!(s.contains("Theoretical"));
            assert!(s.contains("100"));
        }

        #[test]
        fn serde_roundtrip() {
            let result = PriceBoundsResult::new(
                Price::new(42000.0).unwrap(),
                ReferencePriceSource::ChainlinkIndex,
                Decimal::new(1, 2),
            );
            let json = serde_json::to_string(&result).unwrap();
            let deserialized: PriceBoundsResult = serde_json::from_str(&json).unwrap();
            assert_eq!(result, deserialized);
        }
    }
}
