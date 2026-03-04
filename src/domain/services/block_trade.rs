//! # Block Trade Configuration
//!
//! Configurable size thresholds and reporting tiers for block trades.
//!
//! Block trades are large pre-arranged trades that receive special treatment:
//! - Off-book execution
//! - Delayed reporting based on size
//! - Reduced market impact
//!
//! ## Architecture
//!
//! - BTC block trades: ≥ 25 BTC
//! - ETH block trades: ≥ 250 ETH
//! - Reporting tiers based on size multiples (5x = Large, 10x = VeryLarge)

use std::collections::HashMap;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{Instrument, Quantity};

/// Configuration for block trade size thresholds and reporting tiers.
///
/// Determines when a trade qualifies as a block trade and what reporting
/// tier applies based on trade size.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::services::BlockTradeConfig;
/// use otc_rfq::domain::value_objects::{Instrument, Quantity, Symbol, AssetClass};
///
/// let config = BlockTradeConfig::default();
/// let symbol = Symbol::new("BTC/USD").unwrap();
/// let instrument = Instrument::builder(symbol, AssetClass::CryptoSpot).build();
/// let quantity = Quantity::new(30.0).unwrap();
///
/// assert!(config.qualifies(&instrument, quantity));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTradeConfig {
    /// Per-instrument minimum sizes for block trade qualification.
    ///
    /// Maps underlying symbol (e.g., "BTC", "ETH") to minimum quantity.
    pub thresholds: HashMap<String, Quantity>,

    /// Default threshold for instruments without specific configuration.
    ///
    /// If `None`, instruments without specific thresholds will not qualify
    /// as block trades regardless of size.
    pub default_threshold: Option<Quantity>,

    /// Multipliers for determining reporting tiers based on threshold multiples.
    pub tier_multipliers: TierMultipliers,
}

/// Multipliers for determining reporting tier boundaries.
///
/// Tiers are determined by comparing trade size to the base threshold:
/// - Standard: 1x to `large` multiplier
/// - Large: `large` to `very_large` multiplier
/// - VeryLarge: `very_large` and above
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TierMultipliers {
    /// Multiplier for Large tier threshold (default: 5.0).
    ///
    /// A trade is considered Large if its size is at least this multiple
    /// of the base threshold.
    pub large: f64,

    /// Multiplier for VeryLarge tier threshold (default: 10.0).
    ///
    /// A trade is considered VeryLarge if its size is at least this multiple
    /// of the base threshold.
    pub very_large: f64,
}

/// Reporting tier for block trades based on size.
///
/// Larger trades receive longer reporting delays to minimize market impact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[must_use]
pub enum ReportingTier {
    /// Standard block trade with 15 minute reporting delay.
    Standard,

    /// Large block trade with 60 minute reporting delay.
    Large,

    /// Very large block trade with end-of-day reporting delay.
    VeryLarge,
}

impl BlockTradeConfig {
    /// Creates a new block trade configuration with default thresholds.
    ///
    /// Default configuration:
    /// - BTC: 25.0
    /// - ETH: 250.0
    /// - Large tier: 5x threshold
    /// - VeryLarge tier: 10x threshold
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if a trade qualifies as a block trade.
    ///
    /// A trade qualifies if its quantity meets or exceeds the configured
    /// threshold for the instrument's underlying asset.
    ///
    /// # Arguments
    ///
    /// * `instrument` - The trading instrument
    /// * `quantity` - The trade quantity
    ///
    /// # Returns
    ///
    /// `true` if the trade qualifies as a block trade, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::services::BlockTradeConfig;
    /// use otc_rfq::domain::value_objects::{Instrument, Quantity, Symbol, AssetClass};
    ///
    /// let config = BlockTradeConfig::default();
    /// let symbol = Symbol::new("BTC/USD").unwrap();
    /// let instrument = Instrument::builder(symbol, AssetClass::CryptoSpot).build();
    ///
    /// assert!(config.qualifies(&instrument, Quantity::new(25.0).unwrap()));
    /// assert!(!config.qualifies(&instrument, Quantity::new(24.9).unwrap()));
    /// ```
    #[must_use]
    pub fn qualifies(&self, instrument: &Instrument, quantity: Quantity) -> bool {
        if let Some(threshold) = self.get_threshold(instrument) {
            quantity >= threshold
        } else {
            false
        }
    }

    /// Gets the block trade threshold for a specific instrument.
    ///
    /// Returns the configured threshold for the instrument's underlying asset,
    /// or the default threshold if no specific configuration exists.
    ///
    /// # Arguments
    ///
    /// * `instrument` - The trading instrument
    ///
    /// # Returns
    ///
    /// The threshold quantity, or `None` if no threshold is configured.
    #[must_use]
    pub fn get_threshold(&self, instrument: &Instrument) -> Option<Quantity> {
        let underlying = instrument.base_asset();
        self.thresholds
            .get(underlying)
            .copied()
            .or(self.default_threshold)
    }

    /// Determines the reporting tier for a block trade.
    ///
    /// The tier is based on how many times larger the trade is compared to
    /// the base threshold:
    /// - Standard: 1x to 5x threshold
    /// - Large: 5x to 10x threshold
    /// - VeryLarge: 10x threshold and above
    ///
    /// # Arguments
    ///
    /// * `instrument` - The trading instrument
    /// * `quantity` - The trade quantity
    ///
    /// # Returns
    ///
    /// The reporting tier, or `None` if the trade does not qualify as a block trade.
    ///
    /// # Examples
    ///
    /// ```
    /// use otc_rfq::domain::services::{BlockTradeConfig, ReportingTier};
    /// use otc_rfq::domain::value_objects::{Instrument, Quantity, Symbol, AssetClass};
    ///
    /// let config = BlockTradeConfig::default();
    /// let symbol = Symbol::new("BTC/USD").unwrap();
    /// let instrument = Instrument::builder(symbol, AssetClass::CryptoSpot).build();
    ///
    /// assert_eq!(
    ///     config.determine_tier(&instrument, Quantity::new(25.0).unwrap()),
    ///     Some(ReportingTier::Standard)
    /// );
    /// assert_eq!(
    ///     config.determine_tier(&instrument, Quantity::new(125.0).unwrap()),
    ///     Some(ReportingTier::Large)
    /// );
    /// assert_eq!(
    ///     config.determine_tier(&instrument, Quantity::new(250.0).unwrap()),
    ///     Some(ReportingTier::VeryLarge)
    /// );
    /// ```
    #[must_use]
    pub fn determine_tier(
        &self,
        instrument: &Instrument,
        quantity: Quantity,
    ) -> Option<ReportingTier> {
        let threshold = self.get_threshold(instrument)?;

        if !self.qualifies(instrument, quantity) {
            return None;
        }

        let ratio =
            quantity.get().to_f64().unwrap_or(0.0) / threshold.get().to_f64().unwrap_or(1.0);

        if ratio >= self.tier_multipliers.very_large {
            Some(ReportingTier::VeryLarge)
        } else if ratio >= self.tier_multipliers.large {
            Some(ReportingTier::Large)
        } else {
            Some(ReportingTier::Standard)
        }
    }
}

impl Default for BlockTradeConfig {
    fn default() -> Self {
        let mut thresholds = HashMap::new();

        // SAFETY: These are valid positive decimal values that will never fail
        // BTC threshold: 25.0
        if let Ok(btc_threshold) = Quantity::from_decimal(Decimal::new(25, 0)) {
            thresholds.insert("BTC".to_string(), btc_threshold);
        }
        // ETH threshold: 250.0
        if let Ok(eth_threshold) = Quantity::from_decimal(Decimal::new(250, 0)) {
            thresholds.insert("ETH".to_string(), eth_threshold);
        }

        Self {
            thresholds,
            default_threshold: None,
            tier_multipliers: TierMultipliers::default(),
        }
    }
}

impl Default for TierMultipliers {
    fn default() -> Self {
        Self {
            large: 5.0,
            very_large: 10.0,
        }
    }
}

impl ReportingTier {
    /// Returns the reporting delay in minutes for this tier.
    ///
    /// - Standard: 15 minutes
    /// - Large: 60 minutes
    /// - VeryLarge: 1440 minutes (end of day / 24 hours)
    #[must_use]
    pub const fn delay_minutes(&self) -> u32 {
        match self {
            Self::Standard => 15,
            Self::Large => 60,
            Self::VeryLarge => 1440,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{AssetClass, Symbol};

    fn create_btc_instrument() -> Instrument {
        let symbol = Symbol::new("BTC/USD").unwrap();
        Instrument::builder(symbol, AssetClass::CryptoSpot).build()
    }

    fn create_eth_instrument() -> Instrument {
        let symbol = Symbol::new("ETH/USD").unwrap();
        Instrument::builder(symbol, AssetClass::CryptoSpot).build()
    }

    fn create_unknown_instrument() -> Instrument {
        let symbol = Symbol::new("SOL/USD").unwrap();
        Instrument::builder(symbol, AssetClass::CryptoSpot).build()
    }

    #[test]
    fn default_configuration_has_correct_thresholds() {
        let config = BlockTradeConfig::default();

        assert_eq!(
            config.thresholds.get("BTC"),
            Some(&Quantity::new(25.0).unwrap())
        );
        assert_eq!(
            config.thresholds.get("ETH"),
            Some(&Quantity::new(250.0).unwrap())
        );
        assert_eq!(config.default_threshold, None);
    }

    #[test]
    fn default_tier_multipliers_are_correct() {
        let multipliers = TierMultipliers::default();

        assert_eq!(multipliers.large, 5.0);
        assert_eq!(multipliers.very_large, 10.0);
    }

    #[test]
    fn btc_at_threshold_qualifies() {
        let config = BlockTradeConfig::default();
        let instrument = create_btc_instrument();
        let quantity = Quantity::new(25.0).unwrap();

        assert!(config.qualifies(&instrument, quantity));
    }

    #[test]
    fn btc_below_threshold_does_not_qualify() {
        let config = BlockTradeConfig::default();
        let instrument = create_btc_instrument();
        let quantity = Quantity::new(24.9).unwrap();

        assert!(!config.qualifies(&instrument, quantity));
    }

    #[test]
    fn btc_above_threshold_qualifies() {
        let config = BlockTradeConfig::default();
        let instrument = create_btc_instrument();
        let quantity = Quantity::new(30.0).unwrap();

        assert!(config.qualifies(&instrument, quantity));
    }

    #[test]
    fn eth_at_threshold_qualifies() {
        let config = BlockTradeConfig::default();
        let instrument = create_eth_instrument();
        let quantity = Quantity::new(250.0).unwrap();

        assert!(config.qualifies(&instrument, quantity));
    }

    #[test]
    fn eth_below_threshold_does_not_qualify() {
        let config = BlockTradeConfig::default();
        let instrument = create_eth_instrument();
        let quantity = Quantity::new(249.9).unwrap();

        assert!(!config.qualifies(&instrument, quantity));
    }

    #[test]
    fn unknown_instrument_without_default_does_not_qualify() {
        let config = BlockTradeConfig::default();
        let instrument = create_unknown_instrument();
        let quantity = Quantity::new(1000.0).unwrap();

        assert!(!config.qualifies(&instrument, quantity));
    }

    #[test]
    fn unknown_instrument_with_default_qualifies_if_above_default() {
        let config = BlockTradeConfig {
            default_threshold: Some(Quantity::new(100.0).unwrap()),
            ..Default::default()
        };

        let instrument = create_unknown_instrument();
        let quantity = Quantity::new(150.0).unwrap();

        assert!(config.qualifies(&instrument, quantity));
    }

    #[test]
    fn btc_1x_threshold_is_standard_tier() {
        let config = BlockTradeConfig::default();
        let instrument = create_btc_instrument();
        let quantity = Quantity::new(25.0).unwrap();

        assert_eq!(
            config.determine_tier(&instrument, quantity),
            Some(ReportingTier::Standard)
        );
    }

    #[test]
    fn btc_5x_threshold_is_large_tier() {
        let config = BlockTradeConfig::default();
        let instrument = create_btc_instrument();
        let quantity = Quantity::new(125.0).unwrap();

        assert_eq!(
            config.determine_tier(&instrument, quantity),
            Some(ReportingTier::Large)
        );
    }

    #[test]
    fn btc_10x_threshold_is_very_large_tier() {
        let config = BlockTradeConfig::default();
        let instrument = create_btc_instrument();
        let quantity = Quantity::new(250.0).unwrap();

        assert_eq!(
            config.determine_tier(&instrument, quantity),
            Some(ReportingTier::VeryLarge)
        );
    }

    #[test]
    fn eth_1x_threshold_is_standard_tier() {
        let config = BlockTradeConfig::default();
        let instrument = create_eth_instrument();
        let quantity = Quantity::new(250.0).unwrap();

        assert_eq!(
            config.determine_tier(&instrument, quantity),
            Some(ReportingTier::Standard)
        );
    }

    #[test]
    fn eth_5x_threshold_is_large_tier() {
        let config = BlockTradeConfig::default();
        let instrument = create_eth_instrument();
        let quantity = Quantity::new(1250.0).unwrap();

        assert_eq!(
            config.determine_tier(&instrument, quantity),
            Some(ReportingTier::Large)
        );
    }

    #[test]
    fn eth_10x_threshold_is_very_large_tier() {
        let config = BlockTradeConfig::default();
        let instrument = create_eth_instrument();
        let quantity = Quantity::new(2500.0).unwrap();

        assert_eq!(
            config.determine_tier(&instrument, quantity),
            Some(ReportingTier::VeryLarge)
        );
    }

    #[test]
    fn below_threshold_returns_none_tier() {
        let config = BlockTradeConfig::default();
        let instrument = create_btc_instrument();
        let quantity = Quantity::new(24.9).unwrap();

        assert_eq!(config.determine_tier(&instrument, quantity), None);
    }

    #[test]
    fn custom_thresholds_work() {
        let mut config = BlockTradeConfig::default();
        config
            .thresholds
            .insert("SOL".to_string(), Quantity::new(1000.0).unwrap());

        let instrument = create_unknown_instrument();
        let quantity = Quantity::new(1000.0).unwrap();

        assert!(config.qualifies(&instrument, quantity));
    }

    #[test]
    fn custom_tier_multipliers_work() {
        let mut config = BlockTradeConfig::default();
        config.tier_multipliers.large = 2.0;
        config.tier_multipliers.very_large = 4.0;

        let instrument = create_btc_instrument();

        assert_eq!(
            config.determine_tier(&instrument, Quantity::new(50.0).unwrap()),
            Some(ReportingTier::Large)
        );
        assert_eq!(
            config.determine_tier(&instrument, Quantity::new(100.0).unwrap()),
            Some(ReportingTier::VeryLarge)
        );
    }

    #[test]
    fn get_threshold_returns_correct_value() {
        let config = BlockTradeConfig::default();

        assert_eq!(
            config.get_threshold(&create_btc_instrument()),
            Some(Quantity::new(25.0).unwrap())
        );
        assert_eq!(
            config.get_threshold(&create_eth_instrument()),
            Some(Quantity::new(250.0).unwrap())
        );
        assert_eq!(config.get_threshold(&create_unknown_instrument()), None);
    }

    #[test]
    fn reporting_tier_delay_minutes_are_correct() {
        assert_eq!(ReportingTier::Standard.delay_minutes(), 15);
        assert_eq!(ReportingTier::Large.delay_minutes(), 60);
        assert_eq!(ReportingTier::VeryLarge.delay_minutes(), 1440);
    }

    #[test]
    fn serialization_round_trip_preserves_config() {
        let config = BlockTradeConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BlockTradeConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.thresholds.len(), deserialized.thresholds.len());
        assert_eq!(
            config.thresholds.get("BTC"),
            deserialized.thresholds.get("BTC")
        );
        assert_eq!(
            config.thresholds.get("ETH"),
            deserialized.thresholds.get("ETH")
        );
        assert_eq!(config.default_threshold, deserialized.default_threshold);
        assert_eq!(
            config.tier_multipliers.large,
            deserialized.tier_multipliers.large
        );
        assert_eq!(
            config.tier_multipliers.very_large,
            deserialized.tier_multipliers.very_large
        );
    }

    #[test]
    fn serialization_round_trip_preserves_tier() {
        let tier = ReportingTier::Large;

        let json = serde_json::to_string(&tier).unwrap();
        let deserialized: ReportingTier = serde_json::from_str(&json).unwrap();

        assert_eq!(tier, deserialized);
    }

    #[test]
    fn zero_quantity_does_not_qualify() {
        let config = BlockTradeConfig::default();
        let instrument = create_btc_instrument();
        let quantity = Quantity::new(0.0).unwrap();

        assert!(!config.qualifies(&instrument, quantity));
    }
}
