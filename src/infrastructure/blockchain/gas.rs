//! # Gas Management
//!
//! Gas estimation and pricing strategies for Ethereum and L2 networks.
//!
//! Supports both legacy gas pricing and EIP-1559 dynamic fee transactions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Gas price configuration.
///
/// Supports both legacy gas pricing and EIP-1559 dynamic fees.
/// Gas prices are stored as u64 (wei), which is sufficient for practical gas prices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum GasPrice {
    /// Legacy gas price in wei.
    Legacy {
        /// Gas price in wei.
        gas_price: u64,
    },
    /// EIP-1559 dynamic fee.
    Eip1559 {
        /// Maximum fee per gas in wei.
        max_fee_per_gas: u64,
        /// Maximum priority fee per gas in wei.
        max_priority_fee_per_gas: u64,
    },
}

impl GasPrice {
    /// Creates a legacy gas price.
    #[must_use]
    pub const fn legacy(gas_price: u64) -> Self {
        Self::Legacy { gas_price }
    }

    /// Creates an EIP-1559 gas price.
    #[must_use]
    pub const fn eip1559(max_fee_per_gas: u64, max_priority_fee_per_gas: u64) -> Self {
        Self::Eip1559 {
            max_fee_per_gas,
            max_priority_fee_per_gas,
        }
    }

    /// Returns the effective gas price for cost estimation.
    ///
    /// For legacy transactions, returns the gas price.
    /// For EIP-1559, returns the max fee per gas.
    #[must_use]
    pub const fn effective_price(&self) -> u64 {
        match self {
            Self::Legacy { gas_price } => *gas_price,
            Self::Eip1559 {
                max_fee_per_gas, ..
            } => *max_fee_per_gas,
        }
    }

    /// Returns whether this is an EIP-1559 gas price.
    #[must_use]
    pub const fn is_eip1559(&self) -> bool {
        matches!(self, Self::Eip1559 { .. })
    }
}

impl fmt::Display for GasPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Legacy { gas_price } => write!(f, "legacy: {} wei", gas_price),
            Self::Eip1559 {
                max_fee_per_gas,
                max_priority_fee_per_gas,
            } => write!(
                f,
                "eip1559: max_fee={} wei, priority_fee={} wei",
                max_fee_per_gas, max_priority_fee_per_gas
            ),
        }
    }
}

/// Gas estimator with configurable buffer.
///
/// Applies a percentage buffer to gas estimates to account for
/// estimation inaccuracies and state changes.
#[derive(Debug, Clone)]
pub struct GasEstimator {
    /// Buffer percentage to add to gas estimates (e.g., 20 for 20%).
    buffer_percent: u64,
}

impl GasEstimator {
    /// Default gas buffer percentage.
    pub const DEFAULT_BUFFER_PERCENT: u64 = 20;

    /// Creates a new gas estimator with the specified buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer_percent` - Percentage to add to gas estimates (e.g., 20 for 20%)
    #[must_use]
    pub const fn new(buffer_percent: u64) -> Self {
        Self { buffer_percent }
    }

    /// Creates a gas estimator with the default buffer.
    #[must_use]
    pub const fn with_default_buffer() -> Self {
        Self::new(Self::DEFAULT_BUFFER_PERCENT)
    }

    /// Returns the buffer percentage.
    #[must_use]
    pub const fn buffer_percent(&self) -> u64 {
        self.buffer_percent
    }

    /// Applies the buffer to a gas estimate.
    ///
    /// # Arguments
    ///
    /// * `estimate` - The raw gas estimate
    ///
    /// # Returns
    ///
    /// The buffered gas estimate.
    #[must_use]
    pub const fn apply_buffer(&self, estimate: u64) -> u64 {
        estimate + (estimate * self.buffer_percent / 100)
    }

    /// Estimates the transaction cost in wei.
    ///
    /// # Arguments
    ///
    /// * `gas_limit` - The gas limit for the transaction
    /// * `gas_price` - The gas price configuration
    ///
    /// # Returns
    ///
    /// The estimated cost in wei.
    #[must_use]
    pub fn estimate_cost(&self, gas_limit: u64, gas_price: &GasPrice) -> u128 {
        gas_limit as u128 * gas_price.effective_price() as u128
    }
}

impl Default for GasEstimator {
    fn default() -> Self {
        Self::with_default_buffer()
    }
}

/// Fee history for EIP-1559 gas price calculation.
#[derive(Debug, Clone, Default)]
pub struct FeeHistory {
    /// Base fees per gas for recent blocks.
    pub base_fees: Vec<u64>,
    /// Priority fees at different percentiles.
    pub priority_fees: Vec<Vec<u64>>,
}

impl FeeHistory {
    /// Creates a new fee history.
    #[must_use]
    pub fn new(base_fees: Vec<u64>, priority_fees: Vec<Vec<u64>>) -> Self {
        Self {
            base_fees,
            priority_fees,
        }
    }

    /// Calculates the recommended max fee per gas.
    ///
    /// Uses the median base fee with a 2x multiplier for safety.
    #[must_use]
    pub fn recommended_max_fee(&self) -> u64 {
        if self.base_fees.is_empty() {
            return 0;
        }

        let mut sorted = self.base_fees.clone();
        sorted.sort_unstable();
        let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0);

        // 2x multiplier for safety margin
        median.saturating_mul(2)
    }

    /// Calculates the recommended priority fee for a given percentile index.
    ///
    /// # Arguments
    ///
    /// * `percentile_index` - Index into the priority_fees percentile array
    #[must_use]
    pub fn recommended_priority_fee(&self, percentile_index: usize) -> u64 {
        if self.priority_fees.is_empty() {
            return 0;
        }

        let fees: Vec<u64> = self
            .priority_fees
            .iter()
            .filter_map(|block_fees| block_fees.get(percentile_index).copied())
            .collect();

        if fees.is_empty() {
            return 0;
        }

        let mut sorted = fees;
        sorted.sort_unstable();
        sorted.get(sorted.len() / 2).copied().unwrap_or(0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn gas_price_legacy() {
        let price = GasPrice::legacy(25_000_000_000);
        assert_eq!(price.effective_price(), 25_000_000_000);
        assert!(!price.is_eip1559());
    }

    #[test]
    fn gas_price_eip1559() {
        let price = GasPrice::eip1559(50_000_000_000, 2_000_000_000);
        assert_eq!(price.effective_price(), 50_000_000_000);
        assert!(price.is_eip1559());
    }

    #[test]
    fn gas_price_display() {
        let legacy = GasPrice::legacy(25_000_000_000);
        assert!(legacy.to_string().contains("legacy"));

        let eip1559 = GasPrice::eip1559(50_000_000_000, 2_000_000_000);
        assert!(eip1559.to_string().contains("eip1559"));
    }

    #[test]
    fn gas_estimator_apply_buffer() {
        let estimator = GasEstimator::new(20);
        assert_eq!(estimator.apply_buffer(100_000), 120_000);
        assert_eq!(estimator.apply_buffer(200_000), 240_000);
    }

    #[test]
    fn gas_estimator_default_buffer() {
        let estimator = GasEstimator::default();
        assert_eq!(estimator.buffer_percent(), 20);
    }

    #[test]
    fn gas_estimator_estimate_cost() {
        let estimator = GasEstimator::default();
        let gas_price = GasPrice::legacy(25_000_000_000);
        let cost = estimator.estimate_cost(21_000, &gas_price);
        assert_eq!(cost, 21_000 * 25_000_000_000);
    }

    #[test]
    fn fee_history_recommended_max_fee() {
        let history = FeeHistory::new(vec![10_000_000_000, 12_000_000_000, 11_000_000_000], vec![]);
        let max_fee = history.recommended_max_fee();
        // Median is 11_000_000_000, 2x = 22_000_000_000
        assert_eq!(max_fee, 22_000_000_000);
    }

    #[test]
    fn fee_history_recommended_priority_fee() {
        let history = FeeHistory::new(
            vec![],
            vec![
                vec![1_000_000_000, 2_000_000_000, 3_000_000_000],
                vec![1_500_000_000, 2_500_000_000, 3_500_000_000],
                vec![1_200_000_000, 2_200_000_000, 3_200_000_000],
            ],
        );
        // Percentile index 1 (medium): [2_000_000_000, 2_500_000_000, 2_200_000_000]
        // Median is 2_200_000_000
        let priority_fee = history.recommended_priority_fee(1);
        assert_eq!(priority_fee, 2_200_000_000);
    }

    #[test]
    fn gas_price_serde_roundtrip() {
        let legacy = GasPrice::legacy(25_000_000_000);
        let json = serde_json::to_string(&legacy).unwrap();
        let deserialized: GasPrice = serde_json::from_str(&json).unwrap();
        assert_eq!(legacy, deserialized);

        let eip1559 = GasPrice::eip1559(50_000_000_000, 2_000_000_000);
        let json = serde_json::to_string(&eip1559).unwrap();
        let deserialized: GasPrice = serde_json::from_str(&json).unwrap();
        assert_eq!(eip1559, deserialized);
    }
}
