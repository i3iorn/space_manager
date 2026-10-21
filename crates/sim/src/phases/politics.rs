//! Phase 7 — politics and public.
//!
//! Approval, political capital, promises falling due, public opinion, and the
//! goodwill.
//!
//! **The goodwill is the institution's patience.** Every order that lands draws
//! on it, a broken promise costs ten, and it comes back at three percent a
//! month. Below forty the state starts obstructing, and below eighty it starts
//! to wonder whether the agency should exist at all.

use domain::chronicle::{Category, ChronicleEntry, Severity};
use domain::programme::DeadlineAudience;

use crate::balance;
use crate::error::SimError;
use crate::tick::{MonthContext, Phase};

/// The politics phase.
pub struct PoliticsPhase;

impl Phase for PoliticsPhase {
    const NAME: &'static str = "politics";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        check_promises(context);
        drift_approval(context);
        decay_prestige(context);
        regenerate_capital(context);
        check_mandate(context);
        Ok(())
    }
}

/// Warns when the state begins to reconsider the agency's mandate.
///
/// **Below minus eighty the goodwill is no longer merely low** — the state is
/// starting to ask whether it should keep the agency at all. The full crisis
/// arrives with the event catalogue; the warning is what the player can be
/// told today.
fn check_mandate(context: &mut MonthContext<'_>) {
    let goodwill = context.state.political.goodwill.percent();
    if goodwill >= balance::GOODWILL_MANDATE {
        return;
    }
    let message = "The state's patience is exhausted. It is beginning to wonder \
                    whether it should keep the agency at all."
        .to_owned();
    if !context.warnings.contains(&message) {
        context.warnings.push(message);
    }
}

/// Resolves the promises that have come due.
///
/// A broken promise costs nearly three times what a kept one rewards, and the
/// state remembers the people it was promised to.
fn check_promises(context: &mut MonthContext<'_>) {
    let now = context.month;
    let mut resolved: Vec<(usize, bool)> = Vec::new();

    for (index, promise) in context.state.political.promises.iter().enumerate() {
        if promise.is_resolved() {
            continue;
        }
        let Some(due) = promise.commitment.due() else {
            continue;
        };
        if due > now {
            continue;
        }
        // A promise is kept when the agency reached its own plan by the time
        // it was held to. A promise made without a plan was never a plan; it
        // was a bare date, and reaching it is not delivery.
        let kept = match (promise.commitment.target, promise.commitment.committed) {
            (Some(target), Some(committed)) => target <= committed && now >= target,
            _ => false,
        };
        resolved.push((index, kept));
    }

    for (index, kept) in resolved {
        let description = context.state.political.promises[index].description.clone();
        let audience = context.state.political.promises[index].audience;
        context.state.political.promises[index].kept = Some(kept);

        let change = if kept {
            crate::balance::APPROVAL_PROMISE_KEPT
        } else {
            crate::balance::APPROVAL_PROMISE_BROKEN
        };
        context.state.political.approval.value = (context.state.political.approval.value + change)
            .clamp(
                -domain::scale::signed_percent_to_units(100),
                domain::scale::signed_percent_to_units(100),
            );

        // **A broken promise costs goodwill, and a kept one earns it.** The
        // cost of a broken promise is more than a week of ordinary orders, and
        // that is the point: the state's memory is the mechanism.
        let goodwill_change = if kept {
            crate::balance::GOODWILL_KEPT_PROMISE
        } else {
            -crate::balance::GOODWILL_BROKEN_PROMISE
        };
        context.state.political.goodwill =
            context.state.political.goodwill.granted(goodwill_change);

        let message = if kept {
            format!("A promise was kept: {description}.")
        } else {
            format!("A promise was broken: {description}.")
        };
        let severity = if kept {
            Severity::Minor
        } else {
            Severity::Major
        };
        context
            .programme_lines
            .push(domain::report::ReportLine::new(message.clone()).with_severity(severity));

        let entry = ChronicleEntry::new(
            context.state.next_chronicle_id(),
            now,
            Category::Political,
            message,
            severity,
        );
        context.events.push(entry);

        if !kept {
            context
                .warnings
                .push(format!("A promise to {} was broken.", audience.label()));
        }
    }
}

/// Approval drifts back towards neutral, so a state that has not been courted
/// forgets.
fn drift_approval(context: &mut MonthContext<'_>) {
    if !is_a_year(context) {
        return;
    }
    let approval = &mut context.state.political.approval.value;
    if *approval > 0 {
        *approval -= balance::APPROVAL_DRIFT_PER_YEAR;
    } else if *approval < 0 {
        *approval += balance::APPROVAL_DRIFT_PER_YEAR;
    }
}

/// The agency's prestige decays a point a year.
///
/// **The prestige is slow, and that is the point.** It is built by
/// significant achievement and lost by nothing much at all, so it is the
/// one number in the game that a patient player accumulates.
fn decay_prestige(context: &mut MonthContext<'_>) {
    if !is_a_year(context) {
        return;
    }
    let prestige = &mut context.state.public.prestige.value;
    if *prestige > 0 {
        *prestige -= balance::PRESTIGE_DECAY_PER_YEAR;
    } else if *prestige < 0 {
        *prestige += balance::PRESTIGE_DECAY_PER_YEAR;
    }
}

/// Whether the month being resolved is the first of a year.
///
/// **The drifts are yearly, and the campaign's months are counted from its
/// start.** The twelfth month of the campaign is the first month of its
/// second year, and the drift belongs there.
fn is_a_year(context: &MonthContext<'_>) -> bool {
    context.month.months() % 12 == 0
}

/// Political capital regenerates slowly, and only if nothing was broken.
fn regenerate_capital(context: &mut MonthContext<'_>) {
    let broken = context.state.political.promises.iter().any(|promise| {
        promise.kept == Some(false)
            && promise
                .commitment
                .due()
                .is_some_and(|due| due == context.month)
    });
    if broken {
        return;
    }
    context.state.political.political_capital =
        (context.state.political.political_capital + 3).min(100);
}

/// The effect of a kept or broken promise on the approval, used by the event
/// catalogue and by the interface.
#[must_use]
pub fn approval_change(kept: bool) -> i16 {
    if kept {
        crate::balance::APPROVAL_PROMISE_KEPT
    } else {
        crate::balance::APPROVAL_PROMISE_BROKEN
    }
}

/// The audiences a promise can be made to, for the interface.
#[must_use]
pub fn audiences() -> [DeadlineAudience; 5] {
    DeadlineAudience::ALL
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::AgencyId;
    use domain::ids::CommitmentId;
    use domain::political::Promise;
    use domain::programme::Commitment;
    use domain::state::{Agency, CampaignState};
    use domain::time::Instant;

    use crate::rng::Rng;

    fn setup() -> (CampaignState, Rng) {
        (
            CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039)),
            Rng::new(13),
        )
    }

    #[test]
    fn a_broken_promise_costs_approval() {
        let (mut state, rng) = setup();
        state.political.promises.push(Promise::new(
            "An orbital launch by 2041".to_owned(),
            Commitment::committed(CommitmentId(0), Instant::from_months(6)),
            DeadlineAudience::Political,
            Instant::ZERO,
        ));
        let mut context = MonthContext::new(&mut state, &rng, Instant::from_months(12), Vec::new());
        PoliticsPhase::run(&mut context).expect("the phase runs");
        assert!(context.state.political.approval.value < 0);
        assert!(!context.warnings.is_empty());
    }

    #[test]
    fn a_broken_promise_costs_goodwill() {
        // **A broken promise costs more than a week of ordinary orders.** The
        // state's memory is the mechanism behind the whole commitment model.
        let (mut state, rng) = setup();
        state.political.promises.push(Promise::new(
            "An orbital launch by 2041".to_owned(),
            Commitment::committed(CommitmentId(0), Instant::from_months(6)),
            DeadlineAudience::Political,
            Instant::ZERO,
        ));
        let mut context = MonthContext::new(&mut state, &rng, Instant::from_months(12), Vec::new());
        PoliticsPhase::run(&mut context).expect("the phase runs");
        assert_eq!(
            context.state.political.goodwill.percent(),
            -crate::balance::GOODWILL_BROKEN_PROMISE
        );
    }

    #[test]
    fn a_kept_promise_earns_goodwill() {
        let (mut state, rng) = setup();
        state.political.promises.push(Promise::new(
            "An orbital launch by 2041".to_owned(),
            Commitment::new(
                CommitmentId(0),
                Some(Instant::from_months(6)),
                Some(Instant::from_months(12)),
                DeadlineAudience::Political,
            ),
            DeadlineAudience::Political,
            Instant::ZERO,
        ));
        let mut context = MonthContext::new(&mut state, &rng, Instant::from_months(12), Vec::new());
        PoliticsPhase::run(&mut context).expect("the phase runs");
        assert_eq!(
            context.state.political.goodwill.percent(),
            crate::balance::GOODWILL_KEPT_PROMISE
        );
    }

    #[test]
    fn an_exhausted_patience_warns_the_player() {
        // **Below minus eighty the state is reconsidering the mandate.** The
        // full crisis arrives with the event catalogue; the warning is what
        // the player can be told today.
        let (mut state, rng) = setup();
        state.political.goodwill.value = domain::scale::signed_percent_to_units(-85);
        let mut context = MonthContext::new(&mut state, &rng, Instant::from_months(12), Vec::new());
        PoliticsPhase::run(&mut context).expect("the phase runs");
        assert!(
            context
                .warnings
                .iter()
                .any(|warning| warning.contains("patience")),
            "the player should be warned: {:?}",
            context.warnings
        );
    }

    #[test]
    fn a_patient_state_does_not_warn() {
        let (mut state, rng) = setup();
        state.political.goodwill.value = domain::scale::signed_percent_to_units(-40);
        let mut context = MonthContext::new(&mut state, &rng, Instant::from_months(12), Vec::new());
        PoliticsPhase::run(&mut context).expect("the phase runs");
        assert!(
            !context
                .warnings
                .iter()
                .any(|warning| warning.contains("patience")),
            "the state is merely unhappy, not reconsidering the mandate"
        );
    }

    #[test]
    fn a_kept_promise_earns_approval() {
        let (mut state, rng) = setup();
        state.political.promises.push(Promise::new(
            "An orbital launch by 2041".to_owned(),
            Commitment::new(
                CommitmentId(0),
                Some(Instant::from_months(6)),
                Some(Instant::from_months(12)),
                DeadlineAudience::Political,
            ),
            DeadlineAudience::Political,
            Instant::ZERO,
        ));
        let mut context = MonthContext::new(&mut state, &rng, Instant::from_months(12), Vec::new());
        PoliticsPhase::run(&mut context).expect("the phase runs");
        assert!(context.state.political.approval.value > 0);
    }

    #[test]
    fn a_promise_that_has_not_come_due_is_untouched() {
        let (mut state, rng) = setup();
        state.political.promises.push(Promise::new(
            "An orbital launch by 2041".to_owned(),
            Commitment::new(
                CommitmentId(0),
                Some(Instant::from_months(30)),
                Some(Instant::from_months(36)),
                DeadlineAudience::Political,
            ),
            DeadlineAudience::Political,
            Instant::ZERO,
        ));
        let mut context = MonthContext::new(&mut state, &rng, Instant::from_months(12), Vec::new());
        PoliticsPhase::run(&mut context).expect("the phase runs");
        assert_eq!(context.state.political.promises.len(), 1);
        assert!(!context.state.political.promises[0].is_resolved());
    }
}
