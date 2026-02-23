//! # Negotiation Aggregate
//!
//! Manages a multi-round counter-quote negotiation between requester and market maker.
//!
//! This module provides the [`Negotiation`] aggregate root and [`NegotiationRound`]
//! entity, which together manage the lifecycle of a counter-quote exchange with
//! configurable round limits and price improvement enforcement.
//!
//! # State Machine
//!
//! ```text
//! Open → CounterPending → Open (loop up to max_rounds)
//!   ↓         ↓
//!   ├─────────┴→ Accepted
//!   ├─────────┴→ Rejected
//!   └─────────┴→ Expired
//! ```
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::entities::negotiation::Negotiation;
//! use otc_rfq::domain::entities::counter_quote::CounterQuoteBuilder;
//! use otc_rfq::domain::value_objects::{
//!     CounterpartyId, NegotiationState, OrderSide, Price, Quantity, QuoteId, RfqId, Timestamp,
//! };
//!
//! let mut negotiation = Negotiation::new(
//!     RfqId::new_v4(),
//!     CounterpartyId::new("client-1"),
//!     CounterpartyId::new("mm-1"),
//!     OrderSide::Buy,
//!     3,
//! );
//!
//! assert_eq!(negotiation.state(), NegotiationState::Open);
//! assert_eq!(negotiation.round_count(), 0);
//! ```

use crate::domain::entities::counter_quote::CounterQuote;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::negotiation_state::NegotiationState;
use crate::domain::value_objects::timestamp::Timestamp;
use crate::domain::value_objects::{CounterpartyId, NegotiationId, OrderSide, Price, RfqId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Default maximum number of negotiation rounds.
pub const DEFAULT_MAX_ROUNDS: u8 = 3;

/// A single round in a negotiation, containing a counter-quote and optional response.
///
/// Each round tracks when it was submitted and whether it has been responded to.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::entities::negotiation::NegotiationRound;
/// use otc_rfq::domain::entities::counter_quote::CounterQuoteBuilder;
/// use otc_rfq::domain::value_objects::{
///     CounterpartyId, Price, Quantity, QuoteId, RfqId, Timestamp,
/// };
///
/// let counter = CounterQuoteBuilder::new(
///     QuoteId::new_v4(),
///     RfqId::new_v4(),
///     CounterpartyId::new("mm-1"),
///     Price::new(49500.0).unwrap(),
///     Quantity::new(1.0).unwrap(),
///     Timestamp::now().add_secs(60),
///     1,
/// ).build();
///
/// let round = NegotiationRound::new(1, counter);
/// assert_eq!(round.round_number(), 1);
/// assert!(!round.is_responded());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NegotiationRound {
    /// The round number (1-indexed).
    round_number: u8,
    /// The counter-quote submitted in this round.
    counter_quote: CounterQuote,
    /// When the other party responded, if they did.
    responded_at: Option<Timestamp>,
    /// Whether this round's counter was accepted.
    accepted: Option<bool>,
}

impl NegotiationRound {
    /// Creates a new negotiation round.
    ///
    /// # Arguments
    ///
    /// * `round_number` - The round number (1-indexed)
    /// * `counter_quote` - The counter-quote for this round
    #[must_use]
    pub fn new(round_number: u8, counter_quote: CounterQuote) -> Self {
        Self {
            round_number,
            counter_quote,
            responded_at: None,
            accepted: None,
        }
    }

    /// Returns the round number.
    #[inline]
    #[must_use]
    pub fn round_number(&self) -> u8 {
        self.round_number
    }

    /// Returns the counter-quote for this round.
    #[inline]
    #[must_use]
    pub fn counter_quote(&self) -> &CounterQuote {
        &self.counter_quote
    }

    /// Returns when this round was responded to, if at all.
    #[inline]
    #[must_use]
    pub fn responded_at(&self) -> Option<Timestamp> {
        self.responded_at
    }

    /// Returns whether this round was accepted.
    #[inline]
    #[must_use]
    pub fn accepted(&self) -> Option<bool> {
        self.accepted
    }

    /// Returns true if this round has been responded to.
    #[inline]
    #[must_use]
    pub fn is_responded(&self) -> bool {
        self.responded_at.is_some()
    }

    /// Marks this round as responded to with acceptance or rejection.
    pub fn respond(&mut self, accepted: bool) {
        self.responded_at = Some(Timestamp::now());
        self.accepted = Some(accepted);
    }
}

impl fmt::Display for NegotiationRound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self.accepted {
            Some(true) => "accepted",
            Some(false) => "rejected",
            None => "pending",
        };
        write!(
            f,
            "Round {} [{}]: price={} from={}",
            self.round_number,
            status,
            self.counter_quote.price(),
            self.counter_quote.from_account()
        )
    }
}

/// Negotiation aggregate root managing multi-round counter-quote exchange.
///
/// The [`Negotiation`] aggregate manages the complete lifecycle of a negotiation
/// session between a requester and a market maker. It enforces round limits,
/// price improvement rules, and state transitions.
///
/// # Invariants
///
/// - Maximum rounds enforced (default 3)
/// - Price must improve on each successive round (direction depends on order side)
/// - Only participants (requester or mm) can submit counter-quotes
/// - State transitions follow the negotiation FSM
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::entities::negotiation::Negotiation;
/// use otc_rfq::domain::entities::counter_quote::CounterQuoteBuilder;
/// use otc_rfq::domain::value_objects::{
///     CounterpartyId, NegotiationState, OrderSide, Price, Quantity, QuoteId, RfqId, Timestamp,
/// };
///
/// let mut neg = Negotiation::new(
///     RfqId::new_v4(),
///     CounterpartyId::new("client-1"),
///     CounterpartyId::new("mm-1"),
///     OrderSide::Buy,
///     3,
/// );
///
/// let counter = CounterQuoteBuilder::new(
///     QuoteId::new_v4(),
///     neg.rfq_id(),
///     CounterpartyId::new("mm-1"),
///     Price::new(49000.0).unwrap(),
///     Quantity::new(1.0).unwrap(),
///     Timestamp::now().add_secs(60),
///     1,
/// ).build();
///
/// assert!(neg.submit_counter(counter).is_ok());
/// assert_eq!(neg.state(), NegotiationState::CounterPending);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Negotiation {
    /// Unique identifier for this negotiation.
    id: NegotiationId,
    /// The RFQ this negotiation belongs to.
    rfq_id: RfqId,
    /// The client/requester in the negotiation.
    requester: CounterpartyId,
    /// The market maker in the negotiation.
    mm_account: CounterpartyId,
    /// The order side (affects price improvement direction).
    side: OrderSide,
    /// Rounds of counter-quotes exchanged.
    rounds: Vec<NegotiationRound>,
    /// Maximum allowed rounds.
    max_rounds: u8,
    /// Current negotiation state.
    state: NegotiationState,
    /// When this negotiation was created.
    created_at: Timestamp,
    /// When this negotiation was last updated.
    updated_at: Timestamp,
}

impl Negotiation {
    /// Creates a new negotiation session.
    ///
    /// # Arguments
    ///
    /// * `rfq_id` - The RFQ this negotiation belongs to
    /// * `requester` - The client requesting the quote
    /// * `mm_account` - The market maker providing quotes
    /// * `side` - The order side (Buy/Sell), determines price improvement direction
    /// * `max_rounds` - Maximum number of rounds allowed
    #[must_use]
    pub fn new(
        rfq_id: RfqId,
        requester: CounterpartyId,
        mm_account: CounterpartyId,
        side: OrderSide,
        max_rounds: u8,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: NegotiationId::new_v4(),
            rfq_id,
            requester,
            mm_account,
            side,
            rounds: Vec::new(),
            max_rounds,
            state: NegotiationState::Open,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a negotiation with a specific ID (for reconstruction from storage).
    ///
    /// # Safety
    ///
    /// This method bypasses validation and should only be used when
    /// reconstructing from trusted storage.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        id: NegotiationId,
        rfq_id: RfqId,
        requester: CounterpartyId,
        mm_account: CounterpartyId,
        side: OrderSide,
        rounds: Vec<NegotiationRound>,
        max_rounds: u8,
        state: NegotiationState,
        created_at: Timestamp,
        updated_at: Timestamp,
    ) -> Self {
        Self {
            id,
            rfq_id,
            requester,
            mm_account,
            side,
            rounds,
            max_rounds,
            state,
            created_at,
            updated_at,
        }
    }

    fn transition_to(&mut self, target: NegotiationState) -> DomainResult<()> {
        if !self.state.can_transition_to(target) {
            return Err(DomainError::InvalidNegotiationStateTransition {
                from: self.state,
                to: target,
            });
        }
        self.state = target;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    // ========== Accessors ==========

    /// Returns the negotiation ID.
    #[inline]
    #[must_use]
    pub fn id(&self) -> NegotiationId {
        self.id
    }

    /// Returns the RFQ ID.
    #[inline]
    #[must_use]
    pub fn rfq_id(&self) -> RfqId {
        self.rfq_id
    }

    /// Returns the requester (client).
    #[inline]
    #[must_use]
    pub fn requester(&self) -> &CounterpartyId {
        &self.requester
    }

    /// Returns the market maker account.
    #[inline]
    #[must_use]
    pub fn mm_account(&self) -> &CounterpartyId {
        &self.mm_account
    }

    /// Returns the order side.
    #[inline]
    #[must_use]
    pub fn side(&self) -> OrderSide {
        self.side
    }

    /// Returns the negotiation rounds.
    #[inline]
    #[must_use]
    pub fn rounds(&self) -> &[NegotiationRound] {
        &self.rounds
    }

    /// Returns the maximum number of rounds.
    #[inline]
    #[must_use]
    pub fn max_rounds(&self) -> u8 {
        self.max_rounds
    }

    /// Returns the current state.
    #[inline]
    #[must_use]
    pub fn state(&self) -> NegotiationState {
        self.state
    }

    /// Returns when this negotiation was created.
    #[inline]
    #[must_use]
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    /// Returns when this negotiation was last updated.
    #[inline]
    #[must_use]
    pub fn updated_at(&self) -> Timestamp {
        self.updated_at
    }

    /// Returns the number of rounds completed.
    #[inline]
    #[must_use]
    pub fn round_count(&self) -> usize {
        self.rounds.len()
    }

    /// Returns the latest round, if any.
    #[must_use]
    pub fn latest_round(&self) -> Option<&NegotiationRound> {
        self.rounds.last()
    }

    /// Returns the latest counter-quote price, if any.
    #[must_use]
    pub fn latest_price(&self) -> Option<Price> {
        self.rounds.last().map(|r| r.counter_quote().price())
    }

    /// Returns true if the negotiation is still active.
    #[inline]
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    // ========== State Transitions ==========

    /// Submits a counter-quote from either party.
    ///
    /// Validates:
    /// - Negotiation is in a state that accepts counters (Open or CounterPending)
    /// - Maximum rounds not exceeded
    /// - Price improves over previous round (if not the first round)
    /// - Counter is not expired
    /// - Submitter is a participant in this negotiation
    ///
    /// # Arguments
    ///
    /// * `counter` - The counter-quote to submit
    ///
    /// # Errors
    ///
    /// - `DomainError::InvalidNegotiationStateTransition` if not in valid state
    /// - `DomainError::MaxNegotiationRoundsReached` if max rounds exceeded
    /// - `DomainError::NoPriceImprovement` if price doesn't improve
    /// - `DomainError::QuoteExpired` if counter has expired
    /// - `DomainError::ValidationError` if submitter is not a participant
    pub fn submit_counter(&mut self, counter: CounterQuote) -> DomainResult<()> {
        // Validate state allows counter submission
        if self.state.is_terminal() {
            return Err(DomainError::InvalidNegotiationStateTransition {
                from: self.state,
                to: NegotiationState::CounterPending,
            });
        }

        // Validate counter is not expired
        if counter.is_expired() {
            return Err(DomainError::QuoteExpired(
                "counter-quote has expired".to_string(),
            ));
        }

        // Validate submitter is a participant
        if counter.from_account() != &self.requester && counter.from_account() != &self.mm_account {
            return Err(DomainError::ValidationError(
                "submitter is not a participant in this negotiation".to_string(),
            ));
        }

        // Validate round limits
        if self.rounds.len() >= usize::from(self.max_rounds) {
            return Err(DomainError::MaxNegotiationRoundsReached {
                max_rounds: self.max_rounds,
            });
        }

        // Validate price improvement (skip for first round)
        if let Some(previous_price) = self.latest_price() {
            self.validate_price_improvement(previous_price, counter.price())?;
        }

        // Mark previous round as responded if pending
        if let Some(last) = self.rounds.last_mut()
            && !last.is_responded()
        {
            last.respond(false);
        }

        let round_number = self
            .rounds
            .len()
            .checked_add(1)
            .and_then(|n| u8::try_from(n).ok())
            .ok_or(DomainError::ValidationError(
                "round number overflow".to_string(),
            ))?;

        let round = NegotiationRound::new(round_number, counter);
        self.rounds.push(round);

        // Transition to CounterPending
        match self.state {
            NegotiationState::Open => self.transition_to(NegotiationState::CounterPending)?,
            NegotiationState::CounterPending => {
                // CounterPending → Open → CounterPending (respond then new counter)
                self.state = NegotiationState::Open;
                self.transition_to(NegotiationState::CounterPending)?;
            }
            _ => {
                return Err(DomainError::InvalidNegotiationStateTransition {
                    from: self.state,
                    to: NegotiationState::CounterPending,
                });
            }
        }

        Ok(())
    }

    /// Accepts the current counter-quote, completing the negotiation.
    ///
    /// # Errors
    ///
    /// - `DomainError::InvalidNegotiationStateTransition` if not in valid state
    /// - `DomainError::ValidationError` if there are no rounds to accept
    pub fn accept(&mut self) -> DomainResult<()> {
        if self.rounds.is_empty() {
            return Err(DomainError::ValidationError(
                "cannot accept with no counter-quotes".to_string(),
            ));
        }

        // Mark latest round as accepted
        if let Some(last) = self.rounds.last_mut() {
            last.respond(true);
        }

        self.transition_to(NegotiationState::Accepted)
    }

    /// Rejects the negotiation.
    ///
    /// # Errors
    ///
    /// - `DomainError::InvalidNegotiationStateTransition` if in terminal state
    pub fn reject(&mut self) -> DomainResult<()> {
        // Mark latest round as rejected if pending
        if let Some(last) = self.rounds.last_mut()
            && !last.is_responded()
        {
            last.respond(false);
        }

        self.transition_to(NegotiationState::Rejected)
    }

    /// Expires the negotiation.
    ///
    /// # Errors
    ///
    /// - `DomainError::InvalidNegotiationStateTransition` if in terminal state
    pub fn expire(&mut self) -> DomainResult<()> {
        self.transition_to(NegotiationState::Expired)
    }

    /// Validates that the proposed price improves over the previous price.
    ///
    /// For **Buy** side: price must **decrease** (buyer wants lower price).
    /// For **Sell** side: price must **increase** (seller wants higher price).
    fn validate_price_improvement(&self, previous: Price, proposed: Price) -> DomainResult<()> {
        let is_improvement = match self.side {
            OrderSide::Buy => proposed < previous,
            OrderSide::Sell => proposed > previous,
        };

        if !is_improvement {
            return Err(DomainError::NoPriceImprovement { previous, proposed });
        }

        Ok(())
    }
}

impl fmt::Display for Negotiation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Negotiation[{}] rfq={} state={} rounds={}/{}",
            self.id,
            self.rfq_id,
            self.state,
            self.rounds.len(),
            self.max_rounds
        )
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entities::counter_quote::CounterQuoteBuilder;
    use crate::domain::value_objects::{Quantity, QuoteId};

    fn test_rfq_id() -> RfqId {
        RfqId::new_v4()
    }

    fn test_requester() -> CounterpartyId {
        CounterpartyId::new("client-1")
    }

    fn test_mm() -> CounterpartyId {
        CounterpartyId::new("mm-1")
    }

    fn future_timestamp() -> Timestamp {
        Timestamp::now().add_secs(300)
    }

    fn create_test_negotiation(side: OrderSide) -> Negotiation {
        Negotiation::new(test_rfq_id(), test_requester(), test_mm(), side, 3)
    }

    fn make_counter(rfq_id: RfqId, from: CounterpartyId, price: f64, round: u8) -> CounterQuote {
        CounterQuoteBuilder::new(
            QuoteId::new_v4(),
            rfq_id,
            from,
            Price::new(price).unwrap(),
            Quantity::new(1.0).unwrap(),
            future_timestamp(),
            round,
        )
        .build()
    }

    mod construction {
        use super::*;

        #[test]
        fn new_creates_open_negotiation() {
            let neg = create_test_negotiation(OrderSide::Buy);
            assert_eq!(neg.state(), NegotiationState::Open);
            assert_eq!(neg.round_count(), 0);
            assert_eq!(neg.max_rounds(), 3);
            assert!(neg.is_active());
            assert!(neg.latest_round().is_none());
            assert!(neg.latest_price().is_none());
        }

        #[test]
        fn accessors_return_correct_values() {
            let rfq_id = test_rfq_id();
            let requester = test_requester();
            let mm = test_mm();
            let neg = Negotiation::new(rfq_id, requester.clone(), mm.clone(), OrderSide::Sell, 5);

            assert_eq!(neg.rfq_id(), rfq_id);
            assert_eq!(neg.requester(), &requester);
            assert_eq!(neg.mm_account(), &mm);
            assert_eq!(neg.side(), OrderSide::Sell);
            assert_eq!(neg.max_rounds(), 5);
        }
    }

    mod submit_counter {
        use super::*;

        #[test]
        fn first_counter_transitions_to_pending() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let counter = make_counter(neg.rfq_id(), test_mm(), 50000.0, 1);

            assert!(neg.submit_counter(counter).is_ok());
            assert_eq!(neg.state(), NegotiationState::CounterPending);
            assert_eq!(neg.round_count(), 1);
        }

        #[test]
        fn second_counter_with_price_improvement_succeeds_buy() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let rfq_id = neg.rfq_id();

            // First counter at 50000
            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            // Second counter at 49500 (lower = improvement for buyer)
            let c2 = make_counter(rfq_id, test_requester(), 49500.0, 2);
            assert!(neg.submit_counter(c2).is_ok());
            assert_eq!(neg.round_count(), 2);
        }

        #[test]
        fn second_counter_with_price_improvement_succeeds_sell() {
            let mut neg = create_test_negotiation(OrderSide::Sell);
            let rfq_id = neg.rfq_id();

            // First counter at 50000
            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            // Second counter at 50500 (higher = improvement for seller)
            let c2 = make_counter(rfq_id, test_requester(), 50500.0, 2);
            assert!(neg.submit_counter(c2).is_ok());
        }

        #[test]
        fn no_price_improvement_fails_buy() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let rfq_id = neg.rfq_id();

            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            // Same price = no improvement
            let c2 = make_counter(rfq_id, test_requester(), 50000.0, 2);
            let result = neg.submit_counter(c2);
            assert!(matches!(
                result,
                Err(DomainError::NoPriceImprovement { .. })
            ));
        }

        #[test]
        fn no_price_improvement_fails_sell() {
            let mut neg = create_test_negotiation(OrderSide::Sell);
            let rfq_id = neg.rfq_id();

            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            // Lower price = no improvement for seller
            let c2 = make_counter(rfq_id, test_requester(), 49500.0, 2);
            let result = neg.submit_counter(c2);
            assert!(matches!(
                result,
                Err(DomainError::NoPriceImprovement { .. })
            ));
        }

        #[test]
        fn max_rounds_enforced() {
            let mut neg = Negotiation::new(
                test_rfq_id(),
                test_requester(),
                test_mm(),
                OrderSide::Buy,
                2,
            );
            let rfq_id = neg.rfq_id();

            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            let c2 = make_counter(rfq_id, test_requester(), 49500.0, 2);
            neg.submit_counter(c2).unwrap();

            // Third counter exceeds max_rounds=2
            let c3 = make_counter(rfq_id, test_mm(), 49000.0, 3);
            let result = neg.submit_counter(c3);
            assert!(matches!(
                result,
                Err(DomainError::MaxNegotiationRoundsReached { max_rounds: 2 })
            ));
        }

        #[test]
        fn non_participant_rejected() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let rfq_id = neg.rfq_id();

            let outsider = CounterpartyId::new("outsider");
            let counter = make_counter(rfq_id, outsider, 49000.0, 1);
            let result = neg.submit_counter(counter);
            assert!(matches!(result, Err(DomainError::ValidationError(_))));
        }

        #[test]
        fn expired_counter_rejected() {
            let mut neg = create_test_negotiation(OrderSide::Buy);

            let expired_counter = CounterQuoteBuilder::new(
                QuoteId::new_v4(),
                neg.rfq_id(),
                test_mm(),
                Price::new(49000.0).unwrap(),
                Quantity::new(1.0).unwrap(),
                Timestamp::now().sub_secs(60), // expired
                1,
            )
            .build();

            let result = neg.submit_counter(expired_counter);
            assert!(matches!(result, Err(DomainError::QuoteExpired(_))));
        }

        #[test]
        fn cannot_submit_after_accepted() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let rfq_id = neg.rfq_id();

            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();
            neg.accept().unwrap();

            let c2 = make_counter(rfq_id, test_requester(), 49500.0, 2);
            let result = neg.submit_counter(c2);
            assert!(matches!(
                result,
                Err(DomainError::InvalidNegotiationStateTransition { .. })
            ));
        }

        #[test]
        fn cannot_submit_after_rejected() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let rfq_id = neg.rfq_id();

            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();
            neg.reject().unwrap();

            let c2 = make_counter(rfq_id, test_requester(), 49500.0, 2);
            let result = neg.submit_counter(c2);
            assert!(matches!(
                result,
                Err(DomainError::InvalidNegotiationStateTransition { .. })
            ));
        }
    }

    mod accept {
        use super::*;

        #[test]
        fn accept_from_counter_pending() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let c1 = make_counter(neg.rfq_id(), test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            assert!(neg.accept().is_ok());
            assert_eq!(neg.state(), NegotiationState::Accepted);
            assert!(!neg.is_active());

            // Latest round should be marked as accepted
            let last = neg.latest_round().unwrap();
            assert!(last.is_responded());
            assert_eq!(last.accepted(), Some(true));
        }

        #[test]
        fn accept_fails_with_no_rounds() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let result = neg.accept();
            assert!(matches!(result, Err(DomainError::ValidationError(_))));
        }

        #[test]
        fn accept_fails_from_terminal() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let c1 = make_counter(neg.rfq_id(), test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();
            neg.accept().unwrap();

            // Already accepted, cannot accept again
            let result = neg.accept();
            assert!(matches!(
                result,
                Err(DomainError::InvalidNegotiationStateTransition { .. })
            ));
        }
    }

    mod reject {
        use super::*;

        #[test]
        fn reject_from_open() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            assert!(neg.reject().is_ok());
            assert_eq!(neg.state(), NegotiationState::Rejected);
        }

        #[test]
        fn reject_from_counter_pending() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let c1 = make_counter(neg.rfq_id(), test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            assert!(neg.reject().is_ok());
            assert_eq!(neg.state(), NegotiationState::Rejected);

            // Last round should be marked as rejected
            let last = neg.latest_round().unwrap();
            assert!(last.is_responded());
            assert_eq!(last.accepted(), Some(false));
        }

        #[test]
        fn reject_fails_from_terminal() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            neg.reject().unwrap();

            let result = neg.reject();
            assert!(matches!(
                result,
                Err(DomainError::InvalidNegotiationStateTransition { .. })
            ));
        }
    }

    mod expire {
        use super::*;

        #[test]
        fn expire_from_open() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            assert!(neg.expire().is_ok());
            assert_eq!(neg.state(), NegotiationState::Expired);
        }

        #[test]
        fn expire_from_counter_pending() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let c1 = make_counter(neg.rfq_id(), test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            assert!(neg.expire().is_ok());
            assert_eq!(neg.state(), NegotiationState::Expired);
        }

        #[test]
        fn expire_fails_from_terminal() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            neg.expire().unwrap();

            let result = neg.expire();
            assert!(matches!(
                result,
                Err(DomainError::InvalidNegotiationStateTransition { .. })
            ));
        }
    }

    mod multi_round {
        use super::*;

        #[test]
        fn full_negotiation_flow_buy_side() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let rfq_id = neg.rfq_id();

            // Round 1: MM offers 50000
            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();
            assert_eq!(neg.state(), NegotiationState::CounterPending);

            // Round 2: Client counters 49000 (improvement for buyer)
            let c2 = make_counter(rfq_id, test_requester(), 49000.0, 2);
            neg.submit_counter(c2).unwrap();
            assert_eq!(neg.round_count(), 2);

            // Round 3: MM counters 48500 (improvement for buyer)
            let c3 = make_counter(rfq_id, test_mm(), 48500.0, 3);
            neg.submit_counter(c3).unwrap();
            assert_eq!(neg.round_count(), 3);

            // Client accepts
            neg.accept().unwrap();
            assert_eq!(neg.state(), NegotiationState::Accepted);
            assert_eq!(neg.latest_price(), Some(Price::new(48500.0).unwrap()));
        }

        #[test]
        fn full_negotiation_flow_sell_side() {
            let mut neg = create_test_negotiation(OrderSide::Sell);
            let rfq_id = neg.rfq_id();

            // Round 1: MM offers 50000
            let c1 = make_counter(rfq_id, test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            // Round 2: Client counters 50500 (improvement for seller)
            let c2 = make_counter(rfq_id, test_requester(), 50500.0, 2);
            neg.submit_counter(c2).unwrap();

            // Round 3: MM counters 51000 (improvement for seller)
            let c3 = make_counter(rfq_id, test_mm(), 51000.0, 3);
            neg.submit_counter(c3).unwrap();

            neg.accept().unwrap();
            assert_eq!(neg.state(), NegotiationState::Accepted);
        }
    }

    mod display {
        use super::*;

        #[test]
        fn display_format() {
            let neg = create_test_negotiation(OrderSide::Buy);
            let display = neg.to_string();
            assert!(display.contains("Negotiation"));
            assert!(display.contains("OPEN"));
            assert!(display.contains("0/3"));
        }
    }

    mod serde {
        use super::*;

        #[test]
        fn negotiation_serde_roundtrip() {
            let mut neg = create_test_negotiation(OrderSide::Buy);
            let c1 = make_counter(neg.rfq_id(), test_mm(), 50000.0, 1);
            neg.submit_counter(c1).unwrap();

            let json = serde_json::to_string(&neg).unwrap();
            let deserialized: Negotiation = serde_json::from_str(&json).unwrap();
            assert_eq!(neg.id(), deserialized.id());
            assert_eq!(neg.state(), deserialized.state());
            assert_eq!(neg.round_count(), deserialized.round_count());
        }

        #[test]
        fn round_serde_roundtrip() {
            let counter = make_counter(test_rfq_id(), test_mm(), 50000.0, 1);
            let round = NegotiationRound::new(1, counter);

            let json = serde_json::to_string(&round).unwrap();
            let deserialized: NegotiationRound = serde_json::from_str(&json).unwrap();
            assert_eq!(round.round_number(), deserialized.round_number());
        }
    }

    mod from_parts {
        use super::*;

        #[test]
        fn reconstructs_from_parts() {
            let id = NegotiationId::new_v4();
            let rfq_id = test_rfq_id();
            let now = Timestamp::now();

            let neg = Negotiation::from_parts(
                id,
                rfq_id,
                test_requester(),
                test_mm(),
                OrderSide::Buy,
                vec![],
                3,
                NegotiationState::Open,
                now,
                now,
            );

            assert_eq!(neg.id(), id);
            assert_eq!(neg.rfq_id(), rfq_id);
            assert_eq!(neg.state(), NegotiationState::Open);
        }
    }
}
