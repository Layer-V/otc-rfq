//! # Counter-Quote Entity
//!
//! Represents a counter-offer in a negotiation between requester and market maker.
//!
//! This module provides the [`CounterQuote`] entity, which represents a single
//! counter-offer submitted during a multi-round negotiation. Each counter-quote
//! references the original quote and tracks the round number.
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::entities::counter_quote::{CounterQuote, CounterQuoteBuilder};
//! use otc_rfq::domain::value_objects::{
//!     CounterpartyId, Price, Quantity, QuoteId, RfqId, Timestamp,
//! };
//!
//! let counter = CounterQuoteBuilder::new(
//!     QuoteId::new_v4(),
//!     RfqId::new_v4(),
//!     CounterpartyId::new("client-1"),
//!     Price::new(49500.0).unwrap(),
//!     Quantity::new(1.0).unwrap(),
//!     Timestamp::now().add_secs(60),
//!     1,
//! ).build();
//!
//! assert_eq!(counter.round(), 1);
//! assert!(!counter.is_expired());
//! ```

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::timestamp::Timestamp;
use crate::domain::value_objects::{CounterpartyId, Price, Quantity, QuoteId, RfqId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A counter-offer submitted during negotiation.
///
/// Represents a single counter-quote from either the requester or the market maker
/// during a multi-round negotiation session. Each counter-quote must reference
/// the original quote being negotiated and specify the round number.
///
/// # Invariants
///
/// - Price must be positive
/// - Quantity must be positive
/// - `valid_until` must be in the future when created
/// - Round must be >= 1
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::entities::counter_quote::{CounterQuote, CounterQuoteBuilder};
/// use otc_rfq::domain::value_objects::{
///     CounterpartyId, Price, Quantity, QuoteId, RfqId, Timestamp,
/// };
///
/// let counter = CounterQuoteBuilder::new(
///     QuoteId::new_v4(),
///     RfqId::new_v4(),
///     CounterpartyId::new("mm-1"),
///     Price::new(50000.0).unwrap(),
///     Quantity::new(1.5).unwrap(),
///     Timestamp::now().add_secs(120),
///     1,
/// ).build();
///
/// assert!(counter.price().is_positive());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterQuote {
    /// Unique identifier for this counter-quote.
    id: QuoteId,
    /// The original quote this counter references.
    original_quote_id: QuoteId,
    /// The RFQ this negotiation belongs to.
    rfq_id: RfqId,
    /// The counterparty submitting this counter-quote.
    from_account: CounterpartyId,
    /// The proposed counter price.
    counter_price: Price,
    /// The proposed counter quantity.
    counter_quantity: Quantity,
    /// When this counter-quote expires.
    valid_until: Timestamp,
    /// The negotiation round number (1-indexed).
    round: u8,
    /// When this counter-quote was created.
    created_at: Timestamp,
}

impl CounterQuote {
    /// Creates a new counter-quote with validation.
    ///
    /// # Arguments
    ///
    /// * `original_quote_id` - The quote being countered
    /// * `rfq_id` - The RFQ this belongs to
    /// * `from_account` - Who is submitting the counter
    /// * `counter_price` - The proposed price (must be positive)
    /// * `counter_quantity` - The proposed quantity (must be positive)
    /// * `valid_until` - When this counter expires (must be in the future)
    /// * `round` - The round number (must be >= 1)
    ///
    /// # Errors
    ///
    /// Returns `DomainError::InvalidPrice` if price is not positive.
    /// Returns `DomainError::InvalidQuantity` if quantity is not positive.
    /// Returns `DomainError::QuoteExpired` if valid_until is in the past.
    /// Returns `DomainError::ValidationError` if round is 0.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        original_quote_id: QuoteId,
        rfq_id: RfqId,
        from_account: CounterpartyId,
        counter_price: Price,
        counter_quantity: Quantity,
        valid_until: Timestamp,
        round: u8,
    ) -> DomainResult<Self> {
        Self::validate_price(&counter_price)?;
        Self::validate_quantity(&counter_quantity)?;
        Self::validate_expiry(&valid_until)?;
        Self::validate_round(round)?;

        Ok(Self {
            id: QuoteId::new_v4(),
            original_quote_id,
            rfq_id,
            from_account,
            counter_price,
            counter_quantity,
            valid_until,
            round,
            created_at: Timestamp::now(),
        })
    }

    /// Creates a counter-quote from stored parts (reconstruction from storage).
    ///
    /// # Safety
    ///
    /// This method bypasses validation and should only be used when
    /// reconstructing from trusted storage.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        id: QuoteId,
        original_quote_id: QuoteId,
        rfq_id: RfqId,
        from_account: CounterpartyId,
        counter_price: Price,
        counter_quantity: Quantity,
        valid_until: Timestamp,
        round: u8,
        created_at: Timestamp,
    ) -> Self {
        Self {
            id,
            original_quote_id,
            rfq_id,
            from_account,
            counter_price,
            counter_quantity,
            valid_until,
            round,
            created_at,
        }
    }

    fn validate_price(price: &Price) -> DomainResult<()> {
        if !price.is_positive() {
            return Err(DomainError::InvalidPrice(
                "counter price must be positive".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_quantity(quantity: &Quantity) -> DomainResult<()> {
        if !quantity.is_positive() {
            return Err(DomainError::InvalidQuantity(
                "counter quantity must be positive".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_expiry(valid_until: &Timestamp) -> DomainResult<()> {
        if valid_until.is_expired() {
            return Err(DomainError::QuoteExpired(
                "counter-quote valid_until must be in the future".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_round(round: u8) -> DomainResult<()> {
        if round == 0 {
            return Err(DomainError::ValidationError(
                "round must be >= 1".to_string(),
            ));
        }
        Ok(())
    }

    // ========== Accessors ==========

    /// Returns the counter-quote ID.
    #[inline]
    #[must_use]
    pub fn id(&self) -> QuoteId {
        self.id
    }

    /// Returns the original quote ID being countered.
    #[inline]
    #[must_use]
    pub fn original_quote_id(&self) -> QuoteId {
        self.original_quote_id
    }

    /// Returns the RFQ ID.
    #[inline]
    #[must_use]
    pub fn rfq_id(&self) -> RfqId {
        self.rfq_id
    }

    /// Returns the counterparty who submitted this counter.
    #[inline]
    #[must_use]
    pub fn from_account(&self) -> &CounterpartyId {
        &self.from_account
    }

    /// Returns the proposed counter price.
    #[inline]
    #[must_use]
    pub fn price(&self) -> Price {
        self.counter_price
    }

    /// Returns the proposed counter quantity.
    #[inline]
    #[must_use]
    pub fn quantity(&self) -> Quantity {
        self.counter_quantity
    }

    /// Returns when this counter-quote expires.
    #[inline]
    #[must_use]
    pub fn valid_until(&self) -> Timestamp {
        self.valid_until
    }

    /// Returns the negotiation round number (1-indexed).
    #[inline]
    #[must_use]
    pub fn round(&self) -> u8 {
        self.round
    }

    /// Returns when this counter-quote was created.
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    /// Returns true if this counter-quote has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.valid_until.is_expired()
    }
}

impl fmt::Display for CounterQuote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CounterQuote[{}] round={} price={} qty={} from={}",
            self.id, self.round, self.counter_price, self.counter_quantity, self.from_account
        )
    }
}

/// Builder for constructing a [`CounterQuote`].
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::entities::counter_quote::CounterQuoteBuilder;
/// use otc_rfq::domain::value_objects::{
///     CounterpartyId, Price, Quantity, QuoteId, RfqId, Timestamp,
/// };
///
/// let counter = CounterQuoteBuilder::new(
///     QuoteId::new_v4(),
///     RfqId::new_v4(),
///     CounterpartyId::new("client-1"),
///     Price::new(49500.0).unwrap(),
///     Quantity::new(1.0).unwrap(),
///     Timestamp::now().add_secs(60),
///     1,
/// ).build();
/// ```
#[must_use = "builders do nothing unless .build() is called"]
pub struct CounterQuoteBuilder {
    original_quote_id: QuoteId,
    rfq_id: RfqId,
    from_account: CounterpartyId,
    counter_price: Price,
    counter_quantity: Quantity,
    valid_until: Timestamp,
    round: u8,
}

impl CounterQuoteBuilder {
    /// Creates a new builder with required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        original_quote_id: QuoteId,
        rfq_id: RfqId,
        from_account: CounterpartyId,
        counter_price: Price,
        counter_quantity: Quantity,
        valid_until: Timestamp,
        round: u8,
    ) -> Self {
        Self {
            original_quote_id,
            rfq_id,
            from_account,
            counter_price,
            counter_quantity,
            valid_until,
            round,
        }
    }

    /// Builds the counter-quote without validation.
    ///
    /// For production use, prefer [`CounterQuote::new`] which validates inputs.
    #[must_use]
    pub fn build(self) -> CounterQuote {
        CounterQuote {
            id: QuoteId::new_v4(),
            original_quote_id: self.original_quote_id,
            rfq_id: self.rfq_id,
            from_account: self.from_account,
            counter_price: self.counter_price,
            counter_quantity: self.counter_quantity,
            valid_until: self.valid_until,
            round: self.round,
            created_at: Timestamp::now(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn future_timestamp() -> Timestamp {
        Timestamp::now().add_secs(300)
    }

    fn past_timestamp() -> Timestamp {
        Timestamp::now().sub_secs(300)
    }

    fn test_price() -> Price {
        Price::new(50000.0).unwrap()
    }

    fn test_quantity() -> Quantity {
        Quantity::new(1.0).unwrap()
    }

    mod construction {
        use super::*;

        #[test]
        fn new_creates_valid_counter_quote() {
            let result = CounterQuote::new(
                QuoteId::new_v4(),
                RfqId::new_v4(),
                CounterpartyId::new("client-1"),
                test_price(),
                test_quantity(),
                future_timestamp(),
                1,
            );

            assert!(result.is_ok());
            let counter = result.unwrap();
            assert_eq!(counter.round(), 1);
            assert_eq!(counter.price(), test_price());
            assert!(!counter.is_expired());
        }

        #[test]
        fn new_fails_with_zero_price() {
            let result = CounterQuote::new(
                QuoteId::new_v4(),
                RfqId::new_v4(),
                CounterpartyId::new("client-1"),
                Price::zero(),
                test_quantity(),
                future_timestamp(),
                1,
            );

            assert!(matches!(result, Err(DomainError::InvalidPrice(_))));
        }

        #[test]
        fn new_fails_with_zero_quantity() {
            let result = CounterQuote::new(
                QuoteId::new_v4(),
                RfqId::new_v4(),
                CounterpartyId::new("client-1"),
                test_price(),
                Quantity::zero(),
                future_timestamp(),
                1,
            );

            assert!(matches!(result, Err(DomainError::InvalidQuantity(_))));
        }

        #[test]
        fn new_fails_with_expired_timestamp() {
            let result = CounterQuote::new(
                QuoteId::new_v4(),
                RfqId::new_v4(),
                CounterpartyId::new("client-1"),
                test_price(),
                test_quantity(),
                past_timestamp(),
                1,
            );

            assert!(matches!(result, Err(DomainError::QuoteExpired(_))));
        }

        #[test]
        fn new_fails_with_round_zero() {
            let result = CounterQuote::new(
                QuoteId::new_v4(),
                RfqId::new_v4(),
                CounterpartyId::new("client-1"),
                test_price(),
                test_quantity(),
                future_timestamp(),
                0,
            );

            assert!(matches!(result, Err(DomainError::ValidationError(_))));
        }

        #[test]
        fn builder_creates_counter_quote() {
            let counter = CounterQuoteBuilder::new(
                QuoteId::new_v4(),
                RfqId::new_v4(),
                CounterpartyId::new("mm-1"),
                test_price(),
                test_quantity(),
                future_timestamp(),
                2,
            )
            .build();

            assert_eq!(counter.round(), 2);
            assert!(counter.price().is_positive());
        }
    }

    mod accessors {
        use super::*;

        #[test]
        fn all_accessors_work() {
            let original_id = QuoteId::new_v4();
            let rfq_id = RfqId::new_v4();
            let from = CounterpartyId::new("client-1");
            let price = test_price();
            let qty = test_quantity();
            let valid = future_timestamp();

            let counter =
                CounterQuoteBuilder::new(original_id, rfq_id, from.clone(), price, qty, valid, 3)
                    .build();

            assert_eq!(counter.original_quote_id(), original_id);
            assert_eq!(counter.rfq_id(), rfq_id);
            assert_eq!(counter.from_account(), &from);
            assert_eq!(counter.price(), price);
            assert_eq!(counter.quantity(), qty);
            assert_eq!(counter.valid_until(), valid);
            assert_eq!(counter.round(), 3);
        }
    }

    mod display {
        use super::*;

        #[test]
        fn display_format() {
            let counter = CounterQuoteBuilder::new(
                QuoteId::new_v4(),
                RfqId::new_v4(),
                CounterpartyId::new("mm-1"),
                test_price(),
                test_quantity(),
                future_timestamp(),
                1,
            )
            .build();

            let display = counter.to_string();
            assert!(display.contains("CounterQuote"));
            assert!(display.contains("round=1"));
            assert!(display.contains("mm-1"));
        }
    }

    mod serde {
        use super::*;

        #[test]
        fn serde_roundtrip() {
            let counter = CounterQuoteBuilder::new(
                QuoteId::new_v4(),
                RfqId::new_v4(),
                CounterpartyId::new("client-1"),
                test_price(),
                test_quantity(),
                future_timestamp(),
                1,
            )
            .build();

            let json = serde_json::to_string(&counter).unwrap();
            let deserialized: CounterQuote = serde_json::from_str(&json).unwrap();
            assert_eq!(counter.id(), deserialized.id());
            assert_eq!(counter.round(), deserialized.round());
            assert_eq!(counter.price(), deserialized.price());
        }
    }

    mod from_parts {
        use super::*;

        #[test]
        fn from_parts_reconstructs() {
            let id = QuoteId::new_v4();
            let original_id = QuoteId::new_v4();
            let rfq_id = RfqId::new_v4();
            let now = Timestamp::now();

            let counter = CounterQuote::from_parts(
                id,
                original_id,
                rfq_id,
                CounterpartyId::new("mm-1"),
                test_price(),
                test_quantity(),
                future_timestamp(),
                2,
                now,
            );

            assert_eq!(counter.id(), id);
            assert_eq!(counter.original_quote_id(), original_id);
            assert_eq!(counter.rfq_id(), rfq_id);
            assert_eq!(counter.round(), 2);
            assert_eq!(counter.created_at(), now);
        }
    }
}
