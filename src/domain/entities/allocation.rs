//! # Allocation Entity
//!
//! Represents a quantity allocation to a specific venue/quote in a multi-MM fill.
//!
//! This module provides the [`Allocation`] struct that tracks how much of an
//! RFQ's target quantity has been assigned to a particular venue's quote.
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::entities::allocation::Allocation;
//! use otc_rfq::domain::value_objects::{Price, Quantity, QuoteId, VenueId};
//!
//! let alloc = Allocation::new(
//!     VenueId::new("venue-1"),
//!     QuoteId::new_v4(),
//!     Quantity::new(1.0).unwrap(),
//!     Price::new(50000.0).unwrap(),
//! ).unwrap();
//!
//! assert_eq!(alloc.venue_id().as_str(), "venue-1");
//! ```

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::{Price, Quantity, QuoteId, VenueId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A quantity allocation to a specific venue quote.
///
/// Represents one leg of a multi-MM fill, specifying how much quantity
/// is allocated to a particular venue's quote at a given price.
///
/// # Invariants
///
/// - `allocated_quantity` must be positive
/// - `price` must be positive
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::entities::allocation::Allocation;
/// use otc_rfq::domain::value_objects::{Price, Quantity, QuoteId, VenueId};
///
/// let alloc = Allocation::new(
///     VenueId::new("binance"),
///     QuoteId::new_v4(),
///     Quantity::new(2.5).unwrap(),
///     Price::new(49500.0).unwrap(),
/// ).unwrap();
///
/// assert!(alloc.allocated_quantity().is_positive());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allocation {
    /// The venue receiving this allocation.
    venue_id: VenueId,
    /// The quote this allocation is based on.
    quote_id: QuoteId,
    /// The quantity allocated to this venue.
    allocated_quantity: Quantity,
    /// The execution price for this allocation.
    price: Price,
}

impl Allocation {
    /// Creates a new allocation with validation.
    ///
    /// # Arguments
    ///
    /// * `venue_id` - The venue receiving this allocation
    /// * `quote_id` - The quote this allocation is based on
    /// * `allocated_quantity` - Quantity allocated (must be positive)
    /// * `price` - Execution price (must be positive)
    ///
    /// # Errors
    ///
    /// Returns `DomainError::InvalidQuantity` if quantity is not positive.
    /// Returns `DomainError::InvalidPrice` if price is not positive.
    pub fn new(
        venue_id: VenueId,
        quote_id: QuoteId,
        allocated_quantity: Quantity,
        price: Price,
    ) -> DomainResult<Self> {
        if !allocated_quantity.is_positive() {
            return Err(DomainError::InvalidQuantity(
                "allocated quantity must be positive".to_string(),
            ));
        }
        if !price.is_positive() {
            return Err(DomainError::InvalidPrice(
                "allocation price must be positive".to_string(),
            ));
        }
        Ok(Self {
            venue_id,
            quote_id,
            allocated_quantity,
            price,
        })
    }

    /// Creates an allocation without validation (for reconstruction from storage).
    ///
    /// # Safety
    ///
    /// This method bypasses validation and should only be used when
    /// reconstructing from trusted storage.
    #[must_use]
    pub fn from_parts(
        venue_id: VenueId,
        quote_id: QuoteId,
        allocated_quantity: Quantity,
        price: Price,
    ) -> Self {
        Self {
            venue_id,
            quote_id,
            allocated_quantity,
            price,
        }
    }

    /// Returns the venue ID.
    #[inline]
    #[must_use]
    pub fn venue_id(&self) -> &VenueId {
        &self.venue_id
    }

    /// Returns the quote ID.
    #[inline]
    #[must_use]
    pub fn quote_id(&self) -> QuoteId {
        self.quote_id
    }

    /// Returns the allocated quantity.
    #[inline]
    #[must_use]
    pub fn allocated_quantity(&self) -> Quantity {
        self.allocated_quantity
    }

    /// Returns the execution price.
    #[inline]
    #[must_use]
    pub fn price(&self) -> Price {
        self.price
    }

    /// Returns the notional value (price * quantity) of this allocation.
    ///
    /// Uses checked arithmetic. Returns `None` if the multiplication overflows.
    #[must_use]
    pub fn notional_value(&self) -> Option<Price> {
        self.price.safe_mul(self.allocated_quantity.get()).ok()
    }
}

impl fmt::Display for Allocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Allocation(venue={}, quote={}, qty={}, price={})",
            self.venue_id, self.quote_id, self.allocated_quantity, self.price,
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_venue() -> VenueId {
        VenueId::new("venue-1")
    }

    fn test_quote_id() -> QuoteId {
        QuoteId::new_v4()
    }

    fn test_qty() -> Quantity {
        Quantity::new(2.5).unwrap()
    }

    fn test_price() -> Price {
        Price::new(50000.0).unwrap()
    }

    mod construction {
        use super::*;

        #[test]
        fn new_creates_valid_allocation() {
            let result = Allocation::new(test_venue(), test_quote_id(), test_qty(), test_price());
            assert!(result.is_ok());
            let alloc = result.unwrap();
            assert_eq!(alloc.venue_id().as_str(), "venue-1");
            assert_eq!(alloc.allocated_quantity(), test_qty());
            assert_eq!(alloc.price(), test_price());
        }

        #[test]
        fn new_fails_with_zero_quantity() {
            let result = Allocation::new(
                test_venue(),
                test_quote_id(),
                Quantity::zero(),
                test_price(),
            );
            assert!(matches!(result, Err(DomainError::InvalidQuantity(_))));
        }

        #[test]
        fn new_fails_with_zero_price() {
            let result = Allocation::new(test_venue(), test_quote_id(), test_qty(), Price::zero());
            assert!(matches!(result, Err(DomainError::InvalidPrice(_))));
        }

        #[test]
        fn from_parts_bypasses_validation() {
            let alloc = Allocation::from_parts(
                test_venue(),
                test_quote_id(),
                Quantity::zero(),
                test_price(),
            );
            assert!(alloc.allocated_quantity().is_zero());
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn all_accessors_work() {
            let venue = test_venue();
            let quote_id = test_quote_id();
            let qty = test_qty();
            let price = test_price();

            let alloc = Allocation::new(venue.clone(), quote_id, qty, price).unwrap();

            assert_eq!(alloc.venue_id(), &venue);
            assert_eq!(alloc.quote_id(), quote_id);
            assert_eq!(alloc.allocated_quantity(), qty);
            assert_eq!(alloc.price(), price);
        }

        #[test]
        fn notional_value_computed() {
            let alloc = Allocation::new(
                test_venue(),
                test_quote_id(),
                Quantity::new(2.0).unwrap(),
                Price::new(100.0).unwrap(),
            )
            .unwrap();

            let notional = alloc.notional_value();
            assert!(notional.is_some());
            assert_eq!(notional.unwrap(), Price::new(200.0).unwrap());
        }
    }

    mod display {
        use super::*;

        #[test]
        fn display_format() {
            let alloc =
                Allocation::new(test_venue(), test_quote_id(), test_qty(), test_price()).unwrap();
            let display = alloc.to_string();
            assert!(display.contains("Allocation"));
            assert!(display.contains("venue-1"));
        }
    }

    mod serde_tests {
        use super::*;

        #[test]
        fn serde_roundtrip() {
            let alloc =
                Allocation::new(test_venue(), test_quote_id(), test_qty(), test_price()).unwrap();
            let json = serde_json::to_string(&alloc).unwrap();
            let deserialized: Allocation = serde_json::from_str(&json).unwrap();
            assert_eq!(alloc, deserialized);
        }
    }
}
