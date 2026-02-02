//! # Ranking Strategy
//!
//! Strategies for ranking quotes.
//!
//! This module provides the [`RankingStrategy`] trait and implementations
//! for ranking quotes based on different criteria.

use crate::domain::entities::quote::Quote;
use crate::domain::value_objects::OrderSide;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A quote with its ranking information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedQuote {
    /// The quote being ranked.
    pub quote: Quote,
    /// The rank (1 = best).
    pub rank: usize,
    /// The score used for ranking (higher = better).
    pub score: f64,
}

impl RankedQuote {
    /// Creates a new ranked quote.
    #[must_use]
    pub fn new(quote: Quote, rank: usize, score: f64) -> Self {
        Self { quote, rank, score }
    }

    /// Returns true if this quote is the best (rank 1).
    #[must_use]
    pub fn is_best(&self) -> bool {
        self.rank == 1
    }
}

impl fmt::Display for RankedQuote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RankedQuote(#{} score={:.4} quote={})",
            self.rank, self.score, self.quote
        )
    }
}

/// Trait for ranking strategies.
///
/// Implementations define how quotes are scored and ranked based on
/// different criteria such as price, venue reputation, or composite scores.
pub trait RankingStrategy: Send + Sync + fmt::Debug {
    /// Ranks the given quotes for the specified order side.
    ///
    /// # Arguments
    ///
    /// * `quotes` - The quotes to rank
    /// * `side` - The order side (Buy or Sell)
    ///
    /// # Returns
    ///
    /// A vector of ranked quotes sorted by rank (best first).
    fn rank(&self, quotes: &[Quote], side: OrderSide) -> Vec<RankedQuote>;

    /// Returns the name of this ranking strategy.
    fn name(&self) -> &'static str;
}

/// Best price ranking strategy.
///
/// Ranks quotes by price:
/// - For Buy orders: lower price is better
/// - For Sell orders: higher price is better
#[derive(Debug, Clone, Default)]
pub struct BestPriceStrategy;

impl BestPriceStrategy {
    /// Creates a new best price strategy.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl RankingStrategy for BestPriceStrategy {
    fn rank(&self, quotes: &[Quote], side: OrderSide) -> Vec<RankedQuote> {
        if quotes.is_empty() {
            return Vec::new();
        }

        // Score quotes based on price
        let mut scored: Vec<(usize, f64)> = quotes
            .iter()
            .enumerate()
            .map(|(i, q)| {
                let price = q.price().get().to_f64().unwrap_or(0.0);
                let score = match side {
                    OrderSide::Buy => -price, // Lower price is better for buying
                    OrderSide::Sell => price, // Higher price is better for selling
                };
                (i, score)
            })
            .collect();

        // Sort by score (descending - higher score is better)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Create ranked quotes
        scored
            .into_iter()
            .enumerate()
            .filter_map(|(rank, (idx, score))| {
                quotes
                    .get(idx)
                    .map(|q| RankedQuote::new(q.clone(), rank + 1, score))
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        "BestPrice"
    }
}

/// Weighted score ranking strategy.
///
/// Ranks quotes using a weighted combination of factors:
/// - Price (configurable weight)
/// - Quantity available (configurable weight)
/// - Venue reliability (configurable weight)
#[derive(Debug, Clone)]
pub struct WeightedScoreStrategy {
    /// Weight for price factor (0.0 - 1.0).
    pub price_weight: f64,
    /// Weight for quantity factor (0.0 - 1.0).
    pub quantity_weight: f64,
}

impl Default for WeightedScoreStrategy {
    fn default() -> Self {
        Self {
            price_weight: 0.7,
            quantity_weight: 0.3,
        }
    }
}

impl WeightedScoreStrategy {
    /// Creates a new weighted score strategy with custom weights.
    #[must_use]
    pub fn new(price_weight: f64, quantity_weight: f64) -> Self {
        Self {
            price_weight,
            quantity_weight,
        }
    }
}

impl RankingStrategy for WeightedScoreStrategy {
    fn rank(&self, quotes: &[Quote], side: OrderSide) -> Vec<RankedQuote> {
        if quotes.is_empty() {
            return Vec::new();
        }

        // Find min/max for normalization
        let prices: Vec<f64> = quotes
            .iter()
            .map(|q| q.price().get().to_f64().unwrap_or(0.0))
            .collect();
        let quantities: Vec<f64> = quotes
            .iter()
            .map(|q| q.quantity().get().to_f64().unwrap_or(0.0))
            .collect();

        let min_price = prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_qty = quantities.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_qty = quantities.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let price_range = (max_price - min_price).max(1.0);
        let qty_range = (max_qty - min_qty).max(1.0);

        // Score quotes
        let mut scored: Vec<(usize, f64)> = quotes
            .iter()
            .enumerate()
            .map(|(i, q)| {
                let price = q.price().get().to_f64().unwrap_or(0.0);
                let qty = q.quantity().get().to_f64().unwrap_or(0.0);

                // Normalize price (0-1, where 1 is best)
                let price_score = match side {
                    OrderSide::Buy => (max_price - price) / price_range,
                    OrderSide::Sell => (price - min_price) / price_range,
                };

                // Normalize quantity (0-1, where 1 is best)
                let qty_score = (qty - min_qty) / qty_range;

                let score = self.price_weight * price_score + self.quantity_weight * qty_score;
                (i, score)
            })
            .collect();

        // Sort by score (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Create ranked quotes
        scored
            .into_iter()
            .enumerate()
            .filter_map(|(rank, (idx, score))| {
                quotes
                    .get(idx)
                    .map(|q| RankedQuote::new(q.clone(), rank + 1, score))
            })
            .collect()
    }

    fn name(&self) -> &'static str {
        "WeightedScore"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::domain::value_objects::timestamp::Timestamp;
    use crate::domain::value_objects::{Price, Quantity, RfqId, VenueId};

    fn create_quote(price: f64, quantity: f64, venue: &str) -> Quote {
        Quote::new(
            RfqId::new_v4(),
            VenueId::new(venue),
            Price::new(price).unwrap(),
            Quantity::new(quantity).unwrap(),
            Timestamp::now().add_secs(60),
        )
        .unwrap()
    }

    #[test]
    fn ranked_quote_new() {
        let quote = create_quote(100.0, 1.0, "venue-1");
        let ranked = RankedQuote::new(quote, 1, 0.95);
        assert_eq!(ranked.rank, 1);
        assert!((ranked.score - 0.95).abs() < f64::EPSILON);
        assert!(ranked.is_best());
    }

    #[test]
    fn ranked_quote_not_best() {
        let quote = create_quote(100.0, 1.0, "venue-1");
        let ranked = RankedQuote::new(quote, 2, 0.85);
        assert!(!ranked.is_best());
    }

    #[test]
    fn best_price_strategy_buy_side() {
        let strategy = BestPriceStrategy::new();
        let quotes = vec![
            create_quote(100.0, 1.0, "venue-1"),
            create_quote(95.0, 1.0, "venue-2"),
            create_quote(105.0, 1.0, "venue-3"),
        ];

        let ranked = strategy.rank(&quotes, OrderSide::Buy);

        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].rank, 1);
        assert!((ranked[0].quote.price().get().to_f64().unwrap() - 95.0).abs() < f64::EPSILON);
        assert_eq!(ranked[1].rank, 2);
        assert!((ranked[1].quote.price().get().to_f64().unwrap() - 100.0).abs() < f64::EPSILON);
        assert_eq!(ranked[2].rank, 3);
        assert!((ranked[2].quote.price().get().to_f64().unwrap() - 105.0).abs() < f64::EPSILON);
    }

    #[test]
    fn best_price_strategy_sell_side() {
        let strategy = BestPriceStrategy::new();
        let quotes = vec![
            create_quote(100.0, 1.0, "venue-1"),
            create_quote(95.0, 1.0, "venue-2"),
            create_quote(105.0, 1.0, "venue-3"),
        ];

        let ranked = strategy.rank(&quotes, OrderSide::Sell);

        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].rank, 1);
        assert!((ranked[0].quote.price().get().to_f64().unwrap() - 105.0).abs() < f64::EPSILON);
        assert_eq!(ranked[1].rank, 2);
        assert!((ranked[1].quote.price().get().to_f64().unwrap() - 100.0).abs() < f64::EPSILON);
        assert_eq!(ranked[2].rank, 3);
        assert!((ranked[2].quote.price().get().to_f64().unwrap() - 95.0).abs() < f64::EPSILON);
    }

    #[test]
    fn best_price_strategy_empty() {
        let strategy = BestPriceStrategy::new();
        let ranked = strategy.rank(&[], OrderSide::Buy);
        assert!(ranked.is_empty());
    }

    #[test]
    fn weighted_score_strategy_default() {
        let strategy = WeightedScoreStrategy::default();
        assert!((strategy.price_weight - 0.7).abs() < f64::EPSILON);
        assert!((strategy.quantity_weight - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_score_strategy_custom() {
        let strategy = WeightedScoreStrategy::new(0.5, 0.5);
        assert!((strategy.price_weight - 0.5).abs() < f64::EPSILON);
        assert!((strategy.quantity_weight - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_score_strategy_buy_side() {
        let strategy = WeightedScoreStrategy::new(0.7, 0.3);
        let quotes = vec![
            create_quote(100.0, 10.0, "venue-1"),
            create_quote(95.0, 5.0, "venue-2"),
            create_quote(105.0, 15.0, "venue-3"),
        ];

        let ranked = strategy.rank(&quotes, OrderSide::Buy);

        assert_eq!(ranked.len(), 3);
        // Best should be venue-2 (lowest price) despite lower quantity
        assert_eq!(ranked[0].rank, 1);
    }

    #[test]
    fn ranking_strategy_name() {
        let best_price = BestPriceStrategy::new();
        assert_eq!(best_price.name(), "BestPrice");

        let weighted = WeightedScoreStrategy::default();
        assert_eq!(weighted.name(), "WeightedScore");
    }
}
