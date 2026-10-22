//! The refusals.
//!
//! **A game that says no is a game the player can trust.** Every one of these
//! is a negative outcome: an order the agency cannot honour, and the reason it
//! was refused.
//!
//! The rules the tests hold to:
//!
//! - **A refused order changes nothing.** Not the state, not the treasury, not
//!   the chronicle. The rejection is information, and information has no side
//!   effects.
//! - **A refused order is not charged.** The cost is charged when the order
//!   lands, and an order that never lands is never charged.
//! - **The player is told why.** A refusal with no reason is a bug, because the
//!   player has to be able to act on it.
//! - **The validation has no side effects.** It is checked before the order
//!   does anything at all, so it must not change the world to find out.

use domain::ids::{FacilityId, ObjectiveId, PersonId, ProgrammeId};
use domain::order::OrderKind;
use domain::programme::{Commitment, DeadlineAudience, ObjectiveKind};
use domain::time::Instant;
use sim::{Scenario, Sim};

/// A campaign with a programme, a facility, and people in it.
fn campaign() -> Sim {
    Sim::new(&Scenario::first_year()).expect("the campaign starts")
}

/// A person who is not in the agency.
const GONE: PersonId = PersonId(999);

/// A programme that is not in the agency.
const NO_PROGRAMME: ProgrammeId = ProgrammeId(999);

/// A facility that is not in the agency.
const NO_FACILITY: FacilityId = FacilityId(999);

#[test]
fn an_order_for_somebody_who_has_left_is_refused() {
    // **The player is told by name**, because "there is no such person" tells
    // them nothing about what to do next.
    let mut sim = campaign();
    let error = sim
        .issue(OrderKind::Promote { person: GONE })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::PersonGone { .. }),
        "and the refusal says the person is gone: {error}"
    );
    assert!(
        error.to_string().contains("no longer with the agency"),
        "and the message is written for a player: {error}"
    );
}

#[test]
fn putting_somebody_where_they_already_are_is_refused() {
    // **The one thing refused is a no-op**, and the person is named and so is
    // where they are.
    let mut sim = campaign();
    let person = sim
        .state()
        .people
        .values()
        .find(|person| person.is_assigned())
        .expect("somebody is already on something")
        .id;
    let already = sim.state().people[&person].assignment.clone();
    let error = sim
        .issue(OrderKind::Assign {
            person,
            to: already,
        })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::AlreadyAssigned { .. }),
        "and the refusal says they are already there: {error}"
    );
    assert!(
        error.to_string().contains("already on"),
        "and the message is written for a player: {error}"
    );
}

#[test]
fn an_assignment_moves_somebody_off_what_they_were_on() {
    // **The order is "assign / reassign"**, and the design's own order list
    // calls it that. A person who is working on a project can be put in a
    // laboratory instead, and they come off the project's team.
    let mut sim = campaign();
    let person = sim
        .state()
        .people
        .values()
        .find(|person| person.is_assigned())
        .expect("somebody is already on something")
        .id;
    let was = sim.state().people[&person].assignment.clone();
    let facility = *sim
        .state()
        .facilities
        .keys()
        .next()
        .expect("there is a facility");
    sim.issue(OrderKind::Assign {
        person,
        to: domain::person::Assignment::Facility { facility },
    })
    .expect("the order is accepted");
    sim.advance_to(domain::time::Instant::from_days(5))
        .expect("the campaign advances");

    let after = &sim.state().people[&person].assignment;
    assert!(
        matches!(after, domain::person::Assignment::Facility { facility: other } if *other == facility),
        "and they are in the facility: {after:?}"
    );
    assert_ne!(after, &was, "and they came off whatever they were on");
    // **And they are off the old team's list**, which is the bug that made
    // every staffing decision meaningless.
    if let domain::person::Assignment::Project { project } = was {
        assert!(
            !sim.state().projects[&project].team.contains(&person),
            "and the old team does not still have them"
        );
    }
}

#[test]
fn an_order_on_a_programme_that_does_not_exist_is_refused() {
    let mut sim = campaign();
    let error = sim
        .issue(OrderKind::SetBudget {
            programme: NO_PROGRAMME,
            allocation: domain::money::Money::from_credits(1_000),
        })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::NoSuchProgramme { .. }),
        "and the refusal names the missing programme: {error}"
    );
}

#[test]
fn an_order_on_a_facility_that_does_not_exist_is_refused() {
    let mut sim = campaign();
    let person = sim
        .state()
        .people
        .values()
        .find(|person| !person.is_assigned())
        .expect("there is somebody free")
        .id;
    let error = sim
        .issue(OrderKind::Assign {
            person,
            to: domain::person::Assignment::Facility {
                facility: NO_FACILITY,
            },
        })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::NoSuchFacility { .. }),
        "and the refusal names the missing facility: {error}"
    );
}

#[test]
fn an_order_on_a_vehicle_that_does_not_exist_is_refused() {
    let mut sim = campaign();
    let error = sim
        .issue(OrderKind::AuthorizeLaunch {
            vehicle: domain::ids::VehicleId(999),
            destination: "Orbit".to_owned(),
            crew: Vec::new(),
        })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::NoSuchVehicle { .. }),
        "and the refusal names the missing vehicle: {error}"
    );
}

#[test]
fn a_test_campaign_for_a_capability_the_agency_has_not_developed_is_refused() {
    // **The agency cannot test what it has not built.** The palette only
    // offers the capabilities that have been unlocked, and the engine has to
    // agree with the palette: an order the player could not have written must
    // not be accepted.
    let mut sim = campaign();
    let capability = domain::research::CapabilityId(1);
    assert!(
        !sim.state().unlocked.contains(&capability),
        "the scenario has not developed that capability"
    );
    let error = sim
        .issue(OrderKind::StartTestCampaign { capability })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::CapabilityUnavailable { .. }),
        "and the refusal says so: {error}"
    );
}

#[test]
fn an_order_with_no_handler_is_refused_rather_than_ignored() {
    // **A refused order is information.** An order the engine cannot carry out
    // must be refused with a reason, not accepted and quietly ignored.
    let mut sim = campaign();
    let error = sim
        .issue(OrderKind::PublishResult {
            result: "The soil survey".to_owned(),
            mode: domain::order::KnowledgeMode::Open,
        })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::Rejected { .. }),
        "and the refusal says why: {error}"
    );
    assert!(
        error.to_string().contains("no way to"),
        "and the message is written for a player: {error}"
    );
}

#[test]
fn a_refused_order_changes_nothing() {
    // **The rule the whole order model rests on.** A refusal is information,
    // and information has no side effects: the state, the treasury and the
    // chronicle are all exactly as they were.
    let mut sim = campaign();
    let before = sim.state_hash();
    let balance = sim.state().treasury.balance;
    let chronicle = sim.state().chronicle.len();

    let _ = sim.issue(OrderKind::Promote { person: GONE });
    let _ = sim.issue(OrderKind::SetBudget {
        programme: NO_PROGRAMME,
        allocation: domain::money::Money::from_credits(1_000),
    });
    let _ = sim.issue(OrderKind::StartTestCampaign {
        capability: domain::research::CapabilityId(1),
    });

    assert_eq!(sim.state_hash(), before, "the state is untouched");
    assert_eq!(sim.state().treasury.balance, balance, "and the treasury");
    assert_eq!(sim.state().chronicle.len(), chronicle, "and the chronicle");
    assert!(sim.pending_orders().is_empty(), "and nothing is in flight");
}

#[test]
fn a_refused_order_is_not_charged() {
    // **The cost is charged when the order lands**, and an order that was
    // refused never lands.
    let mut sim = campaign();
    let before = sim.state().treasury.balance;
    let _ = sim.issue(OrderKind::Fire { person: GONE });
    assert_eq!(sim.state().treasury.balance, before);
}

#[test]
fn an_order_that_becomes_impossible_is_refused_when_it_lands() {
    // **The second validation.** The world moves on between the issue and the
    // landing, and an order that can no longer be honoured is refused *then* -
    // and the cost is not charged.
    let mut sim = campaign();
    let person = *sim.state().people.keys().next().expect("there is a person");

    // **The person is let go, and then a shift change is ordered for them.**
    // The fire lands at once and the shift change lands an hour later, by which
    // time the person is gone.
    sim.issue(OrderKind::Fire { person })
        .expect("the dismissal is accepted");
    sim.issue(OrderKind::SetShift {
        person,
        shift: Some(150 * domain::scale::PERCENT_UNITS as u16),
    })
    .expect("the shift change is accepted");
    assert_eq!(sim.pending_orders().len(), 2, "both are in flight");

    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    assert!(
        sim.pending_orders().is_empty(),
        "the refused order is not left in flight"
    );
    assert!(
        !sim.state().people.contains_key(&person),
        "and the person is gone, as the dismissal said"
    );
}

#[test]
fn a_refusal_at_landing_is_reported_to_the_player() {
    // **The player has to be able to act on a refusal.** A refusal at issue is
    // delivered at once, because the player is standing right there; a refusal
    // at landing is delivered in the report, because the player was not.
    let mut sim = campaign();
    let person = *sim.state().people.keys().next().expect("there is a person");
    sim.issue(OrderKind::Fire { person })
        .expect("the dismissal is accepted");
    sim.issue(OrderKind::SetShift {
        person,
        shift: Some(150 * domain::scale::PERCENT_UNITS as u16),
    })
    .expect("the shift change is accepted");

    let reports = sim
        .advance_to(Instant::from_days(40))
        .expect("the campaign advances");
    let reported = reports.iter().any(|report| {
        report
            .rejections
            .iter()
            .any(|rejection| rejection.contains("no longer with the agency"))
    });
    assert!(reported, "and the report says why: {reports:?}");
}

#[test]
fn a_postponed_deadline_on_a_missing_objective_is_refused() {
    let mut sim = campaign();
    let error = sim
        .issue(OrderKind::PostponeDeadline {
            programme: *sim.state().programmes.keys().next().expect("a programme"),
            objective: ObjectiveId(999),
            to: Instant::from_months(60),
        })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::Rejected { .. }),
        "and the refusal says why: {error}"
    );
}

#[test]
fn an_objective_with_a_future_date_is_accepted() {
    // **The positive case of the deadline.** A commitment in the future is
    // accepted, and it goes on the scheduler, and it will fall due.
    let mut sim = campaign();
    let programme = *sim.state().programmes.keys().next().expect("a programme");
    sim.issue(OrderKind::SetObjective {
        programme,
        description: "Reach orbit.".to_owned(),
        commitment: Commitment::new(
            domain::ids::CommitmentId(0),
            Some(Instant::from_months(12)),
            Some(Instant::from_months(24)),
            DeadlineAudience::Political,
        ),
        kind: ObjectiveKind::Primary,
    })
    .expect("the order is accepted");
    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    let state = sim.state();
    let programme = state.programmes.get(&programme).expect("the programme");
    assert!(
        programme
            .objectives
            .iter()
            .any(|objective| objective.description == "Reach orbit."),
        "and the objective is on the programme"
    );
}
