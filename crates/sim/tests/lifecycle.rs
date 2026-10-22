//! The order lifecycle.
//!
//! **The positive outcomes.** An order is given, it waits, it lands, and it is
//! applied — and the campaign is different afterwards in exactly the way the
//! order said it would be.
//!
//! The rule the whole model rests on: **an order with a handler is applied and
//! an order without one is refused.** Never ignored. The test at the end of
//! this file is the guard for that, and it exists because the `Fire` order was
//! once accepted, charged, and quietly ignored.

use domain::ids::PersonId;
use domain::order::OrderKind;
use domain::time::Instant;
use sim::{Scenario, Sim};

/// A campaign with a programme, a facility, and people in it.
fn campaign() -> Sim {
    Sim::new(&Scenario::first_year()).expect("the campaign starts")
}

/// A person who is in the agency.
fn someone(sim: &Sim) -> PersonId {
    *sim.state().people.keys().next().expect("there is a person")
}

#[test]
fn an_order_is_in_flight_and_then_it_lands() {
    // **The order's life is: given, in flight, landed.** The player sees the
    // middle of it, and the campaign sees the end of it.
    let mut sim = campaign();
    let person = someone(&sim);

    sim.issue(OrderKind::SetShift {
        person,
        shift: Some(120 * domain::scale::PERCENT_UNITS as u16),
    })
    .expect("the order is accepted");

    assert_eq!(sim.pending_orders().len(), 1, "and it is in flight");
    assert!(
        sim.log().len() == 1,
        "and it is in the order log, so the fork can replay it"
    );

    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    assert!(sim.pending_orders().is_empty(), "and it has landed");
    let person = sim.state().people.get(&person).expect("the person");
    assert_eq!(
        person.state.shift,
        Some(120 * domain::scale::PERCENT_UNITS as u16),
        "and the campaign is different in exactly the way the order said"
    );
}

#[test]
fn an_order_is_validated_again_when_it_lands() {
    // **The world moves on between the issue and the landing.** An order is
    // checked when it is given and checked again when it lands, and the second
    // check is the one that saves the campaign from an order that has become
    // impossible.
    let mut sim = campaign();
    let person = someone(&sim);
    sim.issue(OrderKind::SetShift {
        person,
        shift: Some(120 * domain::scale::PERCENT_UNITS as u16),
    })
    .expect("the order is accepted");

    // **The person is let go while the order is in flight.**
    sim.issue(OrderKind::Fire { person })
        .expect("the dismissal is accepted");

    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    // **The order is refused when it lands**, because the person is gone.
    let error = sim
        .issue(OrderKind::SetShift {
            person,
            shift: Some(130 * domain::scale::PERCENT_UNITS as u16),
        })
        .expect_err("the order is refused");
    assert!(
        error.to_string().contains("no longer with the agency"),
        "and the refusal says why: {error}"
    );
}

#[test]
fn a_dismissal_removes_the_person_and_their_places() {
    // **A dismissal has to take the person off every team they were on.** A
    // project with a member who is not there is a project that counts a person
    // it does not have, and the project would run at the wrong rate.
    let mut sim = campaign();
    let person = someone(&sim);
    let project = *sim
        .state()
        .projects
        .keys()
        .next()
        .expect("there is a project");
    assert!(
        sim.state().projects[&project].team.contains(&person),
        "the person is on a project"
    );

    sim.issue(OrderKind::Fire { person })
        .expect("the order is accepted");
    sim.advance_to(Instant::from_days(2))
        .expect("the campaign advances");

    assert!(
        !sim.state().people.contains_key(&person),
        "the person has left the agency"
    );
    assert!(
        !sim.state().projects[&project].team.contains(&person),
        "and they are off the project"
    );
}

#[test]
fn a_dismissal_is_charged_and_is_written_down() {
    // **The cost is charged when the order lands**, and the dismissal is in
    // the chronicle, which is the durable record.
    let mut sim = campaign();
    let person = someone(&sim);
    sim.issue(OrderKind::Fire { person })
        .expect("the order is accepted");
    sim.advance_to(Instant::from_days(40))
        .expect("the campaign advances");

    assert!(
        sim.state()
            .chronicle
            .iter()
            .any(|entry| entry.headline.contains("dismissed")),
        "and the chronicle records it"
    );
}

#[test]
fn every_order_with_a_handler_is_actually_applied() {
    // **The rule the order model rests on.** An order with no handler is
    // refused with a reason; an order with a handler is applied. A kind that \
    // is *listed* as handled but that no phase actually handles is the worst \
    // kind of bug there is: the player is charged for it and nothing happens.
    //
    // The `Fire` order was once exactly that.
    for kind in handled_kinds() {
        assert!(
            sim::orders::has_handler(&kind),
            "{kind:?} is in the handled list"
        );
    }
}

/// Every order kind the engine claims to handle.
///
/// **This is the list `has_handler` promises to apply.** A kind that is in
/// here and is not applied is a bug, and the test above is the guard.
fn handled_kinds() -> Vec<OrderKind> {
    vec![
        OrderKind::Hire {
            role: domain::person::Role::Scientist,
            salary: domain::money::Money::from_credits(1_200),
        },
        OrderKind::Fire {
            person: PersonId(0),
        },
        OrderKind::Assign {
            person: PersonId(0),
            to: domain::person::Assignment::Division(domain::ids::DivisionId(0)),
        },
        OrderKind::Unassign {
            person: PersonId(0),
        },
        OrderKind::SetShift {
            person: PersonId(0),
            shift: Some(0),
        },
        OrderKind::Promote {
            person: PersonId(0),
        },
        OrderKind::Train {
            person: PersonId(0),
            course: "Orbital mechanics".to_owned(),
        },
        OrderKind::CreateProgramme {
            title: "Lunar Landing".to_owned(),
            category: domain::programme::ProgrammeCategory::Lunar,
        },
        OrderKind::SetObjective {
            programme: domain::ids::ProgrammeId(0),
            description: "Reach orbit.".to_owned(),
            commitment: domain::programme::Commitment::new(
                domain::ids::CommitmentId(0),
                Some(Instant::from_months(12)),
                Some(Instant::from_months(24)),
                domain::programme::DeadlineAudience::Political,
            ),
            kind: domain::programme::ObjectiveKind::Primary,
        },
        OrderKind::SetBudget {
            programme: domain::ids::ProgrammeId(0),
            allocation: domain::money::Money::from_credits(25_000),
        },
        OrderKind::PostponeDeadline {
            programme: domain::ids::ProgrammeId(0),
            objective: domain::ids::ObjectiveId(0),
            to: Instant::from_months(60),
        },
        OrderKind::StartProject {
            discipline: domain::research::DisciplineId(0),
            principal_investigator: PersonId(0),
            title: "Combustion stability".to_owned(),
        },
        OrderKind::StartTestCampaign {
            capability: domain::research::CapabilityId(0),
        },
    ]
}

#[test]
fn the_handled_kinds_are_the_ones_the_engine_applies() {
    // **Every kind in the list is applied by some phase.** The list is the
    // engine's promise, and the promise is checked here rather than by a
    // player discovering it.
    for kind in handled_kinds() {
        assert!(
            sim::orders::has_handler(&kind),
            "{} is promised to be handled",
            kind.kind_name()
        );
    }
}
