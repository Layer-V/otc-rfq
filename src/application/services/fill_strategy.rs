//! # Multi-MM Fill Strategy
//!
//! Strategies for allocating an RFQ's target quantity across multiple
//! market maker quotes.
//!
//! This module provides the [`MultiMmFillStrategy`] trait and two
//! implementations:
//!
//! - [`ProRataStrategy`]: Distributes proportionally by quoted quantity
//! - [`BestPriceFillStrategy`]: Fills from the best price first, cascading
//!
//! # Examples
//!
//! ```
//! use otc_rfq::application::services::fill_strategy::{
//!     BestPriceFillStrategy, MultiMmFillStrategy, ProRataStrategy,
//! };
//! use otc_rfq::application::services::ranking_strategy::RankedQuote;
//! use otc_rfq::domain::entities::quote::QuoteBuilder;
//! use otc_rfq::domain::value_objects::{
//!     OrderSide, Price, Quantity, RfqId, Timestamp, VenueId,
//! };
//! use otc_rfq::domain::value_objects::size_negotiation_mode::SizeNegotiationMode;
//!
//! let rfq_id = RfqId::new_v4();
//! let quote = QuoteBuilder::new(
//!     rfq_id,
//!     VenueId::new("venue-1"),
//!     Price::new(50000.0).unwrap(),
//!     Quantity::new(2.0).unwrap(),
//!     Timestamp::now().add_secs(300),
//! ).build();
//!
//! let ranked = vec![RankedQuote::new(quote, 1, 1.0)];
//! let strategy = ProRataStrategy;
//! let target = Quantity::new(1.0).unwrap();
//! let mode = SizeNegotiationMode::BestEffort;
//!
//! let allocations = strategy.allocate(&ranked, target, &mode, OrderSide::Buy).unwrap();
//! assert_eq!(allocations.len(), 1);
//! ```

use crate::application::services::ranking_strategy::RankedQuote;
use crate::domain::entities::allocation::Allocation;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::size_negotiation_mode::SizeNegotiationMode;
use crate::domain::value_objects::{OrderSide, Quantity};
use std::fmt;

/// Trait for multi-MM fill allocation strategies.
///
/// Implementations define how an RFQ's target quantity is distributed
/// across one or more ranked quotes from different market makers.
///
/// # Contract
///
/// - If the mode requires a full fill (`AllOrNothing`, `FillOrKill`)
///   and the total quoted quantity is insufficient, the implementation
///   must return `DomainError::InsufficientLiquidity`.
/// - If the mode specifies a `MinQuantity` and the total allocatable
///   quantity is below the threshold, the implementation must return
///   `DomainError::MinQuantityNotMet`.
/// - The sum of all allocated quantities must equal the target quantity
///   (or the available quantity for `BestEffort`).
pub trait MultiMmFillStrategy: Send + Sync + fmt::Debug {
    /// Allocates the target quantity across the given ranked quotes.
    ///
    /// # Arguments
    ///
    /// * `quotes` - Ranked quotes sorted best-first
    /// * `target_qty` - The total quantity to fill
    /// * `mode` - The size negotiation semantics to enforce
    /// * `side` - The order side (Buy or Sell) for price ordering context
    ///
    /// # Errors
    ///
    /// - `DomainError::InsufficientLiquidity` if full-fill modes cannot be satisfied
    /// - `DomainError::MinQuantityNotMet` if the fill is below the minimum threshold
    /// - `DomainError::InvalidQuantity` if target quantity is zero or quotes are empty
    fn allocate(
        &self,
        quotes: &[RankedQuote],
        target_qty: Quantity,
        mode: &SizeNegotiationMode,
        side: OrderSide,
    ) -> DomainResult<Vec<Allocation>>;

    /// Returns the name of this fill strategy.
    fn name(&self) -> &'static str;
}

/// Validates common preconditions for allocation strategies.
///
/// # Errors
///
/// Returns `DomainError::InvalidQuantity` if target quantity is zero or quotes are empty.
fn validate_preconditions(quotes: &[RankedQuote], target_qty: Quantity) -> DomainResult<()> {
    if !target_qty.is_positive() {
        return Err(DomainError::InvalidQuantity(
            "target quantity must be positive".to_string(),
        ));
    }
    if quotes.is_empty() {
        return Err(DomainError::InvalidQuantity(
            "no quotes available for allocation".to_string(),
        ));
    }
    Ok(())
}

/// Computes the total quoted quantity across all ranked quotes.
fn total_quoted_quantity(quotes: &[RankedQuote]) -> DomainResult<Quantity> {
    let mut total = Quantity::zero();
    for rq in quotes {
        total = total.safe_add(rq.quote.quantity())?;
    }
    Ok(total)
}

/// Enforces mode-specific constraints on the fillable quantity.
///
/// Returns the effective quantity to allocate.
///
/// # Errors
///
/// Returns `DomainError::InsufficientLiquidity` or `DomainError::MinQuantityNotMet`
/// if the mode's constraints are violated.
fn enforce_mode(
    mode: &SizeNegotiationMode,
    target_qty: Quantity,
    available_qty: Quantity,
) -> DomainResult<Quantity> {
    match mode {
        SizeNegotiationMode::AllOrNothing | SizeNegotiationMode::FillOrKill => {
            if available_qty.get() < target_qty.get() {
                return Err(DomainError::InsufficientLiquidity {
                    available: available_qty,
                    requested: target_qty,
                });
            }
            Ok(target_qty)
        }
        SizeNegotiationMode::MinQuantity(min) => {
            let fillable = target_qty.min(available_qty);
            if fillable.get() < min.get() {
                return Err(DomainError::MinQuantityNotMet {
                    filled: fillable,
                    minimum: *min,
                });
            }
            Ok(fillable)
        }
        SizeNegotiationMode::BestEffort => {
            if available_qty.is_zero() {
                return Err(DomainError::InsufficientLiquidity {
                    available: available_qty,
                    requested: target_qty,
                });
            }
            Ok(target_qty.min(available_qty))
        }
    }
}

/// Validates that the sum of allocations matches the expected fill quantity.
///
/// # Errors
///
/// Returns `DomainError::AllocationMismatch` if the sum does not match.
fn validate_allocation_sum(allocations: &[Allocation], expected: Quantity) -> DomainResult<()> {
    let mut total = Quantity::zero();
    for alloc in allocations {
        total = total.safe_add(alloc.allocated_quantity())?;
    }
    if total.get() != expected.get() {
        return Err(DomainError::AllocationMismatch {
            allocated: total,
            target: expected,
        });
    }
    Ok(())
}

// ============================================================================
// Pro-rata Strategy
// ============================================================================

/// Distributes the target quantity proportionally across quotes.
///
/// Each quote receives a share proportional to its quoted quantity
/// relative to the total available quantity.
///
/// # Examples
///
/// ```
/// use otc_rfq::application::services::fill_strategy::{MultiMmFillStrategy, ProRataStrategy};
/// use otc_rfq::application::services::ranking_strategy::RankedQuote;
/// use otc_rfq::domain::entities::quote::QuoteBuilder;
/// use otc_rfq::domain::value_objects::{
///     OrderSide, Price, Quantity, RfqId, Timestamp, VenueId,
/// };
/// use otc_rfq::domain::value_objects::size_negotiation_mode::SizeNegotiationMode;
///
/// let rfq_id = RfqId::new_v4();
/// let q1 = QuoteBuilder::new(
///     rfq_id, VenueId::new("v1"),
///     Price::new(100.0).unwrap(), Quantity::new(6.0).unwrap(),
///     Timestamp::now().add_secs(300),
/// ).build();
/// let q2 = QuoteBuilder::new(
///     rfq_id, VenueId::new("v2"),
///     Price::new(101.0).unwrap(), Quantity::new(4.0).unwrap(),
///     Timestamp::now().add_secs(300),
/// ).build();
///
/// let ranked = vec![
///     RankedQuote::new(q1, 1, 1.0),
///     RankedQuote::new(q2, 2, 0.9),
/// ];
///
/// let allocs = ProRataStrategy
///     .allocate(&ranked, Quantity::new(5.0).unwrap(), &SizeNegotiationMode::BestEffort, OrderSide::Buy)
///     .unwrap();
///
/// assert_eq!(allocs.len(), 2);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ProRataStrategy;

impl ProRataStrategy {
    /// Creates a new pro-rata strategy.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl MultiMmFillStrategy for ProRataStrategy {
    fn allocate(
        &self,
        quotes: &[RankedQuote],
        target_qty: Quantity,
        mode: &SizeNegotiationMode,
        _side: OrderSide,
    ) -> DomainResult<Vec<Allocation>> {
        validate_preconditions(quotes, target_qty)?;

        let available = total_quoted_quantity(quotes)?;
        let effective_qty = enforce_mode(mode, target_qty, available)?;

        // Compute total quoted quantity for ratio calculation (Decimal arithmetic)
        let total_decimal = available.get();
        if total_decimal.is_zero() {
            return Err(DomainError::InvalidQuantity(
                "total quoted quantity is zero".to_string(),
            ));
        }

        let mut allocations = Vec::with_capacity(quotes.len());
        let mut allocated_so_far = Quantity::zero();

        for (i, rq) in quotes.iter().enumerate() {
            let is_last = i == quotes.len().saturating_sub(1);

            let alloc_qty = if is_last {
                // Last allocation gets the remainder to avoid rounding drift
                effective_qty.safe_sub(allocated_so_far)?
            } else {
                // ratio = quote_qty / total, then alloc = effective * ratio
                let raw_qty = effective_qty
                    .safe_mul(rq.quote.quantity().get())?
                    .safe_div(total_decimal)?;
                // Cap at this quote's available quantity
                raw_qty.min(rq.quote.quantity())
            };

            if alloc_qty.is_positive() {
                allocated_so_far = allocated_so_far.safe_add(alloc_qty)?;
                allocations.push(Allocation::new(
                    rq.quote.venue_id().clone(),
                    rq.quote.id(),
                    alloc_qty,
                    rq.quote.price(),
                )?);
            }
        }

        validate_allocation_sum(&allocations, effective_qty)?;
        Ok(allocations)
    }

    fn name(&self) -> &'static str {
        "ProRata"
    }
}

// ============================================================================
// Best-Price Fill Strategy
// ============================================================================

/// Fills from the best-priced quote first, cascading to the next.
///
/// Quotes are assumed to be sorted best-first (by rank). The strategy
/// fills as much as possible from the first quote, then moves to the
/// next, until the target quantity is met.
///
/// # Examples
///
/// ```
/// use otc_rfq::application::services::fill_strategy::{
///     BestPriceFillStrategy, MultiMmFillStrategy,
/// };
/// use otc_rfq::application::services::ranking_strategy::RankedQuote;
/// use otc_rfq::domain::entities::quote::QuoteBuilder;
/// use otc_rfq::domain::value_objects::{
///     OrderSide, Price, Quantity, RfqId, Timestamp, VenueId,
/// };
/// use otc_rfq::domain::value_objects::size_negotiation_mode::SizeNegotiationMode;
///
/// let rfq_id = RfqId::new_v4();
/// let q1 = QuoteBuilder::new(
///     rfq_id, VenueId::new("best"),
///     Price::new(99.0).unwrap(), Quantity::new(3.0).unwrap(),
///     Timestamp::now().add_secs(300),
/// ).build();
/// let q2 = QuoteBuilder::new(
///     rfq_id, VenueId::new("next"),
///     Price::new(100.0).unwrap(), Quantity::new(5.0).unwrap(),
///     Timestamp::now().add_secs(300),
/// ).build();
///
/// let ranked = vec![
///     RankedQuote::new(q1, 1, 1.0),
///     RankedQuote::new(q2, 2, 0.9),
/// ];
///
/// let allocs = BestPriceFillStrategy
///     .allocate(&ranked, Quantity::new(5.0).unwrap(), &SizeNegotiationMode::AllOrNothing, OrderSide::Buy)
///     .unwrap();
///
/// // First quote fills 3, second fills 2
/// assert_eq!(allocs.len(), 2);
/// ```
#[derive(Debug, Clone, Default)]
pub struct BestPriceFillStrategy;

impl BestPriceFillStrategy {
    /// Creates a new best-price fill strategy.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl MultiMmFillStrategy for BestPriceFillStrategy {
    fn allocate(
        &self,
        quotes: &[RankedQuote],
        target_qty: Quantity,
        mode: &SizeNegotiationMode,
        _side: OrderSide,
    ) -> DomainResult<Vec<Allocation>> {
        validate_preconditions(quotes, target_qty)?;

        let available = total_quoted_quantity(quotes)?;
        let effective_qty = enforce_mode(mode, target_qty, available)?;

        let mut remaining = effective_qty;
        let mut allocations = Vec::with_capacity(quotes.len());

        for rq in quotes {
            if remaining.is_zero() {
                break;
            }

            let fill_qty = rq.quote.quantity().min(remaining);
            if fill_qty.is_positive() {
                remaining = remaining.safe_sub(fill_qty)?;
                allocations.push(Allocation::new(
                    rq.quote.venue_id().clone(),
                    rq.quote.id(),
                    fill_qty,
                    rq.quote.price(),
                )?);
            }
        }

        validate_allocation_sum(&allocations, effective_qty)?;
        Ok(allocations)
    }

    fn name(&self) -> &'static str {
        "BestPrice"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::domain::entities::quote::QuoteBuilder;
    use crate::domain::value_objects::{Price, RfqId, Timestamp, VenueId};

    fn make_ranked_quote(
        rfq_id: RfqId,
        venue: &str,
        price: f64,
        qty: f64,
        rank: usize,
        score: f64,
    ) -> RankedQuote {
        let quote = QuoteBuilder::new(
            rfq_id,
            VenueId::new(venue),
            Price::new(price).unwrap(),
            Quantity::new(qty).unwrap(),
            Timestamp::now().add_secs(300),
        )
        .build();
        RankedQuote::new(quote, rank, score)
    }

    mod pro_rata_strategy {
        use super::*;

        #[test]
        fn single_quote_full_fill() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 10.0, 1, 1.0)];

            let allocs = ProRataStrategy
                .allocate(
                    &ranked,
                    Quantity::new(5.0).unwrap(),
                    &SizeNegotiationMode::AllOrNothing,
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 1);
            assert_eq!(allocs[0].allocated_quantity(), Quantity::new(5.0).unwrap());
            assert_eq!(allocs[0].venue_id().as_str(), "v1");
        }

        #[test]
        fn two_quotes_proportional() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![
                make_ranked_quote(rfq_id, "v1", 100.0, 6.0, 1, 1.0),
                make_ranked_quote(rfq_id, "v2", 101.0, 4.0, 2, 0.9),
            ];

            let allocs = ProRataStrategy
                .allocate(
                    &ranked,
                    Quantity::new(5.0).unwrap(),
                    &SizeNegotiationMode::BestEffort,
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 2);
            // v1 has 60% of total (6/10), so ~3.0
            // v2 gets the remainder ~2.0
            let total: Quantity = allocs
                .iter()
                .try_fold(Quantity::zero(), |acc, a| {
                    acc.safe_add(a.allocated_quantity())
                })
                .unwrap();
            assert_eq!(total, Quantity::new(5.0).unwrap());
        }

        #[test]
        fn insufficient_liquidity_all_or_nothing() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 3.0, 1, 1.0)];

            let result = ProRataStrategy.allocate(
                &ranked,
                Quantity::new(10.0).unwrap(),
                &SizeNegotiationMode::AllOrNothing,
                OrderSide::Buy,
            );

            assert!(matches!(
                result,
                Err(DomainError::InsufficientLiquidity { .. })
            ));
        }

        #[test]
        fn insufficient_liquidity_fill_or_kill() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 3.0, 1, 1.0)];

            let result = ProRataStrategy.allocate(
                &ranked,
                Quantity::new(10.0).unwrap(),
                &SizeNegotiationMode::FillOrKill,
                OrderSide::Buy,
            );

            assert!(matches!(
                result,
                Err(DomainError::InsufficientLiquidity { .. })
            ));
        }

        #[test]
        fn min_quantity_not_met() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 0.3, 1, 1.0)];

            let result = ProRataStrategy.allocate(
                &ranked,
                Quantity::new(10.0).unwrap(),
                &SizeNegotiationMode::MinQuantity(Quantity::new(1.0).unwrap()),
                OrderSide::Buy,
            );

            assert!(matches!(result, Err(DomainError::MinQuantityNotMet { .. })));
        }

        #[test]
        fn min_quantity_met() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 5.0, 1, 1.0)];

            let allocs = ProRataStrategy
                .allocate(
                    &ranked,
                    Quantity::new(10.0).unwrap(),
                    &SizeNegotiationMode::MinQuantity(Quantity::new(3.0).unwrap()),
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 1);
            assert_eq!(allocs[0].allocated_quantity(), Quantity::new(5.0).unwrap());
        }

        #[test]
        fn best_effort_partial_fill() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 3.0, 1, 1.0)];

            let allocs = ProRataStrategy
                .allocate(
                    &ranked,
                    Quantity::new(10.0).unwrap(),
                    &SizeNegotiationMode::BestEffort,
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 1);
            assert_eq!(allocs[0].allocated_quantity(), Quantity::new(3.0).unwrap());
        }

        #[test]
        fn empty_quotes_fails() {
            let result = ProRataStrategy.allocate(
                &[],
                Quantity::new(1.0).unwrap(),
                &SizeNegotiationMode::BestEffort,
                OrderSide::Buy,
            );

            assert!(matches!(result, Err(DomainError::InvalidQuantity(_))));
        }

        #[test]
        fn zero_target_fails() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 10.0, 1, 1.0)];

            let result = ProRataStrategy.allocate(
                &ranked,
                Quantity::zero(),
                &SizeNegotiationMode::BestEffort,
                OrderSide::Buy,
            );

            assert!(matches!(result, Err(DomainError::InvalidQuantity(_))));
        }

        #[test]
        fn name_returns_pro_rata() {
            assert_eq!(ProRataStrategy.name(), "ProRata");
        }
    }

    mod best_price_strategy {
        use super::*;

        #[test]
        fn single_quote_full_fill() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 99.0, 10.0, 1, 1.0)];

            let allocs = BestPriceFillStrategy
                .allocate(
                    &ranked,
                    Quantity::new(5.0).unwrap(),
                    &SizeNegotiationMode::AllOrNothing,
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 1);
            assert_eq!(allocs[0].allocated_quantity(), Quantity::new(5.0).unwrap());
            assert_eq!(allocs[0].price(), Price::new(99.0).unwrap());
        }

        #[test]
        fn cascading_fill_two_venues() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![
                make_ranked_quote(rfq_id, "best", 99.0, 3.0, 1, 1.0),
                make_ranked_quote(rfq_id, "next", 100.0, 5.0, 2, 0.9),
            ];

            let allocs = BestPriceFillStrategy
                .allocate(
                    &ranked,
                    Quantity::new(5.0).unwrap(),
                    &SizeNegotiationMode::AllOrNothing,
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 2);
            assert_eq!(allocs[0].allocated_quantity(), Quantity::new(3.0).unwrap());
            assert_eq!(allocs[0].venue_id().as_str(), "best");
            assert_eq!(allocs[1].allocated_quantity(), Quantity::new(2.0).unwrap());
            assert_eq!(allocs[1].venue_id().as_str(), "next");
        }

        #[test]
        fn cascading_fill_three_venues() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![
                make_ranked_quote(rfq_id, "v1", 98.0, 2.0, 1, 1.0),
                make_ranked_quote(rfq_id, "v2", 99.0, 2.0, 2, 0.9),
                make_ranked_quote(rfq_id, "v3", 100.0, 2.0, 3, 0.8),
            ];

            let allocs = BestPriceFillStrategy
                .allocate(
                    &ranked,
                    Quantity::new(5.0).unwrap(),
                    &SizeNegotiationMode::AllOrNothing,
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 3);
            assert_eq!(allocs[0].allocated_quantity(), Quantity::new(2.0).unwrap());
            assert_eq!(allocs[1].allocated_quantity(), Quantity::new(2.0).unwrap());
            assert_eq!(allocs[2].allocated_quantity(), Quantity::new(1.0).unwrap());
        }

        #[test]
        fn insufficient_liquidity_rejects() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 99.0, 3.0, 1, 1.0)];

            let result = BestPriceFillStrategy.allocate(
                &ranked,
                Quantity::new(10.0).unwrap(),
                &SizeNegotiationMode::AllOrNothing,
                OrderSide::Buy,
            );

            assert!(matches!(
                result,
                Err(DomainError::InsufficientLiquidity { .. })
            ));
        }

        #[test]
        fn best_effort_partial() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 99.0, 3.0, 1, 1.0)];

            let allocs = BestPriceFillStrategy
                .allocate(
                    &ranked,
                    Quantity::new(10.0).unwrap(),
                    &SizeNegotiationMode::BestEffort,
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 1);
            assert_eq!(allocs[0].allocated_quantity(), Quantity::new(3.0).unwrap());
        }

        #[test]
        fn sell_side_works() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![
                make_ranked_quote(rfq_id, "v1", 101.0, 3.0, 1, 1.0),
                make_ranked_quote(rfq_id, "v2", 100.0, 3.0, 2, 0.9),
            ];

            let allocs = BestPriceFillStrategy
                .allocate(
                    &ranked,
                    Quantity::new(4.0).unwrap(),
                    &SizeNegotiationMode::AllOrNothing,
                    OrderSide::Sell,
                )
                .unwrap();

            assert_eq!(allocs.len(), 2);
            assert_eq!(allocs[0].venue_id().as_str(), "v1");
        }

        #[test]
        fn empty_quotes_fails() {
            let result = BestPriceFillStrategy.allocate(
                &[],
                Quantity::new(1.0).unwrap(),
                &SizeNegotiationMode::BestEffort,
                OrderSide::Buy,
            );

            assert!(matches!(result, Err(DomainError::InvalidQuantity(_))));
        }

        #[test]
        fn zero_target_fails() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 99.0, 10.0, 1, 1.0)];

            let result = BestPriceFillStrategy.allocate(
                &ranked,
                Quantity::zero(),
                &SizeNegotiationMode::BestEffort,
                OrderSide::Buy,
            );

            assert!(matches!(result, Err(DomainError::InvalidQuantity(_))));
        }

        #[test]
        fn name_returns_best_price() {
            assert_eq!(BestPriceFillStrategy.name(), "BestPrice");
        }
    }

    mod mode_enforcement {
        use super::*;

        #[test]
        fn fill_or_kill_rejects_partial() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 5.0, 1, 1.0)];

            let result = BestPriceFillStrategy.allocate(
                &ranked,
                Quantity::new(10.0).unwrap(),
                &SizeNegotiationMode::FillOrKill,
                OrderSide::Buy,
            );

            assert!(matches!(
                result,
                Err(DomainError::InsufficientLiquidity { .. })
            ));
        }

        #[test]
        fn min_quantity_threshold_enforced() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 0.5, 1, 1.0)];

            let result = BestPriceFillStrategy.allocate(
                &ranked,
                Quantity::new(10.0).unwrap(),
                &SizeNegotiationMode::MinQuantity(Quantity::new(1.0).unwrap()),
                OrderSide::Buy,
            );

            assert!(matches!(result, Err(DomainError::MinQuantityNotMet { .. })));
        }

        #[test]
        fn min_quantity_threshold_met_allocates() {
            let rfq_id = RfqId::new_v4();
            let ranked = vec![make_ranked_quote(rfq_id, "v1", 100.0, 5.0, 1, 1.0)];

            let allocs = BestPriceFillStrategy
                .allocate(
                    &ranked,
                    Quantity::new(10.0).unwrap(),
                    &SizeNegotiationMode::MinQuantity(Quantity::new(3.0).unwrap()),
                    OrderSide::Buy,
                )
                .unwrap();

            assert_eq!(allocs.len(), 1);
            assert_eq!(allocs[0].allocated_quantity(), Quantity::new(5.0).unwrap());
        }

        #[test]
        fn best_effort_zero_available_returns_error() {
            let result = enforce_mode(
                &SizeNegotiationMode::BestEffort,
                Quantity::new(5.0).unwrap(),
                Quantity::zero(),
            );

            assert!(matches!(
                result,
                Err(DomainError::InsufficientLiquidity { .. })
            ));
        }
    }
}
