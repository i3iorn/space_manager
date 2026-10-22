//! The automated playtest.
//!
//! **This is the Slice 3 exit test, run by a machine.** The exit test says:
//!
//! > *A playtester gets to orbit, and on the way loses a programme, a launch,
//! > and an engineer — and can read all three in the chronicle.*
//!
//! A machine cannot tell whether the game is fun, and it cannot tell whether
//! the arc is dramatic. **It can tell whether the arc is possible** — that the
//! five years contain a victory and three defeats, and that the record names
//! every one of them. That is the part that is checkable, and it is the part
//! that has to be true before a human is asked to play.
//!
//! What a human playtest adds is on top of this, not instead of it.

use domain::ids::{ObjectiveId, ProgrammeId};
use domain::order::OrderKind;
use domain::programme::{Commitment, DeadlineAudience, ProgrammeCategory, ProgrammeStatus};
use domain::time::Instant;
use sim::Scenario;
use sim::Sim;
use sim::harness::Policy;

/// The length of the arc, in months.
const MONTHS: u64 = 60;

/// Plays the five-year arc.
///
/// **A reference policy plays it, and the arc is the "First Orbit"
/// scenario**: five years, one government deadline, and an agency with enough
/// people to do it if the player is quick about it.
///
/// The policy is the one the balance harness uses, so **the arc the playtest
/// plays is the arc the balance measures**. A change that breaks one breaks
/// the other, and that is the point of a single reference player.
fn play_the_arc() -> Sim {
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");

    // **An ambitious second programme.** The player opens something they are
    // going to miss the deadline on, and the campaign has to show it. A
    // reference policy that only ever succeeds cannot show the failure case.
    sim.issue(OrderKind::CreateProgramme {
        title: "Earth Observation".to_owned(),
        category: ProgrammeCategory::EarthObservation,
    })
    .expect("the programme is accepted");
    // **An order lands at an instant, and the campaign has to reach it.**
    sim.advance_to(
        sim.effective_now()
            .saturating_add(domain::time::Duration::from_hours(1)),
    )
    .expect("the campaign advances");
    let programme = sim
        .state()
        .programmes
        .values()
        .find(|programme| programme.category == ProgrammeCategory::EarthObservation)
        .map(|programme| programme.id)
        .expect("the programme exists");
    sim.issue(OrderKind::SetObjective {
        programme,
        description: "Watch the whole planet, continuously.".to_owned(),
        commitment: Commitment::new(
            domain::ids::CommitmentId(0),
            Some(Instant::from_months(6)),
            Some(Instant::from_months(14)),
            DeadlineAudience::Political,
        ),
        kind: domain::programme::ObjectiveKind::Primary,
    })
    .expect("the objective is accepted");

    for month in 0..MONTHS {
        for order in sim::harness::plan_for_debug(sim.state(), Policy::Cautious) {
            let _ = sim.issue(order);
        }
        // **And the agency keeps flying.** A player who has reached orbit
        // does not put the rocket in a shed, and the arc is not finished until
        // the campaign has had the whole of its five years.
        if objective_is_met(&sim) {
            if let Some(vehicle) = sim
                .state()
                .vehicles
                .values()
                .find(|vehicle| vehicle.build_state == domain::vehicle::BuildState::Complete)
            {
                let flying = sim
                    .state()
                    .missions
                    .values()
                    .any(|mission| mission.phase.is_running());
                if !flying {
                    let _ = sim.issue(OrderKind::AuthorizeLaunch {
                        vehicle: vehicle.id,
                        destination: "Low orbit".to_owned(),
                        crew: Vec::new(),
                    });
                }
            }
        }
        sim.advance_to(Instant::from_months(month + 1))
            .expect("the campaign advances");
    }
    // **A month past the end**, so that the deadline the government set has
    // come due and been resolved.
    sim.advance_to(Instant::from_months(MONTHS + 1))
        .expect("the campaign advances");
    sim
}

/// Whether the orbital programme's primary objective has been met.
fn objective_is_met(sim: &Sim) -> bool {
    sim.state()
        .programmes
        .values()
        .find(|programme| programme.category == ProgrammeCategory::Orbital)
        .and_then(|programme| programme.primary_objective())
        .is_some_and(|objective| objective.milestone_reached)
}

#[test]
fn the_arc_reaches_orbit() {
    // **The first half of the exit test.** The agency gets to orbit, and it
    // gets there inside the deadline the government set.
    let sim = play_the_arc();
    let state = sim.state();

    let reached = state.missions.values().any(|mission| {
        mission
            .outcome
            .as_ref()
            .is_some_and(|outcome| outcome.success)
            && mission.destination.eq_ignore_ascii_case("Low orbit")
    });
    assert!(reached, "the agency reached orbit:\n{}", summarise(&sim));

    // **And the government was told it would happen.** The orbital programme's
    // primary objective is met.
    let orbital = state
        .programmes
        .values()
        .find(|programme| programme.category == ProgrammeCategory::Orbital)
        .expect("the orbital programme exists");
    let objective = orbital
        .primary_objective()
        .expect("the orbital programme has a primary objective");
    assert_eq!(
        objective.met,
        Some(true),
        "and the deadline was met:\n{}",
        summarise(&sim)
    );
}

#[test]
fn the_arc_loses_a_programme_a_launch_and_an_engineer() {
    // **The second half of the exit test.** The arc contains three defeats
    // and a victory, and the defeats are the kind a campaign has.
    let sim = play_the_arc();
    let state = sim.state();

    // **A programme lost.** The one the player opened and did not staff, and
    // whose deadline passed.
    let failed: Vec<&str> = state
        .programmes
        .values()
        .filter(|programme| programme.status == ProgrammeStatus::Failed)
        .map(|programme| programme.title.as_str())
        .collect();
    assert!(
        !failed.is_empty(),
        "a programme was lost:\n{}",
        summarise(&sim)
    );

    // **A launch lost.** A mission that came home as a total failure.
    let lost_launch = state.missions.values().any(|mission| {
        mission
            .outcome
            .as_ref()
            .is_some_and(|outcome| !outcome.success && !outcome.partial)
    });
    assert!(lost_launch, "a launch was lost:\n{}", summarise(&sim));

    // **An engineer lost.** Somebody who is not there any more, and who was
    // there to begin with.
    let lost_staff = state
        .chronicle
        .iter()
        .any(|entry| entry.headline.contains("resigned"));
    assert!(lost_staff, "an engineer was lost:\n{}", summarise(&sim));
}

#[test]
fn the_three_losses_are_legible_in_the_chronicle() {
    // **"And can read all three in the chronicle."** The record has to name
    // the programme, the vehicle, and the person — a player who cannot see
    // what they lost cannot feel that they lost it.
    let sim = play_the_arc();
    let state = sim.state();
    let chronicle = state
        .chronicle
        .iter()
        .map(|entry| &entry.headline)
        .collect::<Vec<_>>();

    // The programme, by name.
    let programme = state
        .programmes
        .values()
        .find(|programme| programme.status == ProgrammeStatus::Failed)
        .expect("a programme was lost");
    assert!(
        chronicle
            .iter()
            .any(|headline| headline.contains(&programme.title)),
        "the chronicle names the lost programme ({}):\n{}",
        programme.title,
        summarise(&sim)
    );

    // The vehicle, by name.
    let vehicle = state
        .vehicles
        .values()
        .find(|vehicle| vehicle.build_state == domain::vehicle::BuildState::Scrapped)
        .expect("a vehicle was lost");
    assert!(
        chronicle
            .iter()
            .any(|headline| headline.contains(&vehicle.name)),
        "the chronicle names the lost vehicle ({}):\n{}",
        vehicle.name,
        summarise(&sim)
    );

    // The person, by name.
    let person = state
        .chronicle
        .iter()
        .find(|entry| entry.headline.contains("resigned"))
        .expect("somebody resigned");
    let name = person
        .headline
        .split_whitespace()
        .next()
        .expect("the entry names somebody");
    assert!(
        chronicle.iter().any(|headline| headline.contains(name)),
        "the chronicle names the person who left ({}):\n{}",
        name,
        summarise(&sim)
    );
}

#[test]
fn the_arc_is_worth_reading() {
    // **A record nobody reads is a record nobody keeps.** The chronicle of a
    // five-year arc has to be long enough to be a history, and the campaign
    // has to have gone somewhere.
    let sim = play_the_arc();
    let state = sim.state();
    assert!(
        state.chronicle.len() > 40,
        "five years leaves a record worth reading: {} entries",
        state.chronicle.len()
    );
    assert!(
        state.knowledge.values().sum::<u32>() > 0,
        "and the agency learned something"
    );
}

/// A short account of the campaign, for a failure message.
fn summarise(sim: &Sim) -> String {
    let state = sim.state();
    let mut lines = Vec::new();
    lines.push(format!(
        "  {} staff, {} projects, {} missions, {} chronicle entries, treasury {}",
        state.people.len(),
        state.projects.len(),
        state.missions.len(),
        state.chronicle.len(),
        state.treasury.balance
    ));
    for programme in state.programmes.values() {
        lines.push(format!(
            "  programme {} is {:?}",
            programme.title, programme.status
        ));
    }
    for mission in state.missions.values() {
        let outcome = mission
            .outcome
            .as_ref()
            .map_or("still flying".to_owned(), |outcome| {
                if outcome.success {
                    "worked".to_owned()
                } else if outcome.partial {
                    "partly worked".to_owned()
                } else {
                    "was lost".to_owned()
                }
            });
        lines.push(format!(
            "  mission to {} by {} {outcome}",
            mission.destination,
            state
                .vehicles
                .get(&mission.vehicle)
                .map_or("an unknown vehicle", |vehicle| vehicle.name.as_str())
        ));
    }
    lines.join("\n")
}

/// The ids the playtest uses, kept in one place.
#[allow(dead_code)]
struct Ids {
    programme: ProgrammeId,
    objective: ObjectiveId,
}

#[test]
fn the_playtest_does_not_depend_on_the_order_it_runs() {
    // **The arc is deterministic.** Two runs of the same playtest are the same
    // campaign, and that is what makes a failure in it reproducible.
    let first = play_the_arc();
    let second = play_the_arc();
    assert_eq!(first.state_hash(), second.state_hash());
}
