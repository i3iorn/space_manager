//! The state: approval, political capital, and promises.
//!
//! A promise reuses the `Commitment` type, because a promise to the state and a
//! deadline on a programme are the same kind of object and should fail the same
//! way. See `docs/DOMAIN_MODEL.md` §13.

use serde::{Deserialize, Serialize};

use crate::programme::{Commitment, DeadlineAudience};
use crate::scale::{signed_percent_to_units, signed_units_to_percent};
use crate::time::Instant;

/// How the state feels about the agency.
///
/// The value is a **scaled integer** at 256 units per percent, so that approval
/// can drift continuously without being truncated.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Approval {
    /// The state's trust, from `-25_600` to `+25_600`.
    pub value: i16,
}

impl Approval {
    /// The state is indifferent.
    #[must_use]
    pub const fn neutral() -> Self {
        Self { value: 0 }
    }

    /// The state is hostile.
    #[must_use]
    pub const fn hostile() -> Self {
        Self {
            value: signed_percent_to_units(-50),
        }
    }

    /// The state is behind the agency.
    #[must_use]
    pub const fn supportive() -> Self {
        Self {
            value: signed_percent_to_units(50),
        }
    }

    /// The approval as a whole percentage.
    #[must_use]
    pub const fn percent(&self) -> i16 {
        signed_units_to_percent(self.value)
    }
}

/// The institution's patience, and the state's willingness to accommodate the
/// director's whims.
///
/// **This is the anti-spamming mechanism, and it is deliberately not a cap.**
/// There is no limit on how many orders the player may give; the cost of
/// spamming is financial and political. See `MANAGEABLE_PARTS.md` §3.5.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Goodwill {
    /// The patience, from `-25_600` to `+25_600`.
    pub value: i16,
}

impl Goodwill {
    /// An ordinary relationship.
    #[must_use]
    pub const fn neutral() -> Self {
        Self { value: 0 }
    }

    /// The goodwill as a whole percentage.
    #[must_use]
    pub const fn percent(&self) -> i16 {
        signed_units_to_percent(self.value)
    }

    /// The goodwill after a charge, clamped to the scale.
    ///
    /// The charge is in whole points: a policy change costs five, a hire costs
    /// one.
    #[must_use]
    pub fn charged(self, points: i16) -> Self {
        let units = i64::from(points) * i64::from(crate::scale::PERCENT_UNITS);
        let value = i64::from(self.value).saturating_sub(units);
        let capped = value.clamp(
            i64::from(-crate::scale::signed_percent_to_units(100)),
            i64::from(crate::scale::signed_percent_to_units(100)),
        );
        Self {
            value: i16::try_from(capped).unwrap_or(0),
        }
    }

    /// The goodwill after a grant, clamped to the scale.
    #[must_use]
    pub fn granted(self, points: i16) -> Self {
        let units = i64::from(points) * i64::from(crate::scale::PERCENT_UNITS);
        let value = i64::from(self.value).saturating_add(units);
        let capped = value.clamp(
            i64::from(-crate::scale::signed_percent_to_units(100)),
            i64::from(crate::scale::signed_percent_to_units(100)),
        );
        Self {
            value: i16::try_from(capped).unwrap_or(0),
        }
    }

    /// Whether the state is accommodating.
    ///
    /// **Above this, approvals are fast** and the state grants favours.
    #[must_use]
    pub const fn is_accommodating(&self) -> bool {
        self.value > crate::scale::signed_percent_to_units(40)
    }

    /// Whether the state is obstructive.
    ///
    /// **Below this, every order takes longer to land** and the costs rise.
    #[must_use]
    pub const fn is_obstructive(&self) -> bool {
        self.value < crate::scale::signed_percent_to_units(-40)
    }
}

/// A promise the agency has made.
///
/// The state's memory is the mechanism behind the whole commitment model: a
/// broken promise costs nearly three times what a kept one rewards.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Promise {
    /// What was promised, in the player's language.
    pub description: String,
    /// The date it was promised for, and who was promised to.
    pub commitment: Commitment,
    /// Who the promise was made to.
    pub audience: DeadlineAudience,
    /// When the promise was made.
    pub made: Instant,
    /// Whether it was kept, if it has come due.
    pub kept: Option<bool>,
}

impl Promise {
    /// A promise that has not yet come due.
    #[must_use]
    pub fn new(
        description: String,
        commitment: Commitment,
        audience: DeadlineAudience,
        made: Instant,
    ) -> Self {
        Self {
            description,
            commitment,
            audience,
            made,
            kept: None,
        }
    }

    /// Whether the promise has come due and been resolved.
    #[must_use]
    pub const fn is_resolved(&self) -> bool {
        self.kept.is_some()
    }
}

/// The agency's political position.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PoliticalState {
    /// The state's trust.
    pub approval: Approval,
    /// The institution's patience, and the state's willingness to accommodate
    /// the director's whims.
    ///
    /// **This is the anti-spamming mechanism, and it is deliberately not a
    /// cap.** Every order that lands draws on it, and a director who changes
    /// the plan every hour will find that the state stops cooperating. See
    /// `MANAGEABLE_PARTS.md` §3.5.
    pub goodwill: Goodwill,
    /// The currency the player spends on favours and exceptions.
    pub political_capital: i32,
    /// The promises outstanding and settled.
    pub promises: Vec<Promise>,
    /// The fractional remainders carried between rate applications.
    pub remainders: crate::scale::Remainders,
}

impl PoliticalState {
    /// A new agency's political position.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            approval: Approval::neutral(),
            goodwill: Goodwill::neutral(),
            political_capital: 0,
            promises: Vec::new(),
            remainders: crate::scale::Remainders::new(),
        }
    }

    /// The promises that have come due and not been resolved.
    pub fn outstanding(&self) -> impl Iterator<Item = &Promise> {
        self.promises
            .iter()
            .filter(|promise| !promise.is_resolved())
    }
}

impl Default for PoliticalState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::CommitmentId;

    #[test]
    fn an_outstanding_promise_is_not_resolved() {
        let promise = Promise::new(
            "An orbital launch by 2041".to_owned(),
            Commitment::committed(CommitmentId(0), Instant::from_months(24)),
            DeadlineAudience::Political,
            Instant::ZERO,
        );
        assert!(!promise.is_resolved());
    }

    #[test]
    fn resolved_promises_are_not_outstanding() {
        let mut state = PoliticalState::new();
        state.promises.push(Promise::new(
            "An orbital launch by 2041".to_owned(),
            Commitment::committed(CommitmentId(0), Instant::from_months(24)),
            DeadlineAudience::Political,
            Instant::ZERO,
        ));
        assert_eq!(state.outstanding().count(), 1);
        state.promises[0].kept = Some(true);
        assert_eq!(state.outstanding().count(), 0);
    }
}
