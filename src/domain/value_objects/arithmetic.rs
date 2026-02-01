//! # Checked Arithmetic
//!
//! Traits and utilities for safe arithmetic operations.
//!
//! This module provides:
//! - [`ArithmeticError`] - Error type for arithmetic failures
//! - [`CheckedArithmetic`] - Trait for safe arithmetic operations
//! - [`Rounding`] - Enum for explicit rounding direction
//! - [`div_round`] - Helper function for division with rounding
//!
//! # Examples
//!
//! ```
//! use otc_rfq::domain::value_objects::arithmetic::{CheckedArithmetic, ArithmeticError};
//! use rust_decimal::Decimal;
//!
//! let a = Decimal::new(100, 0);
//! let b = Decimal::new(3, 0);
//! let result = a.safe_div(b);
//! assert!(result.is_ok());
//! ```

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Error type for arithmetic operations.
///
/// Represents failures that can occur during checked arithmetic,
/// including overflow, underflow, division by zero, and invalid values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum ArithmeticError {
    /// Arithmetic operation resulted in overflow.
    #[error("arithmetic overflow")]
    Overflow,

    /// Arithmetic operation resulted in underflow.
    #[error("arithmetic underflow")]
    Underflow,

    /// Division by zero attempted.
    #[error("division by zero")]
    DivisionByZero,

    /// Invalid value provided (e.g., negative when positive required).
    #[error("invalid value: {0}")]
    InvalidValue(&'static str),
}

/// Result type for arithmetic operations.
pub type ArithmeticResult<T> = Result<T, ArithmeticError>;

/// Rounding direction for division operations.
///
/// Used to explicitly specify how remainders should be handled
/// in division operations.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::arithmetic::{Rounding, div_round};
/// use rust_decimal::Decimal;
///
/// let numerator = Decimal::new(10, 0);
/// let denominator = Decimal::new(3, 0);
///
/// // Round down: 10 / 3 = 3
/// let down = div_round(numerator, denominator, Rounding::Down).unwrap();
/// assert_eq!(down, Decimal::new(3, 0));
///
/// // Round up: 10 / 3 = 4
/// let up = div_round(numerator, denominator, Rounding::Up).unwrap();
/// assert_eq!(up, Decimal::new(4, 0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rounding {
    /// Round towards zero (truncate).
    Down,
    /// Round away from zero (ceiling for positive, floor for negative).
    Up,
}

impl fmt::Display for Rounding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Down => write!(f, "Down"),
            Self::Up => write!(f, "Up"),
        }
    }
}

/// Divide with explicit rounding direction.
///
/// Performs division and rounds the result according to the specified
/// rounding direction.
///
/// # Arguments
///
/// * `numerator` - The dividend
/// * `denominator` - The divisor
/// * `rounding` - The rounding direction to apply
///
/// # Returns
///
/// * `Ok(Decimal)` - The rounded result
/// * `Err(ArithmeticError::DivisionByZero)` - If denominator is zero
///
/// # Errors
///
/// Returns `ArithmeticError::DivisionByZero` if the denominator is zero.
///
/// # Examples
///
/// ```
/// use otc_rfq::domain::value_objects::arithmetic::{Rounding, div_round};
/// use rust_decimal::Decimal;
///
/// let result = div_round(
///     Decimal::new(7, 0),
///     Decimal::new(2, 0),
///     Rounding::Down
/// ).unwrap();
/// assert_eq!(result, Decimal::new(3, 0));
/// ```
#[inline]
#[must_use = "this returns the result of the operation, without modifying the original"]
pub fn div_round(
    numerator: Decimal,
    denominator: Decimal,
    rounding: Rounding,
) -> ArithmeticResult<Decimal> {
    if denominator.is_zero() {
        return Err(ArithmeticError::DivisionByZero);
    }

    let quotient = numerator / denominator;

    match rounding {
        Rounding::Down => Ok(quotient.trunc()),
        Rounding::Up => {
            let truncated = quotient.trunc();
            if quotient == truncated {
                Ok(truncated)
            } else if quotient.is_sign_positive() {
                Ok(truncated + Decimal::ONE)
            } else {
                Ok(truncated - Decimal::ONE)
            }
        }
    }
}

/// Trait for checked arithmetic operations.
///
/// Provides safe arithmetic methods that return `Result` instead of
/// panicking on overflow, underflow, or division by zero.
///
/// # Implementation Notes
///
/// Implementors should ensure that:
/// - No operation panics
/// - Overflow returns `Err(ArithmeticError::Overflow)`
/// - Underflow returns `Err(ArithmeticError::Underflow)`
/// - Division by zero returns `Err(ArithmeticError::DivisionByZero)`
pub trait CheckedArithmetic: Sized {
    /// Safely add two values.
    ///
    /// # Errors
    ///
    /// Returns `ArithmeticError::Overflow` if the result would overflow.
    fn safe_add(self, rhs: Self) -> ArithmeticResult<Self>;

    /// Safely subtract two values.
    ///
    /// # Errors
    ///
    /// Returns `ArithmeticError::Underflow` if the result would underflow.
    fn safe_sub(self, rhs: Self) -> ArithmeticResult<Self>;

    /// Safely multiply two values.
    ///
    /// # Errors
    ///
    /// Returns `ArithmeticError::Overflow` if the result would overflow.
    fn safe_mul(self, rhs: Self) -> ArithmeticResult<Self>;

    /// Safely divide two values.
    ///
    /// # Errors
    ///
    /// Returns `ArithmeticError::DivisionByZero` if the divisor is zero.
    fn safe_div(self, rhs: Self) -> ArithmeticResult<Self>;
}

impl CheckedArithmetic for Decimal {
    #[inline]
    fn safe_add(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_add(rhs).ok_or(ArithmeticError::Overflow)
    }

    #[inline]
    fn safe_sub(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_sub(rhs).ok_or(ArithmeticError::Underflow)
    }

    #[inline]
    fn safe_mul(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_mul(rhs).ok_or(ArithmeticError::Overflow)
    }

    #[inline]
    fn safe_div(self, rhs: Self) -> ArithmeticResult<Self> {
        if rhs.is_zero() {
            return Err(ArithmeticError::DivisionByZero);
        }
        self.checked_div(rhs).ok_or(ArithmeticError::Overflow)
    }
}

impl CheckedArithmetic for u64 {
    #[inline]
    fn safe_add(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_add(rhs).ok_or(ArithmeticError::Overflow)
    }

    #[inline]
    fn safe_sub(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_sub(rhs).ok_or(ArithmeticError::Underflow)
    }

    #[inline]
    fn safe_mul(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_mul(rhs).ok_or(ArithmeticError::Overflow)
    }

    #[inline]
    fn safe_div(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_div(rhs).ok_or(ArithmeticError::DivisionByZero)
    }
}

impl CheckedArithmetic for i64 {
    #[inline]
    fn safe_add(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_add(rhs).ok_or(ArithmeticError::Overflow)
    }

    #[inline]
    fn safe_sub(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_sub(rhs).ok_or(ArithmeticError::Underflow)
    }

    #[inline]
    fn safe_mul(self, rhs: Self) -> ArithmeticResult<Self> {
        self.checked_mul(rhs).ok_or(ArithmeticError::Overflow)
    }

    #[inline]
    fn safe_div(self, rhs: Self) -> ArithmeticResult<Self> {
        if rhs == 0 {
            return Err(ArithmeticError::DivisionByZero);
        }
        self.checked_div(rhs).ok_or(ArithmeticError::Overflow)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    mod arithmetic_error {
        use super::*;

        #[test]
        fn display_formats_correctly() {
            assert_eq!(ArithmeticError::Overflow.to_string(), "arithmetic overflow");
            assert_eq!(
                ArithmeticError::Underflow.to_string(),
                "arithmetic underflow"
            );
            assert_eq!(
                ArithmeticError::DivisionByZero.to_string(),
                "division by zero"
            );
            assert_eq!(
                ArithmeticError::InvalidValue("negative").to_string(),
                "invalid value: negative"
            );
        }
    }

    mod rounding {
        use super::*;

        #[test]
        fn display_formats_correctly() {
            assert_eq!(Rounding::Down.to_string(), "Down");
            assert_eq!(Rounding::Up.to_string(), "Up");
        }

        #[test]
        fn serde_roundtrip() {
            let down = Rounding::Down;
            let json = serde_json::to_string(&down).unwrap();
            let deserialized: Rounding = serde_json::from_str(&json).unwrap();
            assert_eq!(down, deserialized);
        }
    }

    mod div_round_tests {
        use super::*;

        #[test]
        fn div_round_down_truncates() {
            let result =
                div_round(Decimal::new(10, 0), Decimal::new(3, 0), Rounding::Down).unwrap();
            assert_eq!(result, Decimal::new(3, 0));
        }

        #[test]
        fn div_round_up_rounds_up() {
            let result = div_round(Decimal::new(10, 0), Decimal::new(3, 0), Rounding::Up).unwrap();
            assert_eq!(result, Decimal::new(4, 0));
        }

        #[test]
        fn div_round_exact_no_rounding() {
            let down = div_round(Decimal::new(10, 0), Decimal::new(2, 0), Rounding::Down).unwrap();
            let up = div_round(Decimal::new(10, 0), Decimal::new(2, 0), Rounding::Up).unwrap();
            assert_eq!(down, Decimal::new(5, 0));
            assert_eq!(up, Decimal::new(5, 0));
        }

        #[test]
        fn div_round_by_zero_fails() {
            let result = div_round(Decimal::new(10, 0), Decimal::ZERO, Rounding::Down);
            assert_eq!(result, Err(ArithmeticError::DivisionByZero));
        }

        #[test]
        fn div_round_negative_down() {
            let result =
                div_round(Decimal::new(-10, 0), Decimal::new(3, 0), Rounding::Down).unwrap();
            assert_eq!(result, Decimal::new(-3, 0));
        }

        #[test]
        fn div_round_negative_up() {
            let result = div_round(Decimal::new(-10, 0), Decimal::new(3, 0), Rounding::Up).unwrap();
            assert_eq!(result, Decimal::new(-4, 0));
        }
    }

    mod checked_arithmetic_decimal {
        use super::*;

        #[test]
        fn safe_add_works() {
            let a = Decimal::new(100, 0);
            let b = Decimal::new(50, 0);
            assert_eq!(a.safe_add(b).unwrap(), Decimal::new(150, 0));
        }

        #[test]
        fn safe_sub_works() {
            let a = Decimal::new(100, 0);
            let b = Decimal::new(50, 0);
            assert_eq!(a.safe_sub(b).unwrap(), Decimal::new(50, 0));
        }

        #[test]
        fn safe_mul_works() {
            let a = Decimal::new(10, 0);
            let b = Decimal::new(5, 0);
            assert_eq!(a.safe_mul(b).unwrap(), Decimal::new(50, 0));
        }

        #[test]
        fn safe_div_works() {
            let a = Decimal::new(100, 0);
            let b = Decimal::new(5, 0);
            assert_eq!(a.safe_div(b).unwrap(), Decimal::new(20, 0));
        }

        #[test]
        fn safe_div_by_zero_fails() {
            let a = Decimal::new(100, 0);
            let b = Decimal::ZERO;
            assert_eq!(a.safe_div(b), Err(ArithmeticError::DivisionByZero));
        }
    }

    mod checked_arithmetic_u64 {
        use super::*;

        #[test]
        fn safe_add_works() {
            assert_eq!(100u64.safe_add(50).unwrap(), 150);
        }

        #[test]
        fn safe_add_overflow_fails() {
            assert_eq!(u64::MAX.safe_add(1), Err(ArithmeticError::Overflow));
        }

        #[test]
        fn safe_sub_works() {
            assert_eq!(100u64.safe_sub(50).unwrap(), 50);
        }

        #[test]
        fn safe_sub_underflow_fails() {
            assert_eq!(0u64.safe_sub(1), Err(ArithmeticError::Underflow));
        }

        #[test]
        fn safe_mul_works() {
            assert_eq!(10u64.safe_mul(5).unwrap(), 50);
        }

        #[test]
        fn safe_mul_overflow_fails() {
            assert_eq!(u64::MAX.safe_mul(2), Err(ArithmeticError::Overflow));
        }

        #[test]
        fn safe_div_works() {
            assert_eq!(100u64.safe_div(5).unwrap(), 20);
        }

        #[test]
        fn safe_div_by_zero_fails() {
            assert_eq!(100u64.safe_div(0), Err(ArithmeticError::DivisionByZero));
        }
    }

    mod checked_arithmetic_i64 {
        use super::*;

        #[test]
        fn safe_add_works() {
            assert_eq!(100i64.safe_add(50).unwrap(), 150);
        }

        #[test]
        fn safe_add_overflow_fails() {
            assert_eq!(i64::MAX.safe_add(1), Err(ArithmeticError::Overflow));
        }

        #[test]
        fn safe_sub_works() {
            assert_eq!(100i64.safe_sub(50).unwrap(), 50);
        }

        #[test]
        fn safe_sub_underflow_fails() {
            assert_eq!(i64::MIN.safe_sub(1), Err(ArithmeticError::Underflow));
        }

        #[test]
        fn safe_div_by_zero_fails() {
            assert_eq!(100i64.safe_div(0), Err(ArithmeticError::DivisionByZero));
        }
    }
}
