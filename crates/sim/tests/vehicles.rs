//! Designing a vehicle.
//!
//! **A vehicle is the player's decision made of parts.** The design is a set
//! of sliders, the sliders are bounded by the archetype and by what the agency
//! has developed, and the numbers that come out are derived from the sliders
//! and never stored as an authority.
//!
//! The negative cases matter as much as the positive ones: a design the
//! agency cannot legally make has to be refused, and a slider the agency has
//! not researched has to be worth zero.

use domain::order::OrderKind;
use domain::time::Instant;
use domain::vehicle::{ArchetypeId, Slider, SliderValues};
use sim::{Scenario, Sim};

fn campaign() -> Sim {
    Sim::new(&Scenario::first_year()).expect("the campaign starts")
}

/// A design of every slider at its maximum.
fn maximal() -> SliderValues {
    SliderValues {
        propulsion: 100,
        payload: 100,
        life_support: 100,
        redundancy: 100,
        power: 100,
        structure: 100,
        avionics: 100,
        heat_shield: 100,
    }
}

#[test]
fn a_vehicle_can_be_designed() {
    // **The positive case.** A design becomes a vehicle, and the vehicle's
    // numbers come out of the design.
    let mut sim = campaign();
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
    sim.advance_to(Instant::from_days(5))
        .expect("the campaign advances");

    let state = sim.state();
    let vehicle = state.vehicles.values().next().expect("a vehicle");
    assert_eq!(vehicle.name, "Orbital Launcher 1");
    assert_eq!(vehicle.sliders.propulsion, 80);
    assert!(vehicle.derived.delta_v > 0, "and it has a delta-v");
    assert!(
        vehicle.derived.cost_estimate > domain::money::Money::ZERO,
        "and it costs something"
    );
    assert_eq!(vehicle.cost, vehicle.derived.cost_estimate);
}

#[test]
fn the_design_is_bounded_before_it_is_stored() {
    // **The agency cannot own a vehicle it could not legally build.** A
    // design that asks for more than the archetype allows is brought inside
    // it, not refused.
    let mut sim = campaign();
    sim.issue(OrderKind::DesignVehicle {
        archetype: ArchetypeId(0),
        name: "Sounding Rocket 1".to_owned(),
        sliders: maximal(),
    })
    .expect("the design is accepted");
    sim.advance_to(Instant::from_days(5))
        .expect("the campaign advances");

    let state = sim.state();
    let vehicle = state.vehicles.values().next().expect("a vehicle");
    // **A sounding rocket cannot be crewed.**
    assert_eq!(vehicle.sliders.life_support, 0);
    assert_eq!(vehicle.sliders.heat_shield, 0);
    // **And the archetype's floor is respected.**
    assert_eq!(
        vehicle.sliders.propulsion, 80,
        "up to the archetype's ceiling"
    );
}

#[test]
fn a_subsystem_can_be_changed() {
    // **The player adjusts a design.** The change is bounded the same way the
    // design was, because a slider the agency has not developed does not
    // become available by being edited.
    let mut sim = campaign();
    sim.issue(OrderKind::DesignVehicle {
        archetype: ArchetypeId(1),
        name: "Orbital Launcher 1".to_owned(),
        sliders: SliderValues::zero(),
    })
    .expect("the design is accepted");
    sim.advance_to(Instant::from_days(5))
        .expect("the campaign advances");

    let vehicle = *sim.state().vehicles.keys().next().expect("a vehicle");
    sim.issue(OrderKind::SetSubsystem {
        vehicle,
        slider: Slider::Propulsion,
        value: 90,
    })
    .expect("the change is accepted");
    sim.advance_to(Instant::from_days(10))
        .expect("the campaign advances");

    let state = sim.state();
    let vehicle = state.vehicles.get(&vehicle).expect("the vehicle");
    assert_eq!(vehicle.sliders.propulsion, 90);
    assert!(
        vehicle.derived.delta_v > 0,
        "and the numbers moved with the slider"
    );
}

#[test]
fn a_change_to_a_vehicle_that_does_not_exist_is_refused() {
    // **The negative case.** A subsystem change names a vehicle, and a vehicle
    // that is not there cannot be changed.
    let mut sim = campaign();
    let error = sim
        .issue(OrderKind::SetSubsystem {
            vehicle: domain::ids::VehicleId(999),
            slider: Slider::Propulsion,
            value: 90,
        })
        .expect_err("the order is refused");
    assert!(
        matches!(error, sim::error::OrderError::NoSuchVehicle { .. }),
        "and the refusal says why: {error}"
    );
}

#[test]
fn a_heat_shield_needs_the_capability() {
    // **The capability gate.** A player cannot buy a heat shield before the
    // agency has developed re-entry, and the design is stored bounded.
    let mut sim = campaign();
    sim.issue(OrderKind::DesignVehicle {
        archetype: ArchetypeId(2),
        name: "Capsule 1".to_owned(),
        sliders: SliderValues {
            heat_shield: 90,
            ..SliderValues::zero()
        },
    })
    .expect("the design is accepted");
    sim.advance_to(Instant::from_days(5))
        .expect("the campaign advances");

    let state = sim.state();
    let vehicle = state.vehicles.values().next().expect("a vehicle");
    assert_eq!(
        vehicle.sliders.heat_shield, 0,
        "the heat shield is worth nothing without the capability"
    );
}

#[test]
fn the_derived_numbers_are_never_stored_as_an_authority() {
    // **The derived statistics are recomputed from the sliders**, so they
    // cannot drift from the design that produced them.
    let sliders = SliderValues {
        propulsion: 70,
        payload: 50,
        life_support: 30,
        redundancy: 40,
        power: 20,
        structure: 60,
        avionics: 50,
        heat_shield: 10,
    };
    let first = domain::vehicle::derive_stats(sliders, ArchetypeId(1));
    let second = domain::vehicle::derive_stats(sliders, ArchetypeId(1));
    assert_eq!(first, second, "and the same design gives the same numbers");
    assert_eq!(first.cost_estimate, second.cost_estimate);
}
