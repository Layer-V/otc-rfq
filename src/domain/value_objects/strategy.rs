//! # Strategy Value Objects
//!
//! Multi-leg option strategy types and leg composition.
//!
//! This module provides types for defining option strategies composed of
//! multiple legs, each referencing a specific instrument with a direction
//! and quantity ratio.
//!
//! # Supported Strategy Types
//!
//! - [`StrategyType::Spread`] — Vertical/calendar spreads
//! - [`StrategyType::Straddle`] — Same-strike call + put
//! - [`StrategyType::Strangle`] — Different-strike call + put
//! - [`StrategyType::IronCondor`] — Put spread + call spread
//! - [`StrategyType::Butterfly`] — Three-strike spread
//! - [`StrategyType::Custom`] — User-defined multi-leg
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::value_objects::strategy::{Strategy, StrategyType, StrategyLeg};
//! use otc_rfq::domain::value_objects::{Instrument, OrderSide, Symbol, AssetClass};
//!
//! let call = Instrument::builder(
//!     Symbol::new("BTC/USD").unwrap(),
//!     AssetClass::CryptoDerivs,
//! ).build();
//! let put = call.clone();
//!
//! let straddle = Strategy::builder(StrategyType::Straddle, "BTC")
//!     .leg(StrategyLeg::new(call, OrderSide::Buy, 1).unwrap())
//!     .leg(StrategyLeg::new(put, OrderSide::Buy, 1).unwrap())
//!     .build();
//!
//! assert!(straddle.is_ok());
//! ```

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::enums::OrderSide;
use crate::domain::value_objects::instrument::Instrument;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ============================================================================
// StrategyType
// ============================================================================

/// Classification of a multi-leg option strategy.
///
/// Each variant represents a well-known option strategy pattern.
/// Use [`StrategyType::Custom`] for user-defined compositions that
/// do not fit a standard pattern.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::strategy::StrategyType;
///
/// let spread = StrategyType::Spread;
/// assert_eq!(spread.to_string(), "SPREAD");
/// assert_eq!(spread.min_legs(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(u8)]
pub enum StrategyType {
    /// Vertical or calendar spread (2 legs, same underlying).
    Spread = 0,
    /// Straddle: same-strike call + put (2 legs).
    Straddle = 1,
    /// Strangle: different-strike call + put (2 legs).
    Strangle = 2,
    /// Iron condor: put spread + call spread (4 legs).
    IronCondor = 3,
    /// Butterfly: three-strike spread (3 legs).
    Butterfly = 4,
    /// User-defined multi-leg composition (1+ legs).
    Custom = 5,
}

impl StrategyType {
    /// Returns the minimum number of legs required for this strategy type.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::strategy::StrategyType;
    ///
    /// assert_eq!(StrategyType::IronCondor.min_legs(), 4);
    /// assert_eq!(StrategyType::Butterfly.min_legs(), 3);
    /// assert_eq!(StrategyType::Custom.min_legs(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn min_legs(self) -> usize {
        match self {
            Self::Spread | Self::Straddle | Self::Strangle => 2,
            Self::Butterfly => 3,
            Self::IronCondor => 4,
            Self::Custom => 1,
        }
    }

    /// Returns the expected number of legs for this strategy type, if fixed.
    ///
    /// Returns `None` for [`StrategyType::Custom`] which has no fixed count.
    #[inline]
    #[must_use]
    pub const fn expected_legs(self) -> Option<usize> {
        match self {
            Self::Spread | Self::Straddle | Self::Strangle => Some(2),
            Self::Butterfly => Some(3),
            Self::IronCondor => Some(4),
            Self::Custom => None,
        }
    }

    /// Returns true if this is a standard (non-custom) strategy type.
    #[inline]
    #[must_use]
    pub const fn is_standard(self) -> bool {
        !matches!(self, Self::Custom)
    }
}

impl fmt::Display for StrategyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spread => write!(f, "SPREAD"),
            Self::Straddle => write!(f, "STRADDLE"),
            Self::Strangle => write!(f, "STRANGLE"),
            Self::IronCondor => write!(f, "IRON_CONDOR"),
            Self::Butterfly => write!(f, "BUTTERFLY"),
            Self::Custom => write!(f, "CUSTOM"),
        }
    }
}

impl FromStr for StrategyType {
    type Err = crate::domain::value_objects::enums::ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().replace('-', "_").as_str() {
            "SPREAD" => Ok(Self::Spread),
            "STRADDLE" => Ok(Self::Straddle),
            "STRANGLE" => Ok(Self::Strangle),
            "IRON_CONDOR" | "IRONCONDOR" => Ok(Self::IronCondor),
            "BUTTERFLY" => Ok(Self::Butterfly),
            "CUSTOM" => Ok(Self::Custom),
            _ => Err(
                crate::domain::value_objects::enums::ParseEnumError::InvalidValue(
                    "StrategyType",
                    s.to_string(),
                ),
            ),
        }
    }
}

// ============================================================================
// StrategyLeg
// ============================================================================

/// A single leg of a multi-leg option strategy.
///
/// Each leg references a specific instrument with a direction (buy/sell)
/// and a quantity ratio (multiplier relative to the base quantity).
///
/// # Invariants
///
/// - Ratio must be greater than zero
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::strategy::StrategyLeg;
/// use otc_rfq::domain::value_objects::{Instrument, OrderSide, Symbol, AssetClass};
///
/// let instrument = Instrument::builder(
///     Symbol::new("BTC/USD").unwrap(),
///     AssetClass::CryptoDerivs,
/// ).build();
///
/// let leg = StrategyLeg::new(instrument, OrderSide::Buy, 1).unwrap();
/// assert_eq!(leg.side(), OrderSide::Buy);
/// assert_eq!(leg.ratio(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyLeg {
    /// The instrument for this leg.
    instrument: Instrument,
    /// Buy or sell direction.
    side: OrderSide,
    /// Quantity multiplier (must be > 0).
    ratio: u32,
}

impl StrategyLeg {
    /// Creates a new strategy leg.
    ///
    /// # Arguments
    ///
    /// * `instrument` - The instrument for this leg
    /// * `side` - Buy or sell direction
    /// * `ratio` - Quantity multiplier (must be > 0)
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationError`] if ratio is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::value_objects::strategy::StrategyLeg;
    /// use otc_rfq::domain::value_objects::{Instrument, OrderSide, Symbol, AssetClass};
    ///
    /// let instrument = Instrument::builder(
    ///     Symbol::new("ETH/USD").unwrap(),
    ///     AssetClass::CryptoDerivs,
    /// ).build();
    ///
    /// let leg = StrategyLeg::new(instrument, OrderSide::Sell, 2);
    /// assert!(leg.is_ok());
    /// ```
    pub fn new(instrument: Instrument, side: OrderSide, ratio: u32) -> DomainResult<Self> {
        if ratio == 0 {
            return Err(DomainError::ValidationError(
                "strategy leg ratio must be > 0".to_string(),
            ));
        }
        Ok(Self {
            instrument,
            side,
            ratio,
        })
    }

    /// Returns the instrument for this leg.
    #[inline]
    #[must_use]
    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }

    /// Returns the buy/sell direction.
    #[inline]
    #[must_use]
    pub fn side(&self) -> OrderSide {
        self.side
    }

    /// Returns the quantity ratio (multiplier).
    #[inline]
    #[must_use]
    pub fn ratio(&self) -> u32 {
        self.ratio
    }

    /// Returns the underlying (base asset) of this leg's instrument.
    #[inline]
    #[must_use]
    pub fn underlying(&self) -> &str {
        self.instrument.base_asset()
    }
}

impl fmt::Display for StrategyLeg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}x {}",
            self.side,
            self.ratio,
            self.instrument.symbol()
        )
    }
}

// ============================================================================
// Strategy
// ============================================================================

/// A multi-leg option strategy.
///
/// Composes multiple [`StrategyLeg`]s into a named strategy with
/// a classification and optional description.
///
/// # Invariants
///
/// - Must have at least one leg (at least `strategy_type.min_legs()`)
/// - All legs must share the same underlying (base asset)
/// - The `underlying` field must match the legs' base asset
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::strategy::{Strategy, StrategyType, StrategyLeg};
/// use otc_rfq::domain::value_objects::{Instrument, OrderSide, Symbol, AssetClass};
///
/// let inst = Instrument::builder(
///     Symbol::new("BTC/USD").unwrap(),
///     AssetClass::CryptoDerivs,
/// ).build();
///
/// let strategy = Strategy::new(
///     StrategyType::Spread,
///     vec![
///         StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap(),
///         StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap(),
///     ],
///     "BTC",
///     None,
/// );
/// assert!(strategy.is_ok());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strategy {
    /// The strategy classification.
    strategy_type: StrategyType,
    /// The legs composing this strategy.
    legs: Vec<StrategyLeg>,
    /// The underlying asset (e.g., "BTC").
    underlying: String,
    /// Optional human-readable description.
    description: Option<String>,
}

impl Strategy {
    /// Creates a new validated strategy.
    ///
    /// # Arguments
    ///
    /// * `strategy_type` - The strategy classification
    /// * `legs` - The legs composing the strategy
    /// * `underlying` - The underlying asset identifier
    /// * `description` - Optional human-readable description
    ///
    /// # Errors
    ///
    /// - [`DomainError::ValidationError`] if legs are empty
    /// - [`DomainError::ValidationError`] if leg count is below minimum for strategy type
    /// - [`DomainError::ValidationError`] if legs have mixed underlyings
    /// - [`DomainError::ValidationError`] if underlying does not match legs
    pub fn new(
        strategy_type: StrategyType,
        legs: Vec<StrategyLeg>,
        underlying: impl Into<String>,
        description: Option<String>,
    ) -> DomainResult<Self> {
        let underlying = underlying.into();
        Self::validate_legs(&legs, strategy_type, &underlying)?;
        Ok(Self {
            strategy_type,
            legs,
            underlying,
            description,
        })
    }

    /// Creates a [`StrategyBuilder`] for fluent construction.
    ///
    /// # Arguments
    ///
    /// * `strategy_type` - The strategy classification
    /// * `underlying` - The underlying asset identifier
    pub fn builder(strategy_type: StrategyType, underlying: impl Into<String>) -> StrategyBuilder {
        StrategyBuilder::new(strategy_type, underlying)
    }

    /// Validates strategy legs against invariants.
    fn validate_legs(
        legs: &[StrategyLeg],
        strategy_type: StrategyType,
        underlying: &str,
    ) -> DomainResult<()> {
        if legs.is_empty() {
            return Err(DomainError::ValidationError(
                "strategy must have at least one leg".to_string(),
            ));
        }

        let min = strategy_type.min_legs();
        if legs.len() < min {
            return Err(DomainError::ValidationError(format!(
                "strategy type {} requires at least {} legs, got {}",
                strategy_type,
                min,
                legs.len()
            )));
        }

        // Validate all legs share the same underlying
        for leg in legs {
            if leg.underlying() != underlying {
                return Err(DomainError::ValidationError(format!(
                    "leg underlying '{}' does not match strategy underlying '{}'",
                    leg.underlying(),
                    underlying
                )));
            }
        }

        Ok(())
    }

    /// Returns the strategy type.
    #[inline]
    #[must_use]
    pub fn strategy_type(&self) -> StrategyType {
        self.strategy_type
    }

    /// Returns the strategy legs.
    #[inline]
    #[must_use]
    pub fn legs(&self) -> &[StrategyLeg] {
        &self.legs
    }

    /// Returns the number of legs.
    #[inline]
    #[must_use]
    pub fn leg_count(&self) -> usize {
        self.legs.len()
    }

    /// Returns the underlying asset identifier.
    #[inline]
    #[must_use]
    pub fn underlying(&self) -> &str {
        &self.underlying
    }

    /// Returns the optional description.
    #[inline]
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    // ========================================================================
    // Predefined Constructors
    // ========================================================================

    /// Creates a vertical spread (2 legs, same underlying, opposite sides).
    ///
    /// A vertical spread consists of buying one option and selling another
    /// at a different strike but same expiry and underlying.
    ///
    /// # Arguments
    ///
    /// * `long_leg` - The instrument to buy
    /// * `short_leg` - The instrument to sell
    /// * `underlying` - The underlying asset identifier
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationError`] if legs have different underlyings.
    pub fn vertical_spread(
        long_leg: Instrument,
        short_leg: Instrument,
        underlying: impl Into<String>,
    ) -> DomainResult<Self> {
        let legs = vec![
            StrategyLeg::new(long_leg, OrderSide::Buy, 1)?,
            StrategyLeg::new(short_leg, OrderSide::Sell, 1)?,
        ];
        Self::new(
            StrategyType::Spread,
            legs,
            underlying,
            Some("Vertical Spread".to_string()),
        )
    }

    /// Creates a straddle (buy call + buy put at the same strike).
    ///
    /// # Arguments
    ///
    /// * `call_instrument` - The call option instrument
    /// * `put_instrument` - The put option instrument
    /// * `underlying` - The underlying asset identifier
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationError`] if legs have different underlyings.
    pub fn straddle(
        call_instrument: Instrument,
        put_instrument: Instrument,
        underlying: impl Into<String>,
    ) -> DomainResult<Self> {
        let legs = vec![
            StrategyLeg::new(call_instrument, OrderSide::Buy, 1)?,
            StrategyLeg::new(put_instrument, OrderSide::Buy, 1)?,
        ];
        Self::new(
            StrategyType::Straddle,
            legs,
            underlying,
            Some("Straddle".to_string()),
        )
    }

    /// Creates a strangle (buy call + buy put at different strikes).
    ///
    /// # Arguments
    ///
    /// * `call_instrument` - The call option instrument (higher strike)
    /// * `put_instrument` - The put option instrument (lower strike)
    /// * `underlying` - The underlying asset identifier
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationError`] if legs have different underlyings.
    pub fn strangle(
        call_instrument: Instrument,
        put_instrument: Instrument,
        underlying: impl Into<String>,
    ) -> DomainResult<Self> {
        let legs = vec![
            StrategyLeg::new(call_instrument, OrderSide::Buy, 1)?,
            StrategyLeg::new(put_instrument, OrderSide::Buy, 1)?,
        ];
        Self::new(
            StrategyType::Strangle,
            legs,
            underlying,
            Some("Strangle".to_string()),
        )
    }

    /// Creates an iron condor (4 legs: put spread + call spread).
    ///
    /// An iron condor consists of:
    /// 1. Buy low-strike put (protection)
    /// 2. Sell higher-strike put (collect premium)
    /// 3. Sell lower-strike call (collect premium)
    /// 4. Buy higher-strike call (protection)
    ///
    /// # Arguments
    ///
    /// * `buy_put` - Low-strike put (buy for protection)
    /// * `sell_put` - Higher-strike put (sell for premium)
    /// * `sell_call` - Lower-strike call (sell for premium)
    /// * `buy_call` - Higher-strike call (buy for protection)
    /// * `underlying` - The underlying asset identifier
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationError`] if legs have different underlyings.
    pub fn iron_condor(
        buy_put: Instrument,
        sell_put: Instrument,
        sell_call: Instrument,
        buy_call: Instrument,
        underlying: impl Into<String>,
    ) -> DomainResult<Self> {
        let legs = vec![
            StrategyLeg::new(buy_put, OrderSide::Buy, 1)?,
            StrategyLeg::new(sell_put, OrderSide::Sell, 1)?,
            StrategyLeg::new(sell_call, OrderSide::Sell, 1)?,
            StrategyLeg::new(buy_call, OrderSide::Buy, 1)?,
        ];
        Self::new(
            StrategyType::IronCondor,
            legs,
            underlying,
            Some("Iron Condor".to_string()),
        )
    }

    /// Creates a butterfly spread (3 legs: buy 1, sell 2, buy 1).
    ///
    /// A butterfly consists of:
    /// 1. Buy low-strike option (1x)
    /// 2. Sell middle-strike option (2x)
    /// 3. Buy high-strike option (1x)
    ///
    /// # Arguments
    ///
    /// * `low_strike` - Low-strike option to buy
    /// * `mid_strike` - Middle-strike option to sell (2x ratio)
    /// * `high_strike` - High-strike option to buy
    /// * `underlying` - The underlying asset identifier
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationError`] if legs have different underlyings.
    pub fn butterfly(
        low_strike: Instrument,
        mid_strike: Instrument,
        high_strike: Instrument,
        underlying: impl Into<String>,
    ) -> DomainResult<Self> {
        let legs = vec![
            StrategyLeg::new(low_strike, OrderSide::Buy, 1)?,
            StrategyLeg::new(mid_strike, OrderSide::Sell, 2)?,
            StrategyLeg::new(high_strike, OrderSide::Buy, 1)?,
        ];
        Self::new(
            StrategyType::Butterfly,
            legs,
            underlying,
            Some("Butterfly".to_string()),
        )
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} on {} ({} legs)",
            self.strategy_type,
            self.underlying,
            self.legs.len()
        )
    }
}

// ============================================================================
// StrategyBuilder
// ============================================================================

/// Builder for constructing [`Strategy`] instances with a fluent API.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::strategy::{Strategy, StrategyType, StrategyLeg};
/// use otc_rfq::domain::value_objects::{Instrument, OrderSide, Symbol, AssetClass};
///
/// let inst = Instrument::builder(
///     Symbol::new("ETH/USD").unwrap(),
///     AssetClass::CryptoDerivs,
/// ).build();
///
/// let strategy = Strategy::builder(StrategyType::Custom, "ETH")
///     .description("My custom strategy")
///     .leg(StrategyLeg::new(inst, OrderSide::Buy, 1).unwrap())
///     .build();
///
/// assert!(strategy.is_ok());
/// ```
#[derive(Debug, Clone)]
#[must_use = "builders do nothing unless .build() is called"]
pub struct StrategyBuilder {
    strategy_type: StrategyType,
    underlying: String,
    description: Option<String>,
    legs: Vec<StrategyLeg>,
}

impl StrategyBuilder {
    /// Creates a new builder.
    fn new(strategy_type: StrategyType, underlying: impl Into<String>) -> Self {
        Self {
            strategy_type,
            underlying: underlying.into(),
            description: None,
            legs: Vec::new(),
        }
    }

    /// Sets an optional description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a leg to the strategy.
    pub fn leg(mut self, leg: StrategyLeg) -> Self {
        self.legs.push(leg);
        self
    }

    /// Adds multiple legs to the strategy.
    pub fn legs(mut self, legs: impl IntoIterator<Item = StrategyLeg>) -> Self {
        self.legs.extend(legs);
        self
    }

    /// Builds and validates the strategy.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ValidationError`] if validation fails.
    pub fn build(self) -> DomainResult<Strategy> {
        Strategy::new(
            self.strategy_type,
            self.legs,
            self.underlying,
            self.description,
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::domain::value_objects::enums::AssetClass;
    use crate::domain::value_objects::symbol::Symbol;

    /// Helper: creates a CryptoDerivs instrument for the given symbol string.
    fn make_instrument(symbol_str: &str) -> Instrument {
        let symbol = Symbol::new(symbol_str).unwrap();
        Instrument::builder(symbol, AssetClass::CryptoDerivs).build()
    }

    // ====================================================================
    // StrategyType tests
    // ====================================================================

    mod strategy_type_tests {
        use super::*;

        #[test]
        fn display_all_variants() {
            assert_eq!(StrategyType::Spread.to_string(), "SPREAD");
            assert_eq!(StrategyType::Straddle.to_string(), "STRADDLE");
            assert_eq!(StrategyType::Strangle.to_string(), "STRANGLE");
            assert_eq!(StrategyType::IronCondor.to_string(), "IRON_CONDOR");
            assert_eq!(StrategyType::Butterfly.to_string(), "BUTTERFLY");
            assert_eq!(StrategyType::Custom.to_string(), "CUSTOM");
        }

        #[test]
        fn from_str_valid() {
            assert_eq!(
                "SPREAD".parse::<StrategyType>().unwrap(),
                StrategyType::Spread
            );
            assert_eq!(
                "straddle".parse::<StrategyType>().unwrap(),
                StrategyType::Straddle
            );
            assert_eq!(
                "IRON_CONDOR".parse::<StrategyType>().unwrap(),
                StrategyType::IronCondor
            );
            assert_eq!(
                "IRONCONDOR".parse::<StrategyType>().unwrap(),
                StrategyType::IronCondor
            );
            assert_eq!(
                "iron-condor".parse::<StrategyType>().unwrap(),
                StrategyType::IronCondor
            );
            assert_eq!(
                "butterfly".parse::<StrategyType>().unwrap(),
                StrategyType::Butterfly
            );
            assert_eq!(
                "CUSTOM".parse::<StrategyType>().unwrap(),
                StrategyType::Custom
            );
        }

        #[test]
        fn from_str_invalid() {
            assert!("UNKNOWN".parse::<StrategyType>().is_err());
            assert!("".parse::<StrategyType>().is_err());
        }

        #[test]
        fn serde_roundtrip() {
            for variant in [
                StrategyType::Spread,
                StrategyType::Straddle,
                StrategyType::Strangle,
                StrategyType::IronCondor,
                StrategyType::Butterfly,
                StrategyType::Custom,
            ] {
                let json = serde_json::to_string(&variant).unwrap();
                let deserialized: StrategyType = serde_json::from_str(&json).unwrap();
                assert_eq!(variant, deserialized);
            }
        }

        #[test]
        fn serde_values() {
            assert_eq!(
                serde_json::to_string(&StrategyType::Spread).unwrap(),
                "\"SPREAD\""
            );
            assert_eq!(
                serde_json::to_string(&StrategyType::IronCondor).unwrap(),
                "\"IRON_CONDOR\""
            );
        }

        #[test]
        fn repr_values() {
            assert_eq!(StrategyType::Spread as u8, 0);
            assert_eq!(StrategyType::Straddle as u8, 1);
            assert_eq!(StrategyType::Strangle as u8, 2);
            assert_eq!(StrategyType::IronCondor as u8, 3);
            assert_eq!(StrategyType::Butterfly as u8, 4);
            assert_eq!(StrategyType::Custom as u8, 5);
        }

        #[test]
        fn min_legs() {
            assert_eq!(StrategyType::Spread.min_legs(), 2);
            assert_eq!(StrategyType::Straddle.min_legs(), 2);
            assert_eq!(StrategyType::Strangle.min_legs(), 2);
            assert_eq!(StrategyType::Butterfly.min_legs(), 3);
            assert_eq!(StrategyType::IronCondor.min_legs(), 4);
            assert_eq!(StrategyType::Custom.min_legs(), 1);
        }

        #[test]
        fn expected_legs() {
            assert_eq!(StrategyType::Spread.expected_legs(), Some(2));
            assert_eq!(StrategyType::Straddle.expected_legs(), Some(2));
            assert_eq!(StrategyType::Strangle.expected_legs(), Some(2));
            assert_eq!(StrategyType::Butterfly.expected_legs(), Some(3));
            assert_eq!(StrategyType::IronCondor.expected_legs(), Some(4));
            assert_eq!(StrategyType::Custom.expected_legs(), None);
        }

        #[test]
        fn is_standard() {
            assert!(StrategyType::Spread.is_standard());
            assert!(StrategyType::IronCondor.is_standard());
            assert!(!StrategyType::Custom.is_standard());
        }
    }

    // ====================================================================
    // StrategyLeg tests
    // ====================================================================

    mod strategy_leg_tests {
        use super::*;

        #[test]
        fn construction_valid() {
            let inst = make_instrument("BTC/USD");
            let leg = StrategyLeg::new(inst.clone(), OrderSide::Buy, 1);
            assert!(leg.is_ok());
            let leg = leg.unwrap();
            assert_eq!(leg.instrument(), &inst);
            assert_eq!(leg.side(), OrderSide::Buy);
            assert_eq!(leg.ratio(), 1);
        }

        #[test]
        fn construction_ratio_two() {
            let inst = make_instrument("ETH/USD");
            let leg = StrategyLeg::new(inst, OrderSide::Sell, 2).unwrap();
            assert_eq!(leg.ratio(), 2);
            assert_eq!(leg.side(), OrderSide::Sell);
        }

        #[test]
        fn construction_zero_ratio_rejected() {
            let inst = make_instrument("BTC/USD");
            let result = StrategyLeg::new(inst, OrderSide::Buy, 0);
            assert!(result.is_err());
            assert!(matches!(result, Err(DomainError::ValidationError(_))));
        }

        #[test]
        fn underlying_accessor() {
            let inst = make_instrument("BTC/USD");
            let leg = StrategyLeg::new(inst, OrderSide::Buy, 1).unwrap();
            assert_eq!(leg.underlying(), "BTC");
        }

        #[test]
        fn display_format() {
            let inst = make_instrument("BTC/USD");
            let leg = StrategyLeg::new(inst, OrderSide::Buy, 2).unwrap();
            assert_eq!(leg.to_string(), "BUY 2x BTC/USD");
        }

        #[test]
        fn serde_roundtrip() {
            let inst = make_instrument("ETH/USD");
            let leg = StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap();

            let json = serde_json::to_string(&leg).unwrap();
            let deserialized: StrategyLeg = serde_json::from_str(&json).unwrap();
            assert_eq!(leg, deserialized);
        }
    }

    // ====================================================================
    // Strategy validation tests
    // ====================================================================

    mod strategy_validation_tests {
        use super::*;

        #[test]
        fn empty_legs_rejected() {
            let result = Strategy::new(StrategyType::Custom, vec![], "BTC", None);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, DomainError::ValidationError(_)));
            assert!(err.to_string().contains("at least one leg"));
        }

        #[test]
        fn below_minimum_legs_rejected() {
            let inst = make_instrument("BTC/USD");
            let leg = StrategyLeg::new(inst, OrderSide::Buy, 1).unwrap();

            // Spread requires 2 legs, we only provide 1
            let result = Strategy::new(StrategyType::Spread, vec![leg], "BTC", None);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.to_string().contains("requires at least 2 legs"));
        }

        #[test]
        fn iron_condor_requires_four_legs() {
            let inst = make_instrument("BTC/USD");
            let legs: Vec<_> = (0..3)
                .map(|_| StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap())
                .collect();

            let result = Strategy::new(StrategyType::IronCondor, legs, "BTC", None);
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("requires at least 4 legs")
            );
        }

        #[test]
        fn mixed_underlyings_rejected() {
            let btc = make_instrument("BTC/USD");
            let eth = make_instrument("ETH/USD");

            let legs = vec![
                StrategyLeg::new(btc, OrderSide::Buy, 1).unwrap(),
                StrategyLeg::new(eth, OrderSide::Sell, 1).unwrap(),
            ];

            let result = Strategy::new(StrategyType::Spread, legs, "BTC", None);
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("does not match strategy underlying")
            );
        }

        #[test]
        fn underlying_mismatch_rejected() {
            let inst = make_instrument("BTC/USD");
            let legs = vec![
                StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap(),
                StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap(),
            ];

            // Legs say BTC, but underlying says ETH
            let result = Strategy::new(StrategyType::Spread, legs, "ETH", None);
            assert!(result.is_err());
        }

        #[test]
        fn valid_spread_accepted() {
            let inst = make_instrument("BTC/USD");
            let legs = vec![
                StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap(),
                StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap(),
            ];

            let result = Strategy::new(StrategyType::Spread, legs, "BTC", None);
            assert!(result.is_ok());
        }

        #[test]
        fn custom_with_one_leg_accepted() {
            let inst = make_instrument("BTC/USD");
            let legs = vec![StrategyLeg::new(inst, OrderSide::Buy, 1).unwrap()];

            let result = Strategy::new(StrategyType::Custom, legs, "BTC", None);
            assert!(result.is_ok());
        }
    }

    // ====================================================================
    // Strategy accessors tests
    // ====================================================================

    mod strategy_accessor_tests {
        use super::*;

        #[test]
        fn all_accessors() {
            let inst = make_instrument("BTC/USD");
            let legs = vec![
                StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap(),
                StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap(),
            ];

            let strategy = Strategy::new(
                StrategyType::Spread,
                legs,
                "BTC",
                Some("My spread".to_string()),
            )
            .unwrap();

            assert_eq!(strategy.strategy_type(), StrategyType::Spread);
            assert_eq!(strategy.leg_count(), 2);
            assert_eq!(strategy.underlying(), "BTC");
            assert_eq!(strategy.description(), Some("My spread"));
            assert_eq!(strategy.legs().len(), 2);
        }

        #[test]
        fn description_none() {
            let inst = make_instrument("ETH/USD");
            let legs = vec![StrategyLeg::new(inst, OrderSide::Buy, 1).unwrap()];

            let strategy = Strategy::new(StrategyType::Custom, legs, "ETH", None).unwrap();
            assert_eq!(strategy.description(), None);
        }

        #[test]
        fn display_format() {
            let inst = make_instrument("BTC/USD");
            let legs = vec![
                StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap(),
                StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap(),
            ];

            let strategy = Strategy::new(StrategyType::Spread, legs, "BTC", None).unwrap();
            assert_eq!(strategy.to_string(), "SPREAD on BTC (2 legs)");
        }
    }

    // ====================================================================
    // Predefined constructor tests
    // ====================================================================

    mod predefined_constructor_tests {
        use super::*;

        #[test]
        fn vertical_spread() {
            let long = make_instrument("BTC/USD");
            let short = make_instrument("BTC/USD");

            let strategy = Strategy::vertical_spread(long, short, "BTC").unwrap();
            assert_eq!(strategy.strategy_type(), StrategyType::Spread);
            assert_eq!(strategy.leg_count(), 2);
            assert_eq!(strategy.underlying(), "BTC");
            assert_eq!(strategy.description(), Some("Vertical Spread"));
            assert_eq!(strategy.legs()[0].side(), OrderSide::Buy);
            assert_eq!(strategy.legs()[1].side(), OrderSide::Sell);
        }

        #[test]
        fn vertical_spread_mixed_underlying_fails() {
            let long = make_instrument("BTC/USD");
            let short = make_instrument("ETH/USD");

            let result = Strategy::vertical_spread(long, short, "BTC");
            assert!(result.is_err());
        }

        #[test]
        fn straddle() {
            let call = make_instrument("BTC/USD");
            let put = make_instrument("BTC/USD");

            let strategy = Strategy::straddle(call, put, "BTC").unwrap();
            assert_eq!(strategy.strategy_type(), StrategyType::Straddle);
            assert_eq!(strategy.leg_count(), 2);
            assert_eq!(strategy.description(), Some("Straddle"));
            // Both legs should be Buy
            assert_eq!(strategy.legs()[0].side(), OrderSide::Buy);
            assert_eq!(strategy.legs()[1].side(), OrderSide::Buy);
        }

        #[test]
        fn strangle() {
            let call = make_instrument("BTC/USD");
            let put = make_instrument("BTC/USD");

            let strategy = Strategy::strangle(call, put, "BTC").unwrap();
            assert_eq!(strategy.strategy_type(), StrategyType::Strangle);
            assert_eq!(strategy.leg_count(), 2);
            assert_eq!(strategy.description(), Some("Strangle"));
        }

        #[test]
        fn iron_condor() {
            let buy_put = make_instrument("BTC/USD");
            let sell_put = make_instrument("BTC/USD");
            let sell_call = make_instrument("BTC/USD");
            let buy_call = make_instrument("BTC/USD");

            let strategy =
                Strategy::iron_condor(buy_put, sell_put, sell_call, buy_call, "BTC").unwrap();
            assert_eq!(strategy.strategy_type(), StrategyType::IronCondor);
            assert_eq!(strategy.leg_count(), 4);
            assert_eq!(strategy.description(), Some("Iron Condor"));
            assert_eq!(strategy.legs()[0].side(), OrderSide::Buy);
            assert_eq!(strategy.legs()[1].side(), OrderSide::Sell);
            assert_eq!(strategy.legs()[2].side(), OrderSide::Sell);
            assert_eq!(strategy.legs()[3].side(), OrderSide::Buy);
        }

        #[test]
        fn iron_condor_mixed_underlying_fails() {
            let buy_put = make_instrument("BTC/USD");
            let sell_put = make_instrument("BTC/USD");
            let sell_call = make_instrument("ETH/USD"); // different underlying
            let buy_call = make_instrument("BTC/USD");

            let result = Strategy::iron_condor(buy_put, sell_put, sell_call, buy_call, "BTC");
            assert!(result.is_err());
        }

        #[test]
        fn butterfly() {
            let low = make_instrument("BTC/USD");
            let mid = make_instrument("BTC/USD");
            let high = make_instrument("BTC/USD");

            let strategy = Strategy::butterfly(low, mid, high, "BTC").unwrap();
            assert_eq!(strategy.strategy_type(), StrategyType::Butterfly);
            assert_eq!(strategy.leg_count(), 3);
            assert_eq!(strategy.description(), Some("Butterfly"));
            assert_eq!(strategy.legs()[0].side(), OrderSide::Buy);
            assert_eq!(strategy.legs()[0].ratio(), 1);
            assert_eq!(strategy.legs()[1].side(), OrderSide::Sell);
            assert_eq!(strategy.legs()[1].ratio(), 2);
            assert_eq!(strategy.legs()[2].side(), OrderSide::Buy);
            assert_eq!(strategy.legs()[2].ratio(), 1);
        }
    }

    // ====================================================================
    // StrategyBuilder tests
    // ====================================================================

    mod builder_tests {
        use super::*;

        #[test]
        fn builder_with_all_fields() {
            let inst = make_instrument("BTC/USD");

            let strategy = Strategy::builder(StrategyType::Spread, "BTC")
                .description("My spread")
                .leg(StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap())
                .leg(StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap())
                .build()
                .unwrap();

            assert_eq!(strategy.strategy_type(), StrategyType::Spread);
            assert_eq!(strategy.underlying(), "BTC");
            assert_eq!(strategy.description(), Some("My spread"));
            assert_eq!(strategy.leg_count(), 2);
        }

        #[test]
        fn builder_with_legs_batch() {
            let inst = make_instrument("ETH/USD");
            let legs = vec![
                StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap(),
                StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap(),
            ];

            let strategy = Strategy::builder(StrategyType::Spread, "ETH")
                .legs(legs)
                .build()
                .unwrap();

            assert_eq!(strategy.leg_count(), 2);
        }

        #[test]
        fn builder_no_description() {
            let inst = make_instrument("BTC/USD");

            let strategy = Strategy::builder(StrategyType::Custom, "BTC")
                .leg(StrategyLeg::new(inst, OrderSide::Buy, 1).unwrap())
                .build()
                .unwrap();

            assert_eq!(strategy.description(), None);
        }

        #[test]
        fn builder_empty_legs_fails() {
            let result = Strategy::builder(StrategyType::Spread, "BTC").build();
            assert!(result.is_err());
        }

        #[test]
        fn builder_validation_applies() {
            let btc = make_instrument("BTC/USD");
            let eth = make_instrument("ETH/USD");

            let result = Strategy::builder(StrategyType::Spread, "BTC")
                .leg(StrategyLeg::new(btc, OrderSide::Buy, 1).unwrap())
                .leg(StrategyLeg::new(eth, OrderSide::Sell, 1).unwrap())
                .build();

            assert!(result.is_err());
        }
    }

    // ====================================================================
    // Serde tests
    // ====================================================================

    mod serde_tests {
        use super::*;

        #[test]
        fn full_strategy_roundtrip() {
            let inst = make_instrument("BTC/USD");
            let strategy = Strategy::new(
                StrategyType::Spread,
                vec![
                    StrategyLeg::new(inst.clone(), OrderSide::Buy, 1).unwrap(),
                    StrategyLeg::new(inst, OrderSide::Sell, 1).unwrap(),
                ],
                "BTC",
                Some("Test spread".to_string()),
            )
            .unwrap();

            let json = serde_json::to_string(&strategy).unwrap();
            let deserialized: Strategy = serde_json::from_str(&json).unwrap();
            assert_eq!(strategy, deserialized);
        }

        #[test]
        fn iron_condor_roundtrip() {
            let inst = make_instrument("ETH/USD");
            let strategy =
                Strategy::iron_condor(inst.clone(), inst.clone(), inst.clone(), inst, "ETH")
                    .unwrap();

            let json = serde_json::to_string(&strategy).unwrap();
            let deserialized: Strategy = serde_json::from_str(&json).unwrap();
            assert_eq!(strategy, deserialized);
        }

        #[test]
        fn butterfly_roundtrip() {
            let inst = make_instrument("BTC/USD");
            let strategy = Strategy::butterfly(inst.clone(), inst.clone(), inst, "BTC").unwrap();

            let json = serde_json::to_string(&strategy).unwrap();
            let deserialized: Strategy = serde_json::from_str(&json).unwrap();
            assert_eq!(strategy, deserialized);
        }

        #[test]
        fn no_description_roundtrip() {
            let inst = make_instrument("BTC/USD");
            let strategy = Strategy::new(
                StrategyType::Custom,
                vec![StrategyLeg::new(inst, OrderSide::Buy, 1).unwrap()],
                "BTC",
                None,
            )
            .unwrap();

            let json = serde_json::to_string(&strategy).unwrap();
            let deserialized: Strategy = serde_json::from_str(&json).unwrap();
            assert_eq!(strategy.description(), deserialized.description());
        }
    }
}
