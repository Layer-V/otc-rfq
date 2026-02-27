//! # Price Bounds Validation
//!
//! Service for validating proposed block trade prices against reference prices.
//!
//! This module provides:
//! - [`ReferencePriceProvider`]: Async trait for obtaining reference prices
//! - [`FallbackReferencePriceProvider`]: Chains multiple providers with fallback
//! - [`PriceBoundsValidator`]: Validates proposed prices are within tolerance
//!
//! # Fallback Chain
//!
//! ```text
//! CLOB Mid → Theoretical → Chainlink Index
//! ```
//!
//! The first provider that returns a price is used. If none return a price,
//! `DomainError::NoReferencePrice` is returned.
//!
//! # Examples
//!
//! ```
//! use otc_rfq::application::services::price_bounds::{
//!     PriceBoundsValidator, ReferencePriceProvider,
//! };
//! use otc_rfq::domain::value_objects::{
//!     Price, PriceBoundsConfig, LiquidityClassification, Instrument,
//!     ReferencePriceSource,
//! };
//! ```

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::arithmetic::CheckedArithmetic;
use crate::domain::value_objects::instrument::Instrument;
use crate::domain::value_objects::liquidity_classification::LiquidityClassification;
use crate::domain::value_objects::price::Price;
use crate::domain::value_objects::reference_price::{
    PriceBoundsConfig, PriceBoundsResult, ReferencePriceSource,
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

/// Trait for obtaining reference prices for instruments.
///
/// Implementors fetch a reference price from a specific source
/// (e.g., CLOB mid-price, theoretical model, Chainlink oracle).
///
/// # Examples
///
/// ```ignore
/// let (price, source) = provider.get_reference(&instrument).await?.unwrap();
/// ```
#[async_trait]
pub trait ReferencePriceProvider: Send + Sync {
    /// Returns the reference price and its source for the given instrument.
    ///
    /// Returns `Ok(None)` if this provider has no price available for
    /// the instrument. Returns `Err` only on infrastructure failures.
    ///
    /// # Arguments
    ///
    /// * `instrument` - The instrument to look up
    ///
    /// # Errors
    ///
    /// Returns a `DomainError` if the provider encounters an infrastructure failure.
    async fn get_reference(
        &self,
        instrument: &Instrument,
    ) -> DomainResult<Option<(Price, ReferencePriceSource)>>;
}

/// A reference price provider that tries multiple providers in order.
///
/// Returns the first successful result. If all providers return `None`,
/// the composite also returns `None`.
///
/// # Examples
///
/// ```
/// use otc_rfq::application::services::price_bounds::FallbackReferencePriceProvider;
///
/// // Providers are tried in the order they are added.
/// let provider = FallbackReferencePriceProvider::new(vec![]);
/// ```
pub struct FallbackReferencePriceProvider {
    providers: Vec<Arc<dyn ReferencePriceProvider>>,
}

impl FallbackReferencePriceProvider {
    /// Creates a new fallback provider from an ordered list of providers.
    ///
    /// Providers are tried in the order given. The first one returning
    /// `Some(price, source)` wins.
    #[must_use]
    pub fn new(providers: Vec<Arc<dyn ReferencePriceProvider>>) -> Self {
        Self { providers }
    }
}

#[async_trait]
impl ReferencePriceProvider for FallbackReferencePriceProvider {
    async fn get_reference(
        &self,
        instrument: &Instrument,
    ) -> DomainResult<Option<(Price, ReferencePriceSource)>> {
        for provider in &self.providers {
            match provider.get_reference(instrument).await {
                Ok(Some(result)) => return Ok(Some(result)),
                Ok(None) => continue,
                Err(e) => {
                    // Log and continue to next provider on infrastructure errors.
                    tracing::warn!(
                        error = %e,
                        "reference price provider failed, trying next"
                    );
                    continue;
                }
            }
        }
        Ok(None)
    }
}

/// Validates that a proposed price is within acceptable bounds of a reference price.
///
/// Uses a [`ReferencePriceProvider`] to obtain the reference price and a
/// [`PriceBoundsConfig`] to determine the allowed tolerance based on the
/// instrument's [`LiquidityClassification`].
///
/// # Deviation Calculation
///
/// ```text
/// deviation = abs(proposed - reference) / reference
/// ```
///
/// The deviation is compared against the tolerance for the given liquidity tier.
/// If `deviation > tolerance`, `DomainError::PriceOutOfBounds` is returned.
pub struct PriceBoundsValidator {
    config: PriceBoundsConfig,
    reference_provider: Arc<dyn ReferencePriceProvider>,
}

impl PriceBoundsValidator {
    /// Creates a new price bounds validator.
    ///
    /// # Arguments
    ///
    /// * `config` - Tolerance percentages per liquidity tier
    /// * `reference_provider` - Provider for reference prices (may be a fallback chain)
    #[must_use]
    pub fn new(
        config: PriceBoundsConfig,
        reference_provider: Arc<dyn ReferencePriceProvider>,
    ) -> Self {
        Self {
            config,
            reference_provider,
        }
    }

    /// Validates a proposed price against reference prices.
    ///
    /// # Arguments
    ///
    /// * `instrument` - The instrument being traded
    /// * `proposed_price` - The proposed block trade price
    /// * `liquidity` - The liquidity classification of the instrument
    ///
    /// # Returns
    ///
    /// `Ok(PriceBoundsResult)` if the price is within bounds.
    ///
    /// # Errors
    ///
    /// - `DomainError::NoReferencePrice` if no reference price is available
    /// - `DomainError::PriceOutOfBounds` if the deviation exceeds the tolerance
    /// - `DomainError::DivisionByZero` if the reference price is zero
    /// - Arithmetic errors on overflow
    pub async fn validate(
        &self,
        instrument: &Instrument,
        proposed_price: &Price,
        liquidity: LiquidityClassification,
    ) -> DomainResult<PriceBoundsResult> {
        let (reference, source) = self
            .reference_provider
            .get_reference(instrument)
            .await?
            .ok_or(DomainError::NoReferencePrice)?;

        let tolerance = self.config.tolerance_for(liquidity);

        let deviation = compute_deviation(proposed_price, &reference)?;

        if deviation > tolerance {
            return Err(DomainError::PriceOutOfBounds {
                proposed: *proposed_price,
                reference,
                deviation_pct: deviation,
                max_tolerance_pct: tolerance,
            });
        }

        Ok(PriceBoundsResult::new(reference, source, deviation))
    }

    /// Returns the current configuration.
    #[inline]
    #[must_use]
    pub const fn config(&self) -> &PriceBoundsConfig {
        &self.config
    }
}

/// Computes the absolute fractional deviation between proposed and reference prices.
///
/// ```text
/// deviation = abs(proposed - reference) / reference
/// ```
///
/// # Errors
///
/// Returns `DomainError::DivisionByZero` if reference price is zero.
/// Returns arithmetic errors on overflow.
fn compute_deviation(proposed: &Price, reference: &Price) -> DomainResult<Decimal> {
    let ref_decimal = reference.get();
    if ref_decimal.is_zero() {
        return Err(DomainError::DivisionByZero);
    }

    let proposed_decimal = proposed.get();
    // abs(proposed - reference) — use checked subtraction, then abs
    let diff = proposed_decimal.safe_sub(ref_decimal)?;
    let abs_diff = diff.abs();
    let deviation = abs_diff.safe_div(ref_decimal)?;
    Ok(deviation)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::domain::value_objects::enums::{AssetClass, SettlementMethod};
    use crate::domain::value_objects::symbol::Symbol;
    use rust_decimal::Decimal;

    fn test_instrument() -> Instrument {
        Instrument::new(
            Symbol::new("BTC/USD").unwrap(),
            AssetClass::CryptoSpot,
            SettlementMethod::default(),
        )
    }

    /// A mock provider that returns a fixed price and source.
    struct FixedPriceProvider {
        price: Option<(Price, ReferencePriceSource)>,
    }

    impl FixedPriceProvider {
        fn some(price: f64, source: ReferencePriceSource) -> Self {
            Self {
                price: Some((Price::new(price).unwrap(), source)),
            }
        }

        fn none() -> Self {
            Self { price: None }
        }
    }

    #[async_trait]
    impl ReferencePriceProvider for FixedPriceProvider {
        async fn get_reference(
            &self,
            _instrument: &Instrument,
        ) -> DomainResult<Option<(Price, ReferencePriceSource)>> {
            Ok(self.price)
        }
    }

    /// A mock provider that always returns an error.
    struct FailingProvider;

    #[async_trait]
    impl ReferencePriceProvider for FailingProvider {
        async fn get_reference(
            &self,
            _instrument: &Instrument,
        ) -> DomainResult<Option<(Price, ReferencePriceSource)>> {
            Err(DomainError::ValidationError("provider failure".to_string()))
        }
    }

    mod compute_deviation_tests {
        use super::*;

        #[test]
        fn zero_deviation() {
            let price = Price::new(100.0).unwrap();
            let deviation = compute_deviation(&price, &price).unwrap();
            assert_eq!(deviation, Decimal::ZERO);
        }

        #[test]
        fn positive_deviation() {
            let proposed = Price::new(105.0).unwrap();
            let reference = Price::new(100.0).unwrap();
            let deviation = compute_deviation(&proposed, &reference).unwrap();
            assert_eq!(deviation, Decimal::new(5, 2));
        }

        #[test]
        fn negative_deviation_is_absolute() {
            let proposed = Price::new(95.0).unwrap();
            let reference = Price::new(100.0).unwrap();
            let deviation = compute_deviation(&proposed, &reference).unwrap();
            assert_eq!(deviation, Decimal::new(5, 2));
        }

        #[test]
        fn division_by_zero_reference() {
            let proposed = Price::new(100.0).unwrap();
            let reference = Price::zero();
            let result = compute_deviation(&proposed, &reference);
            assert!(matches!(result, Err(DomainError::DivisionByZero)));
        }

        #[test]
        fn large_deviation() {
            let proposed = Price::new(200.0).unwrap();
            let reference = Price::new(100.0).unwrap();
            let deviation = compute_deviation(&proposed, &reference).unwrap();
            assert_eq!(deviation, Decimal::ONE);
        }
    }

    mod fallback_provider {
        use super::*;

        #[tokio::test]
        async fn returns_first_available() {
            let provider = FallbackReferencePriceProvider::new(vec![
                Arc::new(FixedPriceProvider::some(
                    50000.0,
                    ReferencePriceSource::ClobMid,
                )),
                Arc::new(FixedPriceProvider::some(
                    49000.0,
                    ReferencePriceSource::Theoretical,
                )),
            ]);

            let result = provider
                .get_reference(&test_instrument())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.1, ReferencePriceSource::ClobMid);
        }

        #[tokio::test]
        async fn falls_back_when_first_returns_none() {
            let provider = FallbackReferencePriceProvider::new(vec![
                Arc::new(FixedPriceProvider::none()),
                Arc::new(FixedPriceProvider::some(
                    49000.0,
                    ReferencePriceSource::Theoretical,
                )),
            ]);

            let result = provider
                .get_reference(&test_instrument())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.1, ReferencePriceSource::Theoretical);
        }

        #[tokio::test]
        async fn falls_back_when_first_errors() {
            let provider = FallbackReferencePriceProvider::new(vec![
                Arc::new(FailingProvider),
                Arc::new(FixedPriceProvider::some(
                    48000.0,
                    ReferencePriceSource::ChainlinkIndex,
                )),
            ]);

            let result = provider
                .get_reference(&test_instrument())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.1, ReferencePriceSource::ChainlinkIndex);
        }

        #[tokio::test]
        async fn returns_none_when_all_return_none() {
            let provider = FallbackReferencePriceProvider::new(vec![
                Arc::new(FixedPriceProvider::none()),
                Arc::new(FixedPriceProvider::none()),
            ]);

            let result = provider.get_reference(&test_instrument()).await.unwrap();
            assert!(result.is_none());
        }

        #[tokio::test]
        async fn returns_none_when_empty() {
            let provider = FallbackReferencePriceProvider::new(vec![]);

            let result = provider.get_reference(&test_instrument()).await.unwrap();
            assert!(result.is_none());
        }

        #[tokio::test]
        async fn skips_error_and_none_to_find_third() {
            let provider = FallbackReferencePriceProvider::new(vec![
                Arc::new(FailingProvider),
                Arc::new(FixedPriceProvider::none()),
                Arc::new(FixedPriceProvider::some(
                    47000.0,
                    ReferencePriceSource::ChainlinkIndex,
                )),
            ]);

            let result = provider
                .get_reference(&test_instrument())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.0, Price::new(47000.0).unwrap());
            assert_eq!(result.1, ReferencePriceSource::ChainlinkIndex);
        }
    }

    mod validator {
        use super::*;

        fn make_validator(
            reference_price: f64,
            source: ReferencePriceSource,
        ) -> PriceBoundsValidator {
            PriceBoundsValidator::new(
                PriceBoundsConfig::default(),
                Arc::new(FixedPriceProvider::some(reference_price, source)),
            )
        }

        fn make_validator_no_ref() -> PriceBoundsValidator {
            PriceBoundsValidator::new(
                PriceBoundsConfig::default(),
                Arc::new(FixedPriceProvider::none()),
            )
        }

        // ---- Liquid instruments (±5%) ----

        #[tokio::test]
        async fn liquid_exact_boundary_passes() {
            let validator = make_validator(100.0, ReferencePriceSource::ClobMid);
            // Exactly +5%
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(105.0).unwrap(),
                    LiquidityClassification::Liquid,
                )
                .await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().deviation_pct(), Decimal::new(5, 2));
        }

        #[tokio::test]
        async fn liquid_exact_negative_boundary_passes() {
            let validator = make_validator(100.0, ReferencePriceSource::ClobMid);
            // Exactly -5%
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(95.0).unwrap(),
                    LiquidityClassification::Liquid,
                )
                .await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().deviation_pct(), Decimal::new(5, 2));
        }

        #[tokio::test]
        async fn liquid_just_under_boundary_passes() {
            let validator = make_validator(100.0, ReferencePriceSource::ClobMid);
            // 4.99% — within tolerance
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(104.99).unwrap(),
                    LiquidityClassification::Liquid,
                )
                .await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn liquid_just_over_boundary_fails() {
            let validator = make_validator(100.0, ReferencePriceSource::ClobMid);
            // 5.01% — exceeds tolerance
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(105.01).unwrap(),
                    LiquidityClassification::Liquid,
                )
                .await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, DomainError::PriceOutOfBounds { .. }));
            if let DomainError::PriceOutOfBounds {
                proposed,
                reference,
                max_tolerance_pct,
                ..
            } = err
            {
                assert_eq!(proposed, Price::new(105.01).unwrap());
                assert_eq!(reference, Price::new(100.0).unwrap());
                assert_eq!(max_tolerance_pct, Decimal::new(5, 2));
            }
        }

        #[tokio::test]
        async fn liquid_zero_deviation_passes() {
            let validator = make_validator(100.0, ReferencePriceSource::ClobMid);
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(100.0).unwrap(),
                    LiquidityClassification::Liquid,
                )
                .await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().deviation_pct(), Decimal::ZERO);
        }

        // ---- Semi-liquid instruments (±7.5%) ----

        #[tokio::test]
        async fn semi_liquid_at_boundary_passes() {
            let validator = make_validator(100.0, ReferencePriceSource::Theoretical);
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(107.5).unwrap(),
                    LiquidityClassification::SemiLiquid,
                )
                .await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().deviation_pct(), Decimal::new(75, 3));
        }

        #[tokio::test]
        async fn semi_liquid_over_boundary_fails() {
            let validator = make_validator(100.0, ReferencePriceSource::Theoretical);
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(107.51).unwrap(),
                    LiquidityClassification::SemiLiquid,
                )
                .await;
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                DomainError::PriceOutOfBounds { .. }
            ));
        }

        // ---- Illiquid instruments (±10%) ----

        #[tokio::test]
        async fn illiquid_at_boundary_passes() {
            let validator = make_validator(100.0, ReferencePriceSource::ChainlinkIndex);
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(110.0).unwrap(),
                    LiquidityClassification::Illiquid,
                )
                .await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().deviation_pct(), Decimal::new(1, 1));
        }

        #[tokio::test]
        async fn illiquid_over_boundary_fails() {
            let validator = make_validator(100.0, ReferencePriceSource::ChainlinkIndex);
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(110.01).unwrap(),
                    LiquidityClassification::Illiquid,
                )
                .await;
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                DomainError::PriceOutOfBounds { .. }
            ));
        }

        #[tokio::test]
        async fn illiquid_negative_at_boundary_passes() {
            let validator = make_validator(100.0, ReferencePriceSource::ChainlinkIndex);
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(90.0).unwrap(),
                    LiquidityClassification::Illiquid,
                )
                .await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap().deviation_pct(), Decimal::new(1, 1));
        }

        // ---- Error cases ----

        #[tokio::test]
        async fn no_reference_price_returns_error() {
            let validator = make_validator_no_ref();
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(100.0).unwrap(),
                    LiquidityClassification::Liquid,
                )
                .await;
            assert!(matches!(result, Err(DomainError::NoReferencePrice)));
        }

        // ---- Result metadata ----

        #[tokio::test]
        async fn result_contains_correct_source() {
            let validator = make_validator(50000.0, ReferencePriceSource::Theoretical);
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(50000.0).unwrap(),
                    LiquidityClassification::Liquid,
                )
                .await
                .unwrap();
            assert_eq!(result.source(), ReferencePriceSource::Theoretical);
            assert_eq!(result.reference(), Price::new(50000.0).unwrap());
        }

        #[tokio::test]
        async fn config_accessor() {
            let config = PriceBoundsConfig::default();
            let validator = PriceBoundsValidator::new(config, Arc::new(FixedPriceProvider::none()));
            assert_eq!(
                validator.config().liquid_tolerance_pct(),
                Decimal::new(5, 2),
            );
        }

        // ---- Custom config ----

        #[tokio::test]
        async fn custom_config_tighter_tolerance() {
            let config =
                PriceBoundsConfig::new(Decimal::new(1, 2), Decimal::new(2, 2), Decimal::new(3, 2))
                    .unwrap();
            let validator = PriceBoundsValidator::new(
                config,
                Arc::new(FixedPriceProvider::some(
                    100.0,
                    ReferencePriceSource::ClobMid,
                )),
            );

            // 2% deviation with 1% tolerance → fail
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(102.0).unwrap(),
                    LiquidityClassification::Liquid,
                )
                .await;
            assert!(matches!(result, Err(DomainError::PriceOutOfBounds { .. })));

            // 2% deviation with 3% tolerance (illiquid) → pass
            let result = validator
                .validate(
                    &test_instrument(),
                    &Price::new(102.0).unwrap(),
                    LiquidityClassification::Illiquid,
                )
                .await;
            assert!(result.is_ok());
        }
    }
}
