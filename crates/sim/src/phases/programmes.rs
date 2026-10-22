//! The programmes, and their commitments.
//!
//! **This is the strategic layer.** A programme is what the agency has
//! committed to, an objective is a dated piece of that, and the commitment
//! carries two instants: the target the agency plans for and the committed
//! instant it promised. The gap between them is the slack.
//!
//! The orders in this phase are the player's only way to change any of it,
//! and every one of them lands at its own instant.

use domain::chronicle::{Category, Severity, SubjectRef};
use domain::order::OrderKind;
use domain::programme::{Objective, ObjectiveKind, Programme};
use domain::state::CampaignState;

use crate::balance;
use crate::deadlines;
use crate::tick::MonthContext;

/// The programmes phase.
pub struct ProgrammePhase;

impl crate::tick::Phase for ProgrammePhase {
    const NAME: &'static str = "programmes";

    fn run(context: &mut MonthContext<'_>) -> Result<(), crate::error::SimError> {
        apply_orders(context);
        Ok(())
    }
}

/// Applies the programme orders that have landed.
fn apply_orders(context: &mut MonthContext<'_>) {
    let orders = std::mem::take(&mut context.validated);
    for order in orders {
        if !apply_one(context, &order) {
            // **An order that is not ours goes back**, so the next phase can
            // have it.
            context.validated.push(order);
        }
    }
}

/// Applies one order, and reports whether it was one of ours.
pub fn apply_one(context: &mut MonthContext<'_>, order: &OrderKind) -> bool {
    match order {
        OrderKind::CreateProgramme { title, category } => {
            create_programme(context, title.as_str(), *category);
            true
        }
        OrderKind::SetObjective {
            programme,
            description,
            commitment,
            kind,
        } => {
            set_objective(
                context,
                *programme,
                description.as_str(),
                *commitment,
                *kind,
            );
            true
        }
        OrderKind::SetBudget {
            programme,
            allocation,
        } => {
            set_budget(context, *programme, *allocation);
            true
        }
        OrderKind::PostponeDeadline {
            programme,
            objective,
            to,
        } => {
            postpone(context, *programme, *objective, *to);
            true
        }
        _ => false,
    }
}

/// Starts a programme.
fn create_programme(
    context: &mut MonthContext<'_>,
    title: &str,
    category: domain::programme::ProgrammeCategory,
) {
    let id = context.state.id_counters.next_programme();
    let programme = Programme::new(id, title.to_owned(), category);
    let name = programme.title.clone();
    context.state.programmes.insert(id, programme);
    context.programme_lines.push(
        domain::report::ReportLine::new(format!("The {name} programme was opened."))
            .with_severity(Severity::Info),
    );
    context.events.push(
        domain::chronicle::ChronicleEntry::new(
            context.state.next_chronicle_id(),
            context.month,
            Category::Political,
            format!("Programme opened: {name}"),
            Severity::Info,
        )
        .about(SubjectRef::Programme(id)),
    );
}

/// Adds an objective to a programme.
fn set_objective(
    context: &mut MonthContext<'_>,
    programme: domain::ids::ProgrammeId,
    description: &str,
    commitment: domain::programme::Commitment,
    kind: ObjectiveKind,
) {
    if !context.state.programmes.contains_key(&programme) {
        return;
    }
    let id = context.state.id_counters.next_objective();
    let mut commitment = commitment;
    commitment.id = context.state.id_counters.next_commitment();

    let objective = Objective {
        id,
        description: description.to_owned(),
        kind,
        commitment,
        progress: domain::research::Progress::empty(balance::OBJECTIVE_POINTS_TARGET),
        met: None,
        // **The primary objective of a programme that is about reaching
        // somewhere is finished by reaching it**, and not by the programme's
        // staff grinding through enough points.
        completion: programme_completion(context, programme, kind),
        milestone_reached: false,
        remainders: domain::scale::Remainders::new(),
    };

    let programme = context
        .state
        .programmes
        .get_mut(&programme)
        .expect("the programme exists");
    programme.objectives.push(objective);

    // **A commitment with a date is a thing that falls due**, and a thing
    // that falls due is on the scheduler from the moment it is made.
    if let Some(due) = commitment.committed {
        context.schedule.push((
            due,
            crate::scheduler::ScheduledKind::CommitmentDue {
                commitment: commitment.id,
            },
        ));
    }

    context.programme_lines.push(
        domain::report::ReportLine::new(format!("An objective was set: {description}."))
            .with_severity(Severity::Info),
    );
}

/// Sets a programme's budget.
fn set_budget(
    context: &mut MonthContext<'_>,
    programme: domain::ids::ProgrammeId,
    allocation: domain::money::Money,
) {
    let Some(existing) = context.state.programmes.get_mut(&programme) else {
        return;
    };
    existing.budget.allocation = allocation;
    context.programme_lines.push(
        domain::report::ReportLine::new(format!("The budget was set to {allocation}."))
            .with_severity(Severity::Info),
    );
}

/// Pushes a commitment back.
///
/// **Renegotiating is not free.** The cost depends on who holds the deadline,
/// and the state is the hardest to ask.
fn postpone(
    context: &mut MonthContext<'_>,
    programme: domain::ids::ProgrammeId,
    wanted: domain::ids::ObjectiveId,
    to: domain::time::Instant,
) {
    let Some(existing) = context.state.programmes.get(&programme) else {
        return;
    };
    let Some(objective) = existing.objectives.iter().find(|o| o.id == wanted) else {
        return;
    };
    let audience = objective.commitment.audience;
    let capital = deadlines::postpone_capital(audience);
    let goodwill = deadlines::postpone_goodwill(audience);

    let programme = context
        .state
        .programmes
        .get_mut(&programme)
        .expect("the programme exists");
    let Some(objective) = programme.objectives.iter_mut().find(|o| o.id == wanted) else {
        return;
    };
    objective.commitment.committed = Some(to);
    let id = objective.commitment.id;

    context.state.political.political_capital -= i32::try_from(capital).unwrap_or(i32::MAX);
    apply_goodwill(context.state, goodwill);

    // **The new date is the one that falls due now.**
    context.schedule.push((
        to,
        crate::scheduler::ScheduledKind::CommitmentDue { commitment: id },
    ));

    context.programme_lines.push(
        domain::report::ReportLine::new(format!("A deadline was pushed back to {to}."))
            .with_severity(Severity::Minor),
    );
}

/// Resolves a commitment that has fallen due.
///
/// **The commitment is resolved against the work, not against the clock.** An
/// objective that is finished when the date arrives is met; one that is not is
/// missed, and the audience decides what that costs.
pub fn resolve(context: &mut MonthContext<'_>, commitment: domain::ids::CommitmentId) {
    let Some((programme_id, objective_id)) = find_objective(context.state, commitment) else {
        return;
    };

    let programme = context
        .state
        .programmes
        .get_mut(&programme_id)
        .expect("the programme exists");
    let Some(objective) = programme
        .objectives
        .iter_mut()
        .find(|objective| objective.id == objective_id)
    else {
        return;
    };
    if objective.met.is_some() {
        return;
    }

    let met = objective.is_complete();
    objective.met = Some(met);
    let audience = objective.commitment.audience;
    let kind = objective.kind;
    let title = programme.title.clone();

    let outcome = deadlines::outcome(audience, met);
    apply_approval(context.state, outcome.approval);
    apply_goodwill(context.state, outcome.goodwill);
    if !outcome.cost.is_zero() {
        context.state.treasury.balance = context
            .state
            .treasury
            .balance
            .checked_sub(outcome.cost)
            .unwrap_or(context.state.treasury.balance);
    }

    // **A missed primary objective fails the programme.**
    //
    // **A programme the player has lost is a programme with an aftermath**, and
    // the aftermath starts with the programme being marked as failed. Without
    // this, a missed deadline is a number that goes down and nothing else, and
    // the player has no way of knowing that the work is over.
    if kind == ObjectiveKind::Primary && !met {
        if let Some(programme) = context.state.programmes.get_mut(&programme_id) {
            programme.status = domain::programme::ProgrammeStatus::Failed;
        }
    }

    context.programme_lines.push(
        domain::report::ReportLine::new(format!("{title}: {}", outcome.message))
            .with_severity(outcome.severity),
    );
    context.events.push(
        domain::chronicle::ChronicleEntry::new(
            context.state.next_chronicle_id(),
            context.month,
            domain::chronicle::Category::Political,
            format!("{title}: {}", outcome.message),
            outcome.severity,
        )
        .about(SubjectRef::Programme(programme_id)),
    );
}
/// Applies a change to the state's approval.
///
/// **The approval is clamped to the scale**, because a hundred and ten
/// percent approval is not a thing.
fn apply_approval(state: &mut CampaignState, change: i16) {
    let limit = i32::from(domain::scale::signed_percent_to_units(100));
    let value = i32::from(state.political.approval.value).saturating_add(i32::from(change));
    state.political.approval.value = i16::try_from(value.clamp(-limit, limit)).unwrap_or(0);
}

/// Applies a change to the goodwill.
///
/// **A change is a charge or a grant**, and the goodwill knows how to do both
/// and clamp itself.
fn apply_goodwill(state: &mut CampaignState, change: i16) {
    state.political.goodwill = if change < 0 {
        state.political.goodwill.charged(-change)
    } else {
        state.political.goodwill.granted(change)
    };
}
/// What finishes a new objective in a programme.
fn programme_completion(
    context: &MonthContext<'_>,
    programme: domain::ids::ProgrammeId,
    kind: ObjectiveKind,
) -> domain::programme::ObjectiveCompletion {
    context
        .state
        .programmes
        .get(&programme)
        .map_or(domain::programme::ObjectiveCompletion::Staff, |programme| {
            programme.completion_for(kind)
        })
}

/// Finds the programme and objective a commitment belongs to.
fn find_objective(
    state: &CampaignState,
    commitment: domain::ids::CommitmentId,
) -> Option<(domain::ids::ProgrammeId, domain::ids::ObjectiveId)> {
    for programme in state.programmes.values() {
        for objective in &programme.objectives {
            if objective.commitment.id == commitment {
                return Some((programme.id, objective.id));
            }
        }
    }
    None
}
