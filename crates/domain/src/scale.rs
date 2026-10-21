//! Fixed-point storage for continuous quantities.
//!
//! Fatigue accrues at about 0.008% an hour and a progress rate is a fraction of
//! a point a month. Truncating either to a whole percent loses the rate
//! entirely, and the visible symptom is that **nothing ever moves at 1x
//! dilation**.
//!
//! So the continuous *quantities* — fatigue, morale, health, progress — are
//! stored in **scaled integers**, at 256 units to the percent. The remainder is
//! carried in the value itself, so there is still one source of truth per
//! quantity and no separate accumulator to drift.
//!
//! *Ratios* are not scaled. The modifier scale (`-100..=+100`) and the
//! effective performance (basis points) are ratios, not quantities, and they
//! meet the scaled quantities at the point where a modifier is derived.
//!
//! See `docs/DOMAIN_MODEL.md` §2.3.

use serde::{Deserialize, Serialize};

use crate::time::Duration;

/// The number of fixed-point units in one percent.
///
/// Signed, so that the signed and unsigned conversions share one number and
/// cannot drift apart.
pub const PERCENT_UNITS: i32 = 256;

/// The scaled value of one hundred percent.
///
/// **A percent is `PERCENT_UNITS` scaled units, so a hundred percent is a
/// hundred of them.** This is the divisor for any ratio expressed against a
/// full load, and it is a separate constant because getting it wrong is
/// silent: a ratio divided by the wrong number is still a ratio, and it is
/// just wrong by a factor of a hundred.
pub const FULL_PERCENT: i32 = 100 * PERCENT_UNITS;

/// The fractional remainders carried between rate applications.
///
/// A rate applied over a fraction of its period truncates, and the truncation
/// accumulates. Over a thirty-year campaign at hourly quanta it would lose 136
/// percentage points of fatigue — and nothing would look broken.
///
/// The remainder carries the fraction forward, so the total is **exact** however
/// finely the campaign is integrated.
///
/// It lives in the state, because it is part of it: a save that dropped the
/// remainders would produce a different campaign from the one it was taken from.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Remainders {
    /// The remainder of the fatigue accrual.
    pub fatigue: i64,
    /// The remainder of the health recovery.
    pub health: i64,
    /// The remainder of the morale drift.
    pub morale: i64,
    /// The remainder of a project's progress.
    pub progress: i64,
    /// The remainder of a facility's build progress.
    pub construction: i64,
    /// The remainder of a facility's condition.
    pub condition: i64,
    /// The remainder of the money flow.
    pub money: i64,
    /// The remainder of the goodwill's regeneration.
    pub goodwill: i64,
}

impl Remainders {
    /// No remainder at all.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fatigue: 0,
            health: 0,
            morale: 0,
            progress: 0,
            construction: 0,
            condition: 0,
            money: 0,
            goodwill: 0,
        }
    }
}

/// Applies a rate over an elapsed duration, carrying the remainder exactly.
///
/// The rate is `numerator / denominator` scaled units per millisecond, and the
/// remainder is in the same units as the numerator, always less than
/// `denominator`.
///
/// **The denominator must be the same for every call that uses a given
/// remainder.** If it changes, the carried fraction means nothing, and the rate
/// is quietly wrong.
///
/// The total is exact. Summing the deltas telescopes:
///
/// ```text
/// sum(delta) = (sum(numerator x elapsed) - final remainder) / denominator
/// ```
///
/// so the total differs from the true value by less than one scaled unit, no
/// matter how finely the campaign is integrated.
///
/// # Examples
///
/// A rate of 6% a month applied over an hour is 2.133 units. Without the
/// remainder that would be 2 units an hour, and the campaign would lose 0.133
/// units every hour for the rest of its life — 136 percentage points of
/// fatigue over thirty years, and nothing would look broken.
#[must_use]
pub fn accrue(numerator: i64, denominator: i64, elapsed: Duration, remainder: &mut i64) -> i64 {
    let denominator = denominator.max(1);
    let elapsed_millis = i64::try_from(elapsed.millis()).unwrap_or(i64::MAX);
    let total = remainder.saturating_add(numerator.saturating_mul(elapsed_millis));
    *remainder = total.rem_euclid(denominator);
    total.div_euclid(denominator)
}

/// Converts a percentage into scaled units.
#[must_use]
pub const fn percent_to_units(percent: u32) -> u32 {
    percent * PERCENT_UNITS as u32
}

/// Converts scaled units back into a whole percentage, rounding down.
///
/// 256 units is one percent, so this is a plain division.
#[must_use]
pub const fn units_to_percent(units: u32) -> u32 {
    units / PERCENT_UNITS as u32
}

/// Converts a signed percentage into scaled units.
#[must_use]
pub const fn signed_percent_to_units(percent: i16) -> i16 {
    (percent as i32 * PERCENT_UNITS) as i16
}

/// Converts signed scaled units back into a whole percentage, rounding toward
/// zero.
#[must_use]
pub const fn signed_units_to_percent(units: i16) -> i16 {
    (units as i32 / PERCENT_UNITS) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::MILLIS_PER_MONTH;

    /// The length of a month, in milliseconds.
    fn month() -> i64 {
        i64::try_from(MILLIS_PER_MONTH).expect("a month fits in an i64")
    }

    #[test]
    fn a_percentage_round_trips() {
        for percent in [0, 1, 25, 50, 78, 100] {
            let units = percent_to_units(percent);
            assert_eq!(units_to_percent(units), percent);
        }
    }

    #[test]
    fn a_signed_percentage_round_trips() {
        for percent in [-100, -50, -25, 0, 30, 100] {
            let units = signed_percent_to_units(percent);
            assert_eq!(signed_units_to_percent(units), percent);
        }
    }

    #[test]
    fn a_fractional_percentage_is_not_lost() {
        // One percent in sixty: the resolution that a whole-percent delta would
        // throw away.
        let units = percent_to_units(1) / 60;
        assert!(units > 0);
    }

    #[test]
    fn the_signed_scale_fits_in_an_i16() {
        assert_eq!(signed_percent_to_units(100), 25_600);
        assert_eq!(signed_percent_to_units(-100), -25_600);
    }

    #[test]
    fn the_two_scales_agree() {
        assert_eq!(
            i32::try_from(percent_to_units(100)).unwrap_or(0),
            i32::from(signed_percent_to_units(100))
        );
    }

    #[test]
    fn a_rate_applied_over_a_month_is_exact() {
        let mut remainder = 0;
        assert_eq!(
            accrue(1_536, month(), Duration::from_months(1), &mut remainder),
            1_536
        );
        assert_eq!(remainder, 0);
    }

    #[test]
    fn a_rate_applied_in_fragments_totals_exactly() {
        // Six hundred and forty-eight hours is twenty-seven days, which is not a
        // whole month, and the fragments must add up to the same total as one
        // application.
        let mut whole = 0;
        let one_shot = accrue(1_536, month(), Duration::from_days(27), &mut whole);

        let mut pieces = 0;
        let mut remainder = 0;
        for _ in 0..648 {
            pieces += accrue(1_536, month(), Duration::from_hours(1), &mut remainder);
        }
        assert_eq!(pieces, one_shot);
    }

    #[test]
    fn the_remainder_carries_the_fraction() {
        let mut remainder = 0;
        // An hour of 6%-a-month fatigue is 2.133 units.
        let first = accrue(1_536, month(), Duration::from_hours(1), &mut remainder);
        assert_eq!(first, 2);
        assert!(remainder > 0, "the fraction must be carried forward");
    }

    #[test]
    fn a_negative_rate_works_too() {
        // Recovery is a negative rate, and the remainder must not go negative.
        let mut remainder = 0;
        let total = accrue(-2_560, month(), Duration::from_days(30), &mut remainder);
        assert_eq!(total, -2_560);
        assert!(remainder >= 0);
    }

    #[test]
    fn the_total_is_independent_of_how_it_is_split() {
        // The same rate, applied over the same span in wildly different pieces,
        // must give the same total. This is the property that lets the engine
        // choose its own integration quantum.
        let whole = {
            let mut remainder = 0;
            accrue(1_536, month(), Duration::from_days(30), &mut remainder)
        };
        let pieces = {
            let mut remainder = 0;
            let mut total = 0;
            for _ in 0..360 {
                total += accrue(1_536, month(), Duration::from_hours(2), &mut remainder);
            }
            total
        };
        assert_eq!(whole, pieces);
    }
}
