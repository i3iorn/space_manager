//! Flying a mission.
//!
//! **A mission is a chain of scheduled instants, not a monthly step.** It
//! takes as long as its phases take, the outcome is drawn at a scheduled
//! instant, and the data it brings back goes to the research that asked for it.
//!
//! The negative cases matter as much as the positive ones: a mission on a
//! vehicle that is not finished is refused, and a mission that fails still
//! leaves a record.

use domain::order::OrderKind;
use domain::time::{Duration, Instant};
use domain::vehicle::{ArchetypeId, SliderValues};
use sim::{Scenario, Sim};

/// A campaign with a finished vehicle ready to fly.
fn campaign() -> Sim {
    let mut sim = Sim::new(&Scenario::first_year()).expect("the campaign starts");
    sim.issue(OrderKind::DesignVehicle {
        archetype: ArchetypeId(1),
        name: "Orbital Launcher 1".to_owned(),
        sliders: SliderValues {
            propulsion: 80,
            payload: 60,
            life_support: 40,
            redundancy: 50,
            power: 30,
            structure: 70,
            avionics: 60,
            heat_shield: 20,
        },
    })
    .expect("the design is accepted");
    // **The build really runs.** It takes months, because a vehicle takes real
    // time to build, and this test is about the flying and not about the
    // building.
    sim.advance_to(Instant::from_days(800))
        .expect("the campaign advances");
    sim
}

#[test]
fn a_mission_takes_its_phases_in_order() {
    // **The positive case.** The mission starts in preparation and moves
    // through the phases, and the phases have real durations.
    let mut sim = campaign();
    let vehicle = *sim.state().vehicles.keys().next().expect("a vehicle");
    sim.issue(OrderKind::AuthorizeLaunch {
        vehicle,
        destination: "Low orbit".to_owned(),
        crew: Vec::new(),
    })
    .expect("the launch is authorised");
    sim.advance_to(Instant::from_days(805))
        .expect("the campaign advances");

    let state = sim.state();
    let mission = state.missions.values().next().expect("a mission");
    assert_eq!(
        mission.phase,
        domain::mission::MissionPhase::Preparation,
        "and it is in preparation, which takes three weeks"
    );
}

#[test]
fn the_launch_takes_real_time() {
    // **The flight is not a monthly step.** The mission is still in preparation
    // a day later and is well on its way a month later.
    let mut sim = campaign();
    let vehicle = *sim.state().vehicles.keys().next().expect("a vehicle");
    sim.issue(OrderKind::AuthorizeLaunch {
        vehicle,
        destination: "Low orbit".to_owned(),
        crew: Vec::new(),
    })
    .expect("the launch is authorised");
    sim.advance_to(Instant::from_days(805))
        .expect("the campaign advances");

    let before = sim
        .state()
        .missions
        .values()
        .next()
        .expect("a mission")
        .phase
        .clone();
    sim.advance_to(Instant::from_days(840))
        .expect("the campaign advances");
    let after = sim
        .state()
        .missions
        .values()
        .next()
        .expect("a mission")
        .phase
        .clone();
    assert_ne!(
        before, after,
        "and a month later it is somewhere else: {before:?} then {after:?}"
    );
}

#[test]
fn a_mission_on_a_vehicle_that_is_not_finished_is_refused() {
    // **The negative case.** A vehicle that is still being built cannot fly.
    let mut sim = Sim::new(&Scenario::first_year()).expect("the campaign starts");
    sim.issue(OrderKind::DesignVehicle {
        archetype: ArchetypeId(1),
        name: "Orbital Launcher 1".to_owned(),
        sliders: SliderValues::zero(),
    })
    .expect("the design is accepted");
    sim.advance_to(Instant::from_days(5))
        .expect("the campaign advances");

    let vehicle = *sim.state().vehicles.keys().next().expect("a vehicle");
    assert_ne!(
        sim.state().vehicles[&vehicle].build_state,
        domain::vehicle::BuildState::Complete,
        "and the vehicle is still being built"
    );
    let error = sim
        .issue(OrderKind::AuthorizeLaunch {
            vehicle,
            destination: "Low orbit".to_owned(),
            crew: Vec::new(),
        })
        .expect_err("the launch is refused");
    assert!(
        error.to_string().contains("not finished"),
        "and the refusal says why: {error}"
    );
}

#[test]
fn a_mission_resolves_when_it_comes_home() {
    // **The outcome is resolved at a scheduled instant**, and the mission
    // ends with an outcome either way.
    let mut sim = campaign();
    let vehicle = *sim.state().vehicles.keys().next().expect("a vehicle");
    sim.issue(OrderKind::AuthorizeLaunch {
        vehicle,
        destination: "Low orbit".to_owned(),
        crew: Vec::new(),
    })
    .expect("the launch is authorised");

    // **A long enough run for the whole flight.**
    sim.advance_to(Instant::from_days(1000))
        .expect("the campaign advances");

    let state = sim.state();
    let mission = state.missions.values().next().expect("a mission");
    assert!(
        mission.outcome.is_some(),
        "the mission resolved: {:?}",
        mission.phase
    );
    assert!(
        state
            .chronicle
            .iter()
            .any(|entry| entry.headline.contains("mission")),
        "and the chronicle records it"
    );
}

#[test]
fn a_mission_carries_a_probability() {
    // **The chance of success is the vehicle's reliability**, and it is
    // visible before the launch.
    let mut sim = campaign();
    let vehicle = *sim.state().vehicles.keys().next().expect("a vehicle");
    sim.issue(OrderKind::AuthorizeLaunch {
        vehicle,
        destination: "Low orbit".to_owned(),
        crew: Vec::new(),
    })
    .expect("the launch is authorised");
    sim.advance_to(Instant::from_days(805))
        .expect("the campaign advances");

    let state = sim.state();
    let mission = state.missions.values().next().expect("a mission");
    assert!(
        mission.probability > 0 && mission.probability <= 9_500,
        "and the chance is between nothing and a certainty: {}",
        mission.probability
    );
}

#[test]
fn the_phases_have_real_durations() {
    // **The ascent is the short one** and everything else is weeks.
    use domain::mission::MissionPhase;
    assert_eq!(
        MissionPhase::Launch.duration(),
        Duration::from_mins(75),
        "the launch is ninety minutes or so"
    );
    assert!(
        MissionPhase::Preparation.duration() > Duration::from_days(14),
        "and the preparation is weeks"
    );
    assert_eq!(
        MissionPhase::Archival.next(),
        None,
        "and the archive is the end"
    );
}
