//! Money types — the single source of truth for monetary values across AOS.
//!
//! Policy:
//! - Monetary amounts are ALWAYS represented by [`Money`], never `f32`/`f64`.
//! - Amounts are stored as `NUMERIC(19,4)` in PostgreSQL and `Decimal` in Rust.
//! - All arithmetic is checked; operations across currencies are rejected.
//! - Rounding is explicit via [`RoundingPolicy`]; there is no implicit rounding.
//! - The default scale is 4 decimal places (matches `NUMERIC(19,4)`).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use utoipa::ToSchema;

/// Default monetary scale matching `NUMERIC(19,4)` in PostgreSQL.
pub const DEFAULT_SCALE: u32 = 4;

/// Errors produced by money operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MoneyError {
    #[error("invalid currency code: {0}")]
    InvalidCurrency(String),
    #[error("invalid amount: {0}")]
    InvalidAmount(String),
    #[error("currency mismatch: {0} != {1}")]
    CurrencyMismatch(String, String),
    #[error("arithmetic overflow on amount: {0}")]
    Overflow(String),
}

/// Result type alias for money operations.
pub type MoneyResult<T> = Result<T, MoneyError>;

/// ISO 4217 currency code (three uppercase letters).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, example = "CDF")]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Create a currency code, validating it is exactly three uppercase ASCII letters.
    pub fn new(code: &str) -> MoneyResult<Self> {
        if code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase()) {
            Ok(Self(code.to_string()))
        } else {
            Err(MoneyError::InvalidCurrency(code.to_string()))
        }
    }

    /// The three-letter code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The Congolese Franc — the DRC base currency.
    pub fn cdf() -> Self {
        Self("CDF".to_string())
    }

    /// The United States Dollar.
    pub fn usd() -> Self {
        Self("USD".to_string())
    }

    /// The Euro.
    pub fn eur() -> Self {
        Self("EUR".to_string())
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Default for CurrencyCode {
    fn default() -> Self {
        Self::cdf()
    }
}

/// Explicit rounding strategy for monetary values.
///
/// AOS never performs implicit rounding — callers must state which strategy
/// applies, and payroll/ledger configs pin a strategy per calculation step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundingPolicy {
    /// Round half away from zero (common for payroll).
    RoundHalfUp,
    /// Round half towards zero.
    RoundHalfDown,
    /// Round half to the nearest even digit (banker's rounding).
    RoundHalfEven,
    /// Always round away from zero.
    RoundUp,
    /// Always round towards zero.
    RoundDown,
    /// Always round towards positive infinity.
    RoundCeiling,
    /// Always round towards negative infinity.
    RoundFloor,
}

impl RoundingPolicy {
    /// Apply the strategy, rounding to `DEFAULT_SCALE` decimal places.
    pub fn apply(self, amount: Decimal) -> Decimal {
        self.apply_to_scale(amount, DEFAULT_SCALE)
    }

    /// Apply the strategy, rounding to an explicit scale.
    pub fn apply_to_scale(self, amount: Decimal, scale: u32) -> Decimal {
        use rust_decimal::RoundingStrategy;
        let strategy = match self {
            RoundingPolicy::RoundHalfUp => RoundingStrategy::MidpointAwayFromZero,
            RoundingPolicy::RoundHalfDown => RoundingStrategy::MidpointTowardZero,
            RoundingPolicy::RoundHalfEven => RoundingStrategy::MidpointNearestEven,
            RoundingPolicy::RoundUp => RoundingStrategy::AwayFromZero,
            RoundingPolicy::RoundDown => RoundingStrategy::ToZero,
            RoundingPolicy::RoundCeiling => RoundingStrategy::ToPositiveInfinity,
            RoundingPolicy::RoundFloor => RoundingStrategy::ToNegativeInfinity,
        };
        amount.round_dp_with_strategy(scale, strategy)
    }
}

/// A monetary value with an associated currency.
///
/// `amount` is serialized as a decimal string in JSON (never a float) and maps
/// to `NUMERIC(19,4)` in PostgreSQL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Money {
    #[schema(value_type = String, example = "125000.0000")]
    pub amount: Decimal,
    pub currency: CurrencyCode,
}

impl Money {
    /// Create a money value from a `Decimal` amount.
    pub fn new(amount: Decimal, currency: CurrencyCode) -> Self {
        Self { amount, currency }
    }

    /// Create a money value from a decimal string (e.g. `"125000.50"`).
    pub fn from_str_amount(amount: &str, currency: CurrencyCode) -> MoneyResult<Self> {
        let parsed = Decimal::from_str(amount)
            .map_err(|_| MoneyError::InvalidAmount(format!("'{amount}' is not a valid decimal")))?;
        Ok(Self {
            amount: parsed,
            currency,
        })
    }

    /// Create a money value from an integer amount.
    pub fn from_i64(amount: i64, currency: CurrencyCode) -> Self {
        Self {
            amount: Decimal::from(amount),
            currency,
        }
    }

    /// Zero in the given currency.
    pub fn zero(currency: CurrencyCode) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    /// True when the amount is zero.
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    /// True when the amount is negative.
    pub fn is_negative(&self) -> bool {
        self.amount.is_sign_negative()
    }

    /// The underlying decimal amount.
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// The currency of this value.
    pub fn currency(&self) -> &CurrencyCode {
        &self.currency
    }

    /// Add two money values; errors if the currencies differ.
    pub fn checked_add(&self, rhs: &Self) -> MoneyResult<Self> {
        self.require_same_currency(rhs)?;
        let sum = self
            .amount
            .checked_add(rhs.amount)
            .ok_or_else(|| MoneyError::Overflow(self.amount.to_string()))?;
        Ok(Self {
            amount: sum,
            currency: self.currency.clone(),
        })
    }

    /// Subtract two money values; errors if the currencies differ.
    pub fn checked_sub(&self, rhs: &Self) -> MoneyResult<Self> {
        self.require_same_currency(rhs)?;
        let diff = self
            .amount
            .checked_sub(rhs.amount)
            .ok_or_else(|| MoneyError::Overflow(self.amount.to_string()))?;
        Ok(Self {
            amount: diff,
            currency: self.currency.clone(),
        })
    }

    /// Negate the amount, preserving currency. Negation of a decimal is a sign
    /// flip and cannot overflow, so it is infallible.
    pub fn checked_neg(&self) -> MoneyResult<Self> {
        Ok(Self {
            amount: -self.amount,
            currency: self.currency.clone(),
        })
    }

    /// Multiply by a decimal factor (e.g. a rate or proration fraction).
    pub fn checked_mul(&self, factor: Decimal) -> MoneyResult<Self> {
        let product = self
            .amount
            .checked_mul(factor)
            .ok_or_else(|| MoneyError::Overflow(self.amount.to_string()))?;
        Ok(Self {
            amount: product,
            currency: self.currency.clone(),
        })
    }

    /// The absolute value, preserving currency.
    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            currency: self.currency.clone(),
        }
    }

    /// Apply an explicit rounding strategy at the default scale.
    pub fn rounded(&self, policy: RoundingPolicy) -> Self {
        Self {
            amount: policy.apply(self.amount),
            currency: self.currency.clone(),
        }
    }

    /// Apply an explicit rounding strategy at an explicit scale.
    pub fn rounded_to_scale(&self, policy: RoundingPolicy, scale: u32) -> Self {
        Self {
            amount: policy.apply_to_scale(self.amount, scale),
            currency: self.currency.clone(),
        }
    }

    /// Canonical string form, e.g. `"125000.5000 CDF"`.
    pub fn to_canonical_string(&self) -> String {
        format!("{} {}", self.amount, self.currency)
    }

    fn require_same_currency(&self, rhs: &Self) -> MoneyResult<()> {
        if self.currency == rhs.currency {
            Ok(())
        } else {
            Err(MoneyError::CurrencyMismatch(
                self.currency.to_string(),
                rhs.currency.to_string(),
            ))
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

/// Convenience macro: `money!("125000.50", "CDF")`.
#[macro_export]
macro_rules! money {
    ($amount:expr, $currency:expr) => {
        $crate::money::Money::from_str_amount(
            $amount,
            $crate::money::CurrencyCode::new($currency)
                .expect("currency must be a valid ISO 4217 code"),
        )
        .expect("amount must be a valid decimal string")
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_code_validates() {
        assert!(CurrencyCode::new("CDF").is_ok());
        assert!(CurrencyCode::new("usd").is_err());
        assert!(CurrencyCode::new("US").is_err());
        assert!(CurrencyCode::new("USDD").is_err());
    }

    #[test]
    fn add_requires_same_currency() {
        let a = money!("100.00", "CDF");
        let b = money!("50.00", "CDF");
        let c = money!("10.00", "USD");
        assert_eq!(a.checked_add(&b).unwrap(), money!("150.00", "CDF"));
        assert!(a.checked_add(&c).is_err());
    }

    #[test]
    fn subtraction_round_trips() {
        let a = money!("100.00", "CDF");
        let b = money!("40.50", "CDF");
        let diff = a.checked_sub(&b).unwrap();
        assert_eq!(diff, money!("59.50", "CDF"));
    }

    #[test]
    fn rounding_half_up() {
        let m = money!("125000.00005", "CDF");
        let r = m.rounded(RoundingPolicy::RoundHalfUp);
        assert_eq!(r.amount.to_string(), "125000.0001");
    }

    #[test]
    fn rounding_floor() {
        let m = money!("125000.00009", "CDF");
        let r = m.rounded(RoundingPolicy::RoundFloor);
        assert_eq!(r.amount.to_string(), "125000.0000");
    }

    #[test]
    fn checked_mul_preserves_currency() {
        let m = money!("1000.00", "CDF");
        let doubled = m.checked_mul(Decimal::from(2)).unwrap();
        assert_eq!(doubled, money!("2000.00", "CDF"));
    }

    #[test]
    fn negative_amounts() {
        let m = money!("-10.00", "CDF");
        assert!(m.is_negative());
        assert_eq!(m.abs(), money!("10.00", "CDF"));
        assert_eq!(m.checked_neg().unwrap(), money!("10.00", "CDF"));
    }
}
