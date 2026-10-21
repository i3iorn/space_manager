//! The commitment mechanic.
//!
//! **The commitment is the heart of Slice 2**, and these tests are about the
//! three things it has to get right: the slack is the gap between the plan and
//! the promise, a deadline that falls due is resolved against the work rather
//! than the clock, and who holds the deadline decides what it costs.

use domain::ids::{ObjectiveId, ProgrammeId};
use domain::order::OrderKind;
use domain::programme::{DeadlineAudience, ObjectiveKind, Programme, ProgrammeCategory};
use domain::time::{Duration, Instant};
use sim::{Scenario, Sim, balance};

/// A campaign with a programme, an objective, and a deadline.
fn campaign() -> Sim {
    let scenario = Scenario::first_year();
    Sim::new(&scenario).expect("the campaign starts")
}

/// The slack between a target and a commitment.
fn slack_of(target: Instant, committed: Instant) -> Duration {
    domain::programme::Commitment::new(
        domain::ids::CommitmentId(0),
        Some(target),
        Some(committed),
        DeadlineAudience::Political,
    )
    .slack()
    .expect("there is slack")
}

#[test]
fn the_slack_is_the_gap_between_the_plan_and_the_promise() {
    // **The slack is a duration, and it is the buffer the player manages.**
    let target = Instant::from_months(18);
    let committed = Instant::from_months(36);
    assert_eq!(
        slack_of(target, committed),
        Duration::from_months(18),
        "eighteen months of slack"
    );
}

#[test]
fn a_commitment_with_no_gap_has_no_slack() {
    // **Zero slack is a real state**, and it is a different one from no
    // commitment at all: the player promised the same day as the plan, and
    // there is nothing between them.
    let instant = Instant::from_months(12);
    let commitment = domain::programme::Commitment::new(
        domain::ids::CommitmentId(0),
        Some(instant),
        Some(instant),
        DeadlineAudience::Political,
    );
    assert_eq!(commitment.slack(), Some(Duration::ZERO));
}

#[test]
fn a_missed_political_deadline_costs_approval() {
    // **The exit test for the deadline table.**
    let outcome = sim::deadlines::outcome(DeadlineAudience::Political, false);
    assert_eq!(outcome.approval, balance::APPROVAL_DEADLINE_MISSED);
    assert!(outcome.approval < 0, "and missing a deadline is a loss");
}

#[test]
fn a_missed_internal_deadline_costs_nothing() {
    let outcome = sim::deadlines::outcome(DeadlineAudience::Internal, false);
    assert_eq!(
        outcome.approval, 0,
        "nobody outside the agency was promised it"
    );
}

#[test]
fn a_kept_deadline_is_rewarded() {
    let outcome = sim::deadlines::outcome(DeadlineAudience::Political, true);
    assert_eq!(outcome.approval, balance::APPROVAL_DEADLINE_MET);
}

#[test]
fn the_scenario_has_a_political_deadline() {
    // **The government's orbital deadline.** The scenario opens with a promise
    // the player has to answer, and the promise has a date on it.
    let sim = campaign();
    let state = sim.state();
    let programme = state
        .programmes
        .values()
        .find(|programme| programme.category == ProgrammeCategory::Orbital)
        .expect("the orbital programme is there");
    let objective = programme.objectives.first().expect("it has an objective");
    assert_eq!(
        objective.commitment.audience,
        DeadlineAudience::Political,
        "and it is a promise to the state"
    );
    assert!(objective.commitment.slack().is_some(), "and it has slack");
    assert!(objective.commitment.committed.is_some(), "and a date on it");
}

#[test]
fn a_commitment_falls_due_and_is_resolved() {
    // **A commitment that is not on the scheduler never falls due.** The
    // scenario's deadline is scheduled, and when the campaign reaches it the
    // commitment is resolved.
    let mut sim = campaign();
    let before = sim.state().political.approval.value;

    // The commitment is eighteen months out; the campaign runs past it.
    sim.advance_to(Instant::from_months(40))
        .expect("the campaign advances");

    let state = sim.state();
    let programme = state.programmes.values().next().expect("a programme");
    let objective = programme.objectives.first().expect("an objective");
    assert!(
        objective.met.is_some(),
        "the commitment was resolved when it fell due"
    );
    assert!(
        state.political.approval.value != before,
        "and the resolution had a consequence"
    );
}

#[test]
fn a_programme_progresses_at_the_rate_of_its_staff() {
    // **An objective is advanced by the programme's staff**, and the points
    // are summed the way a project's are.
    let mut sim = campaign();
    let start = sim.state().programmes.values().next().expect("a programme");
    let start_points = start
        .objectives
        .first()
        .expect("an objective")
        .progress
        .points;
    sim.advance_to(Instant::from_months(6))
        .expect("the campaign advances");
    let after = sim.state().programmes.values().next().expect("a programme");
    let points = after
        .objectives
        .first()
        .expect("an objective")
        .progress
        .points;
    assert!(
        points > start_points,
        "the objective moved: {start_points} then {points}"
    );
}

#[test]
fn a_deadline_can_be_pushed_back() {
    // **Renegotiating is not free, and it is an order.** The political capital
    // it costs is the price of asking the state.
    let mut sim = campaign();
    let programme = sim.state().programmes.values().next().expect("a programme");
    let objective = programme.objectives.first().expect("an objective");
    let objective_id = objective.id;
    let programme_id = programme.id;

    let capital_before = sim.state().political.political_capital;
    sim.issue(OrderKind::PostponeDeadline {
        programme: programme_id,
        objective: objective_id,
        to: Instant::from_months(60),
    })
    .expect("the order is accepted");
    sim.advance_to(Instant::from_months(2))
        .expect("the campaign advances");

    let state = sim.state();
    let programme = state.programmes.get(&programme_id).expect("the programme");
    let objective = programme
        .objectives
        .iter()
        .find(|o| o.id == objective_id)
        .expect("the objective");
    assert_eq!(
        objective.commitment.committed,
        Some(Instant::from_months(60)),
        "and the deadline moved"
    );
    assert!(
        state.political.political_capital < capital_before,
        "and the political capital was spent"
    );
}

#[test]
fn an_objective_can_be_set() {
    // **The player sets deadlines**, and the objective goes on the scheduler.
    let mut sim = campaign();
    let programme = sim.state().programmes.values().next().expect("a programme");
    let programme_id = programme.id;

    sim.issue(OrderKind::SetObjective {
        programme: programme_id,
        description: "Reach orbit.".to_owned(),
        commitment: domain::programme::Commitment::new(
            domain::ids::CommitmentId(0),
            Some(Instant::from_months(10)),
            Some(Instant::from_months(20)),
            DeadlineAudience::Internal,
        ),
        kind: ObjectiveKind::Secondary,
    })
    .expect("the order is accepted");
    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    let state = sim.state();
    let programme = state.programmes.get(&programme_id).expect("the programme");
    assert!(programme.objectives.len() > 1, "the objective was added");
    assert!(
        programme
            .objectives
            .iter()
            .any(|objective| objective.description == "Reach orbit."),
        "and it is the one that was asked for"
    );
}

#[test]
fn a_programme_can_be_opened() {
    let mut sim = campaign();
    sim.issue(OrderKind::CreateProgramme {
        title: "Lunar Landing".to_owned(),
        category: ProgrammeCategory::Lunar,
    })
    .expect("the order is accepted");
    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    let state = sim.state();
    assert!(
        state
            .programmes
            .values()
            .any(|programme| programme.title == "Lunar Landing"),
        "the programme was opened"
    );
}

#[test]
fn a_budget_can_be_set() {
    let mut sim = campaign();
    let programme = sim.state().programmes.values().next().expect("a programme");
    let programme_id = programme.id;
    let allocation = domain::money::Money::from_credits(25_000);
    sim.issue(OrderKind::SetBudget {
        programme: programme_id,
        allocation,
    })
    .expect("the order is accepted");
    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    let state = sim.state();
    let programme = state.programmes.get(&programme_id).expect("the programme");
    assert_eq!(programme.budget.allocation, allocation);
}

#[test]
fn the_ids_are_the_programmes_own() {
    // **A commitment's id is allocated by the counter**, so two objectives in
    // the same programme do not collide.
    let mut sim = campaign();
    let programme = sim.state().programmes.values().next().expect("a programme");
    let programme_id = programme.id;
    sim.issue(OrderKind::SetObjective {
        programme: programme_id,
        description: "A second objective.".to_owned(),
        commitment: domain::programme::Commitment::new(
            domain::ids::CommitmentId(0),
            Some(Instant::from_months(10)),
            Some(Instant::from_months(20)),
            DeadlineAudience::Internal,
        ),
        kind: ObjectiveKind::Secondary,
    })
    .expect("the order is accepted");
    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    let state = sim.state();
    let programme = state.programmes.get(&programme_id).expect("the programme");
    let mut ids: Vec<_> = programme
        .objectives
        .iter()
        .map(|objective| objective.commitment.id)
        .collect();
    let count = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), count, "the commitments have distinct ids");
}

/// The programme's own id, for the tests that need it.
#[must_use]
pub fn programme_id(programme: &Programme) -> ProgrammeId {
    programme.id
}

/// The objective's own id, for the tests that need it.
#[must_use]
pub fn objective_id(objective: &domain::programme::Objective) -> ObjectiveId {
    objective.id
}
