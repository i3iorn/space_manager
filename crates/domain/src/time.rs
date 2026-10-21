//! The clock.
//!
//! Simulated time is an **integer number of milliseconds since the campaign
//! began**. There is no floating-point time, and the simulation never reads the
//! wall clock — it is told how much simulated time has passed.
//!
//! A civil calendar is **derived** from the instant, for display and for the
//! periodic triggers that still have a rhythm. The calendar is a projection, not
//! a stored value: nothing in the campaign state holds a month.
//!
//! See `docs/DOMAIN_MODEL.md` §2.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Milliseconds in a second.
pub const MILLIS_PER_SECOND: u64 = 1_000;

/// Milliseconds in a minute.
pub const MILLIS_PER_MINUTE: u64 = 60 * MILLIS_PER_SECOND;

/// Milliseconds in an hour.
pub const MILLIS_PER_HOUR: u64 = 60 * MILLIS_PER_MINUTE;

/// Milliseconds in a day.
pub const MILLIS_PER_DAY: u64 = 24 * MILLIS_PER_HOUR;

/// Days in a month.
pub const DAYS_PER_MONTH: u64 = 30;

/// Milliseconds in a month.
pub const MILLIS_PER_MONTH: u64 = DAYS_PER_MONTH * MILLIS_PER_DAY;

/// Milliseconds in a month, signed.
///
/// The fixed-point arithmetic in `domain::scale` is signed, and a month is
/// 2,592,000,000 milliseconds — nowhere near `i64::MAX`. The cast is written
/// down once, here, rather than scattered through the arithmetic where it
/// could quietly wrap.
#[allow(clippy::cast_possible_wrap)]
pub const MILLIS_PER_MONTH_I64: i64 = MILLIS_PER_MONTH as i64;

/// Months in a year.
pub const MONTHS_PER_YEAR: u64 = 12;

/// Milliseconds in a year.
pub const MILLIS_PER_YEAR: u64 = MONTHS_PER_YEAR * MILLIS_PER_MONTH;

/// A point in the campaign.
///
/// Integer milliseconds since the campaign began. Monotonic, and never read
/// from a clock.
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct Instant(pub u64);

impl Instant {
    /// The instant the campaign begins.
    pub const ZERO: Self = Self(0);

    /// The instant, in milliseconds since the campaign began.
    #[must_use]
    pub const fn millis(self) -> u64 {
        self.0
    }

    /// The instant this many seconds after the campaign began.
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs * MILLIS_PER_SECOND)
    }

    /// The instant this many minutes after the campaign began.
    #[must_use]
    pub const fn from_mins(mins: u64) -> Self {
        Self(mins * MILLIS_PER_MINUTE)
    }

    /// The instant this many hours after the campaign began.
    #[must_use]
    pub const fn from_hours(hours: u64) -> Self {
        Self(hours * MILLIS_PER_HOUR)
    }

    /// The instant this many days after the campaign began.
    #[must_use]
    pub const fn from_days(days: u64) -> Self {
        Self(days * MILLIS_PER_DAY)
    }

    /// The instant this many months after the campaign began.
    #[must_use]
    pub const fn from_months(months: u64) -> Self {
        Self(months * MILLIS_PER_MONTH)
    }

    /// The instant this many years after the campaign began.
    #[must_use]
    pub const fn from_years(years: u64) -> Self {
        Self(years * MILLIS_PER_YEAR)
    }

    /// The instant this many milliseconds later, if it does not overflow.
    #[must_use]
    pub const fn checked_add(self, duration: Duration) -> Option<Self> {
        match self.0.checked_add(duration.0) {
            Some(millis) => Some(Self(millis)),
            None => None,
        }
    }

    /// The instant this many milliseconds earlier, if it does not underflow.
    #[must_use]
    pub const fn checked_sub(self, duration: Duration) -> Option<Self> {
        match self.0.checked_sub(duration.0) {
            Some(millis) => Some(Self(millis)),
            None => None,
        }
    }

    /// The instant this many milliseconds later, saturating at the maximum.
    #[must_use]
    pub const fn saturating_add(self, duration: Duration) -> Self {
        Self(self.0.saturating_add(duration.0))
    }

    /// The instant this many milliseconds earlier, saturating at zero.
    #[must_use]
    pub const fn saturating_sub(self, duration: Duration) -> Self {
        Self(self.0.saturating_sub(duration.0))
    }

    /// How long ago this instant was, saturating at zero.
    #[must_use]
    pub const fn duration_since(self, earlier: Self) -> Duration {
        Duration(self.0.saturating_sub(earlier.0))
    }

    /// The whole days since the campaign began.
    #[must_use]
    pub const fn days(self) -> u64 {
        self.0 / MILLIS_PER_DAY
    }

    /// The whole months since the campaign began.
    ///
    /// **This is a stable identifier, and the instants are not.** The month
    /// index survives a change to the length of a month, and it is what the
    /// random streams are keyed on — so a change to the calendar does not
    /// change every draw in the campaign.
    #[must_use]
    pub const fn months(self) -> u64 {
        self.0 / MILLIS_PER_MONTH
    }
}

impl fmt::Display for Instant {
    /// **This is the raw count of milliseconds, and it is not a date.**
    ///
    /// An instant does not know when the campaign began, so it cannot say what
    /// year it is. The interface has to project it onto the civil calendar
    /// through `Calendar::date`, and every screen that shows a date goes
    /// through the interface's `App::date` for exactly that reason.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A length of simulated time.
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct Duration(pub u64);

impl Duration {
    /// No time at all.
    pub const ZERO: Self = Self(0);

    /// The length, in milliseconds.
    #[must_use]
    pub const fn millis(self) -> u64 {
        self.0
    }

    /// This many milliseconds.
    #[must_use]
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// This many seconds.
    #[must_use]
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs * MILLIS_PER_SECOND)
    }

    /// This many minutes.
    #[must_use]
    pub const fn from_mins(mins: u64) -> Self {
        Self(mins * MILLIS_PER_MINUTE)
    }

    /// This many hours.
    #[must_use]
    pub const fn from_hours(hours: u64) -> Self {
        Self(hours * MILLIS_PER_HOUR)
    }

    /// This many days.
    #[must_use]
    pub const fn from_days(days: u64) -> Self {
        Self(days * MILLIS_PER_DAY)
    }

    /// This many months.
    #[must_use]
    pub const fn from_months(months: u64) -> Self {
        Self(months * MILLIS_PER_MONTH)
    }

    /// This many years.
    #[must_use]
    pub const fn from_years(years: u64) -> Self {
        Self(years * MILLIS_PER_YEAR)
    }

    /// Whether this is no time at all.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Twice this duration.
    #[must_use]
    pub const fn doubled(self) -> Self {
        Self(self.0.saturating_mul(2))
    }

    /// This duration divided by a whole number, rounded down.
    ///
    /// A divisor of zero yields no time rather than a panic.
    #[must_use]
    pub fn divided_by(self, divisor: u64) -> Self {
        Self(self.0.checked_div(divisor).unwrap_or(0))
    }

    /// The number of whole months in this duration.
    #[must_use]
    pub const fn whole_months(self) -> u64 {
        self.0 / MILLIS_PER_MONTH
    }
}

impl fmt::Display for Duration {
    /// **The largest unit that divides it exactly**, because "18 months" is
    /// what a player reads and "540 days" is what a computer reads.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let millis = self.0;
        if millis < MILLIS_PER_MINUTE {
            write!(f, "{millis}ms")
        } else if millis < MILLIS_PER_HOUR {
            write!(f, "{}m", millis / MILLIS_PER_MINUTE)
        } else if millis < MILLIS_PER_DAY {
            write!(f, "{}h", millis / MILLIS_PER_HOUR)
        } else if millis % MILLIS_PER_MONTH == 0 {
            write!(f, "{} months", millis / MILLIS_PER_MONTH)
        } else if millis % MILLIS_PER_DAY == 0 {
            write!(f, "{} days", millis / MILLIS_PER_DAY)
        } else {
            write!(f, "{}d", millis / MILLIS_PER_DAY)
        }
    }
}

/// A readable point in the campaign.
///
/// **A projection, never stored.** Every `Date` in the game is computed from an
/// `Instant` and the year the agency was founded.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Date {
    /// The year.
    pub year: i32,
    /// The month within the year, 1-12.
    pub month: u32,
    /// The day within the month, 1-30.
    pub day: u32,
    /// The hour within the day, 0-23.
    pub hour: u32,
    /// The minute within the hour, 0-59.
    pub minute: u32,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }
}

/// The civil calendar, derived from the instant.
///
/// This is a **projection**, not state. It is constructed on demand from the
/// year the agency was founded.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Calendar {
    /// The year the campaign began.
    pub year_of_origin: i32,
}

impl Calendar {
    /// A calendar whose year zero is the given year.
    #[must_use]
    pub const fn new(year_of_origin: i32) -> Self {
        Self { year_of_origin }
    }

    /// The readable date of an instant.
    #[must_use]
    pub fn date(&self, instant: Instant) -> Date {
        let millis = instant.millis();
        let day_index = millis / MILLIS_PER_DAY;
        let millis_into_day = millis % MILLIS_PER_DAY;

        let year_index = day_index / (DAYS_PER_MONTH * MONTHS_PER_YEAR);
        let day_of_year = day_index % (DAYS_PER_MONTH * MONTHS_PER_YEAR);

        Date {
            year: self.year_of_origin + i32::try_from(year_index).unwrap_or(i32::MAX),
            month: (day_of_year / DAYS_PER_MONTH) as u32 + 1,
            day: (day_of_year % DAYS_PER_MONTH) as u32 + 1,
            hour: (millis_into_day / MILLIS_PER_HOUR) as u32,
            minute: ((millis_into_day % MILLIS_PER_HOUR) / MILLIS_PER_MINUTE) as u32,
        }
    }

    /// The fiscal year an instant falls in.
    #[must_use]
    pub fn fiscal_year(&self, instant: Instant) -> i32 {
        let year_index = instant.days() / (DAYS_PER_MONTH * MONTHS_PER_YEAR);
        self.year_of_origin + i32::try_from(year_index).unwrap_or(i32::MAX)
    }
}

/// How fast simulated time passes.
///
/// **This is a host-side control, not domain state.** The simulation is never
/// told which one is in force; the host converts real elapsed time into
/// simulated elapsed time and hands that over. That is what makes dilation
/// independence hold — a campaign played at 1x and at 10,000x is the same
/// campaign.
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
pub enum TimeScale {
    /// Sixty simulated seconds per real second. **A minute a second.**
    ///
    /// **This is the base rate, and there is nothing slower.** The game is a
    /// running agency: a campaign is thirty to sixty years, and at one minute a
    /// second that is a long evening of watching rather than a lifetime. The
    /// steps above it are for when the player wants to see the years go by.
    #[default]
    Sixty,
    /// Three hundred simulated seconds per real second.
    ThreeHundred,
    /// Six hundred simulated seconds per real second.
    SixHundred,
    /// Three thousand simulated seconds per real second.
    ThreeThousand,
    /// Six thousand simulated seconds per real second.
    SixThousand,
    /// Thirty thousand simulated seconds per real second.
    ThirtyThousand,
    /// Sixty thousand simulated seconds per real second.
    SixtyThousand,
    /// Three hundred thousand simulated seconds per real second.
    ThreeHundredThousand,
    /// Six hundred thousand simulated seconds per real second.
    SixHundredThousand,
}

impl TimeScale {
    /// Every dilation, from slowest to fastest.
    pub const ALL: [Self; 9] = [
        Self::Sixty,
        Self::ThreeHundred,
        Self::SixHundred,
        Self::ThreeThousand,
        Self::SixThousand,
        Self::ThirtyThousand,
        Self::SixtyThousand,
        Self::ThreeHundredThousand,
        Self::SixHundredThousand,
    ];

    /// The number of simulated seconds per real second.
    #[must_use]
    pub const fn multiplier(self) -> u64 {
        match self {
            Self::Sixty => 60,
            Self::ThreeHundred => 300,
            Self::SixHundred => 600,
            Self::ThreeThousand => 3_000,
            Self::SixThousand => 6_000,
            Self::ThirtyThousand => 30_000,
            Self::SixtyThousand => 60_000,
            Self::ThreeHundredThousand => 300_000,
            Self::SixHundredThousand => 600_000,
        }
    }

    /// The dilation a number key selects.
    #[must_use]
    pub const fn from_key(key: u8) -> Option<Self> {
        match key {
            1 => Some(Self::Sixty),
            2 => Some(Self::ThreeHundred),
            3 => Some(Self::SixHundred),
            4 => Some(Self::ThreeThousand),
            5 => Some(Self::SixThousand),
            6 => Some(Self::ThirtyThousand),
            7 => Some(Self::SixtyThousand),
            8 => Some(Self::ThreeHundredThousand),
            9 => Some(Self::SixHundredThousand),
            _ => None,
        }
    }

    /// The next dilation up.
    #[must_use]
    pub fn faster(self) -> Self {
        let index = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(index + 1).min(Self::ALL.len() - 1)]
    }

    /// The next dilation down.
    #[must_use]
    pub fn slower(self) -> Self {
        let index = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[index.saturating_sub(1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instant_is_a_count_of_milliseconds() {
        let instant = Instant::ZERO;
        assert_eq!(instant.millis(), 0);
        assert_eq!(
            instant.checked_add(Duration::from_secs(1)),
            Some(Instant(1_000))
        );
    }

    #[test]
    fn an_instant_cannot_go_backwards() {
        let instant = Instant(500);
        assert_eq!(instant.checked_sub(Duration::from_secs(1)), None);
        assert_eq!(instant.duration_since(Instant(1_000)), Duration::ZERO);
    }

    #[test]
    fn the_calendar_projects_a_date_from_an_instant() {
        let calendar = Calendar::new(2039);
        assert_eq!(calendar.date(Instant::ZERO).to_string(), "2039-01-01 00:00");
        assert_eq!(
            calendar.date(Instant(MILLIS_PER_MONTH)).to_string(),
            "2039-02-01 00:00"
        );
        assert_eq!(
            calendar
                .date(Instant(
                    MILLIS_PER_MONTH
                        + 13 * MILLIS_PER_DAY
                        + 9 * MILLIS_PER_HOUR
                        + 32 * MILLIS_PER_MINUTE
                ))
                .to_string(),
            "2039-02-14 09:32"
        );
    }

    #[test]
    fn the_calendar_turns_the_year_over_after_twelve_months() {
        let calendar = Calendar::new(2039);
        assert_eq!(
            calendar.date(Instant(MILLIS_PER_YEAR)).to_string(),
            "2040-01-01 00:00"
        );
        assert_eq!(calendar.fiscal_year(Instant(MILLIS_PER_YEAR)), 2040);
    }

    #[test]
    fn the_dilations_are_ordered_and_complete() {
        assert_eq!(TimeScale::ALL.len(), 9);
        assert_eq!(TimeScale::ALL[0], TimeScale::Sixty);
        assert_eq!(TimeScale::ALL[8], TimeScale::SixHundredThousand);
        for (index, scale) in TimeScale::ALL.iter().enumerate() {
            assert_eq!(scale.multiplier(), scale.multiplier());
            assert!(scale.multiplier() >= 1);
            if index > 0 {
                assert!(scale.multiplier() > TimeScale::ALL[index - 1].multiplier());
            }
        }
    }

    #[test]
    fn a_dilation_can_be_stepped_up_and_down() {
        assert_eq!(TimeScale::Sixty.faster(), TimeScale::ThreeHundred);
        assert_eq!(TimeScale::Sixty.slower(), TimeScale::Sixty);
        assert_eq!(
            TimeScale::SixHundredThousand.faster(),
            TimeScale::SixHundredThousand
        );
    }

    #[test]
    fn the_number_keys_select_the_dilations() {
        assert_eq!(TimeScale::from_key(1), Some(TimeScale::Sixty));
        assert_eq!(TimeScale::from_key(9), Some(TimeScale::SixHundredThousand));
        assert_eq!(TimeScale::from_key(0), None);
    }
}
