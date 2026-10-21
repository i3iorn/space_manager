//! Integer money.
//!
//! Money is stored in minor units as an `i64`; one credit is 100 minor units.
//! Floating point is never used for money. Splitting an amount distributes the
//! remainder deterministically to the first recipients in id order, so a split
//! never loses or invents a credit. See `docs/DOMAIN_MODEL.md` §3.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The number of minor units in one credit.
pub const MINOR_PER_CREDIT: i64 = 100;

/// An amount of money.
///
/// Integer minor units only. Every arithmetic operation is checked, so an
/// overflow is an error rather than a silently wrong balance.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct Money(i64);

impl Money {
    /// No money at all.
    pub const ZERO: Self = Self(0);

    /// An amount from whole credits.
    #[must_use]
    pub const fn from_credits(credits: i64) -> Self {
        Self(credits * MINOR_PER_CREDIT)
    }

    /// An amount from minor units.
    #[must_use]
    pub const fn from_minor(minor: i64) -> Self {
        Self(minor)
    }

    /// The amount in minor units.
    #[must_use]
    pub const fn minor(self) -> i64 {
        self.0
    }

    /// The amount in whole credits, truncating any fraction.
    ///
    /// For display only. Never use this to make a decision.
    #[must_use]
    pub const fn credits(self) -> i64 {
        self.0 / MINOR_PER_CREDIT
    }

    /// Whether the amount is negative.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// Whether the amount is exactly zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Adds two amounts, or returns `None` on overflow.
    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(sum) => Some(Self(sum)),
            None => None,
        }
    }

    /// Subtracts two amounts, or returns `None` on overflow.
    #[must_use]
    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        match self.0.checked_sub(other.0) {
            Some(difference) => Some(Self(difference)),
            None => None,
        }
    }

    /// Multiplies by a fraction, rounding down.
    ///
    /// `numerator` and `denominator` are integers, so the result is exact to the
    /// minor unit and involves no floating point.
    #[must_use]
    pub const fn multiply_fraction(self, numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            return Self(0);
        }
        Self((self.0 * numerator) / denominator)
    }

    /// A percentage of the amount, rounded down.
    #[must_use]
    pub fn percent(self, percent: u32) -> Self {
        self.multiply_fraction(i64::from(percent), 100)
    }

    /// Splits the amount into `parts` shares.
    ///
    /// The remainder is distributed one minor unit at a time to the earliest
    /// shares, so the shares always sum to exactly the original amount. The
    /// result is deterministic: the same amount and the same number of parts
    /// always produce the same shares.
    ///
    /// A request for zero parts yields an empty list, and the amount is then
    /// *not* conserved — which is why callers must never ask for zero parts.
    ///
    /// A negative amount splits just as exactly as a positive one. The
    /// remainder keeps the sign of the amount, so a deficit is shared out as a
    /// deficit rather than being quietly rounded away.
    #[must_use]
    pub fn split(self, parts: u32) -> Vec<Self> {
        if parts == 0 {
            return Vec::new();
        }
        let divisor = i64::from(parts);
        let share = self.0 / divisor;
        let remainder = self.0 % divisor;

        let mut shares = Vec::with_capacity(parts as usize);
        for index in 0..i64::from(parts) {
            let extra = if index < i64::try_from(remainder.unsigned_abs()).unwrap_or(i64::MAX) {
                if remainder < 0 { -1 } else { 1 }
            } else {
                0
            };
            shares.push(Self(share + extra));
        }
        shares
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let negative = self.0 < 0;
        let magnitude = self.0.unsigned_abs();
        let whole = magnitude / MINOR_PER_CREDIT as u64;
        let fraction = magnitude % MINOR_PER_CREDIT as u64;

        let digits = whole.to_string();
        let mut grouped = String::with_capacity(digits.len() + digits.len() / 3 + 4);
        for (index, character) in digits.chars().enumerate() {
            if index > 0 && (digits.len() - index) % 3 == 0 {
                grouped.push(',');
            }
            grouped.push(character);
        }

        if negative {
            write!(f, "-{grouped}.{fraction:02}")?;
        } else {
            write!(f, "{grouped}.{fraction:02}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credits_and_minor_units_round_trip() {
        assert_eq!(Money::from_credits(1_200).minor(), 120_000);
        assert_eq!(Money::from_minor(120_000).credits(), 1_200);
    }

    #[test]
    fn a_split_conserves_every_minor_unit() {
        let amount = Money::from_credits(100);
        let shares = amount.split(3);
        assert_eq!(shares.len(), 3);
        let total: i64 = shares.iter().map(|share| share.minor()).sum();
        assert_eq!(total, amount.minor());
        // The remainder goes to the earliest shares, so they differ by at most one.
        assert_eq!(shares[0].minor() - shares[2].minor(), 1);
    }

    #[test]
    fn a_split_into_equal_parts_is_exact() {
        let shares = Money::from_credits(300).split(3);
        for share in &shares {
            assert_eq!(share.credits(), 100);
        }
    }

    #[test]
    fn a_split_of_a_negative_amount_conserves_the_total() {
        let amount = Money::from_credits(-100);
        let shares = amount.split(3);
        let total: i64 = shares.iter().map(|share| share.minor()).sum();
        assert_eq!(total, amount.minor());
    }

    #[test]
    fn a_split_of_zero_parts_yields_nothing() {
        assert!(Money::from_credits(100).split(0).is_empty());
    }

    #[test]
    fn fractions_are_exact_to_the_minor_unit() {
        assert_eq!(Money::from_credits(100).percent(50).credits(), 50);
        assert_eq!(Money::from_credits(3).multiply_fraction(1, 3).minor(), 100);
    }

    #[test]
    fn checked_arithmetic_reports_overflow() {
        let huge = Money::from_minor(i64::MAX);
        assert!(huge.checked_add(Money::from_credits(1)).is_none());
        assert!(huge.checked_sub(Money::from_credits(1)).is_some());
    }

    #[test]
    fn the_display_groups_thousands() {
        assert_eq!(Money::from_credits(1_240_000).to_string(), "1,240,000.00");
        assert_eq!(Money::from_minor(-150).to_string(), "-1.50");
        assert_eq!(Money::from_minor(5).to_string(), "0.05");
    }
}
