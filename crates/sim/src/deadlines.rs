//! What happens when a commitment comes due.
//!
//! **A deadline has consequences, and the consequences depend on who holds
//! it.** An internal date that slips costs nothing but a note in the report; a
//! political date that slips costs the state's approval, and a mandatory one
//! costs the agency's licence to operate.
//!
//! The table is one outcome per audience, because the whole point of the
//! audience is that the promise was made to *somebody*, and different
//! somebodies are differently forgiving.

use domain::chronicle::Severity;
use domain::money::Money;
use domain::programme::DeadlineAudience;

use crate::balance;

/// What a deadline's resolution costs.
#[derive(Clone, Copy, Debug)]
pub struct Outcome {
    /// The change in the state's approval.
    pub approval: i16,
    /// The change in goodwill.
    pub goodwill: i16,
    /// The political capital it costs.
    pub political_capital: i64,
    /// The money it costs.
    pub cost: Money,
    /// How bad it is, for the report and the chronicle.
    pub severity: Severity,
    /// What the chronicle says.
    pub message: &'static str,
}

/// The outcome of a deadline, met or missed.
///
/// **One outcome per audience [D].** The audience is who the promise was made
/// to, and the audience is what decides the price of breaking it.
#[must_use]
pub fn outcome(audience: DeadlineAudience, met: bool) -> Outcome {
    if met {
        return met_outcome(audience);
    }
    missed_outcome(audience)
}

/// What a kept deadline costs.
fn met_outcome(audience: DeadlineAudience) -> Outcome {
    Outcome {
        approval: balance::APPROVAL_DEADLINE_MET,
        goodwill: 0,
        political_capital: 0,
        cost: Money::ZERO,
        severity: Severity::Info,
        message: match audience {
            DeadlineAudience::Internal => "The agency's own deadline was met.",
            DeadlineAudience::Contractual => "The contractor's deadline was met.",
            DeadlineAudience::Political => "The state was promised this, and it was delivered.",
            DeadlineAudience::Public => "The public was promised this, and it was delivered.",
            DeadlineAudience::Mandatory => "The regulator was satisfied.",
        },
    }
}

/// What a missed deadline costs.
///
/// **An internal deadline costs nothing outside the agency.** The exit test
/// for the deadline table is that missing a political deadline costs approval
/// and missing an internal one does not, and that is the rule: an internal
/// date that slips is a plan that was wrong, and nobody was promised it.
fn missed_outcome(audience: DeadlineAudience) -> Outcome {
    match audience {
        DeadlineAudience::Internal => Outcome {
            approval: 0,
            goodwill: 0,
            political_capital: 0,
            cost: Money::ZERO,
            severity: Severity::Minor,
            message: "The agency's own deadline slipped, and nobody was told.",
        },
        DeadlineAudience::Contractual => Outcome {
            approval: balance::APPROVAL_DEADLINE_MISSED / 2,
            goodwill: balance::GOODWILL_POSTPONE_CONTRACTUAL,
            political_capital: 0,
            cost: Money::ZERO,
            severity: Severity::Major,
            message: "A contractual deadline was missed.",
        },
        DeadlineAudience::Political => Outcome {
            approval: balance::APPROVAL_DEADLINE_MISSED,
            goodwill: balance::GOODWILL_POSTPONE_POLITICAL,
            political_capital: 0,
            cost: Money::ZERO,
            severity: Severity::Major,
            message: "A deadline the state was promised was missed.",
        },
        DeadlineAudience::Public => Outcome {
            approval: balance::APPROVAL_DEADLINE_MISSED,
            goodwill: balance::GOODWILL_POSTPONE_POLITICAL,
            political_capital: 0,
            cost: Money::ZERO,
            severity: Severity::Major,
            message: "A deadline the public was told about was missed.",
        },
        DeadlineAudience::Mandatory => Outcome {
            approval: balance::APPROVAL_DEADLINE_MISSED,
            goodwill: balance::GOODWILL_POSTPONE_POLITICAL,
            political_capital: 0,
            cost: Money::ZERO,
            severity: Severity::Critical,
            message: "A deadline the regulator required was missed.",
        },
    }
}

/// The political capital postponing a deadline costs.
#[must_use]
pub fn postpone_capital(audience: DeadlineAudience) -> i64 {
    match audience {
        DeadlineAudience::Internal => balance::POSTPONE_INTERNAL_CAPITAL,
        DeadlineAudience::Contractual => balance::POSTPONE_CONTRACTUAL_CAPITAL,
        DeadlineAudience::Political | DeadlineAudience::Public | DeadlineAudience::Mandatory => {
            balance::POSTPONE_POLITICAL_CAPITAL
        }
    }
}

/// The goodwill postponing a deadline costs.
#[must_use]
pub fn postpone_goodwill(audience: DeadlineAudience) -> i16 {
    match audience {
        DeadlineAudience::Internal => balance::GOODWILL_POSTPONE_INTERNAL,
        DeadlineAudience::Contractual => balance::GOODWILL_POSTPONE_CONTRACTUAL,
        DeadlineAudience::Political | DeadlineAudience::Public | DeadlineAudience::Mandatory => {
            balance::GOODWILL_POSTPONE_POLITICAL
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missed_political_deadline_costs_approval() {
        let outcome = outcome(DeadlineAudience::Political, false);
        assert_eq!(outcome.approval, balance::APPROVAL_DEADLINE_MISSED);
    }

    #[test]
    fn a_missed_internal_deadline_costs_nothing() {
        // **The exit test for the whole table.**
        let outcome = outcome(DeadlineAudience::Internal, false);
        assert_eq!(outcome.approval, 0, "and nobody outside the agency cares");
    }

    #[test]
    fn a_kept_deadline_is_rewarded() {
        for audience in DeadlineAudience::ALL {
            let outcome = outcome(audience, true);
            assert_eq!(outcome.approval, balance::APPROVAL_DEADLINE_MET);
        }
    }

    #[test]
    fn postponing_a_political_deadline_costs_the_most() {
        assert!(
            postpone_capital(DeadlineAudience::Political)
                > postpone_capital(DeadlineAudience::Contractual)
                && postpone_capital(DeadlineAudience::Contractual)
                    > postpone_capital(DeadlineAudience::Internal),
            "the state is the hardest to ask"
        );
    }
}
