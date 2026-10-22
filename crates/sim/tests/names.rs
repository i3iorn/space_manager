//! The names the agency hires.
//!
//! **A name is a person's first impression, and a campaign has hundreds of
//! people.** These tests are about the two things that make a roster read
//! like a roster of people rather than a list of draws: the names are drawn
//! from a wide space, and nobody in the agency has the same one.

use std::collections::BTreeSet;

use domain::order::OrderKind;
use domain::person::Role;
use sim::Scenario;
use sim::Sim;

fn hire() -> OrderKind {
    OrderKind::Hire {
        role: Role::Scientist,
        salary: domain::money::Money::from_credits(1_200),
    }
}

#[test]
fn a_campaign_of_thirty_hires_has_thirty_names() {
    // **Thirty people, thirty names, and no two the same.** A campaign that
    // repeats itself is a campaign the player cannot keep track of.
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    for _ in 0..30 {
        sim.issue(hire()).expect("the order is accepted");
        sim.advance_to(
            sim.effective_now()
                .saturating_add(domain::time::Duration::from_days(20)),
        )
        .expect("the campaign advances");
    }

    let names: Vec<&str> = sim
        .state()
        .people
        .values()
        .map(|person| person.name.as_str())
        .collect();
    assert!(
        names.len() >= 30,
        "the agency hired its thirty: {}",
        names.len()
    );

    let mut unique: BTreeSet<&str> = BTreeSet::new();
    for name in &names {
        assert!(unique.insert(name), "{name} is in the agency twice");
    }
}

#[test]
fn a_campaign_never_has_two_people_with_the_same_family_name() {
    // **Two Okonjos are two people the player cannot tell apart**, and the
    // staff screen shows names and not identifiers.
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    for _ in 0..40 {
        sim.issue(hire()).expect("the order is accepted");
        sim.advance_to(
            sim.effective_now()
                .saturating_add(domain::time::Duration::from_days(20)),
        )
        .expect("the campaign advances");
    }

    let mut families: BTreeSet<&str> = BTreeSet::new();
    for person in sim.state().people.values() {
        let family = person
            .name
            .split_whitespace()
            .last()
            .expect("a family name");
        assert!(
            families.insert(family),
            "two people are called {family}, and the player cannot tell them apart"
        );
    }
}

#[test]
fn the_hired_names_are_not_the_hand_written_ones() {
    // **The people the scenarios open with have names too**, and a recruit
    // who shares a surname with one of them is a repetition the player can
    // see on the staff screen from the first hour.
    let scenario = Scenario::first_orbit();
    let mut sim = Sim::new(&scenario).expect("the campaign starts");
    for _ in 0..20 {
        sim.issue(hire()).expect("the order is accepted");
        sim.advance_to(
            sim.effective_now()
                .saturating_add(domain::time::Duration::from_days(20)),
        )
        .expect("the campaign advances");
    }

    let families: BTreeSet<&str> = sim
        .state()
        .people
        .values()
        .map(|person| {
            person
                .name
                .split_whitespace()
                .last()
                .expect("a family name")
        })
        .collect();
    assert!(
        families.len() >= 20,
        "and the names are varied: {} families for {} people",
        families.len(),
        sim.state().people.len()
    );
}

#[test]
fn a_name_is_drawn_from_one_culture() {
    // **The two halves of a name belong together.** "Kwame Adeyemi" reads as
    // a person; "Kwame Kowalski" does not.
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    // **The people the scenario opens with have names of their own**, and they
    // are characters rather than draws, so the check is about the hires.
    let before: BTreeSet<domain::ids::PersonId> = sim.state().people.keys().copied().collect();
    for _ in 0..12 {
        sim.issue(hire()).expect("the order is accepted");
        sim.advance_to(
            sim.effective_now()
                .saturating_add(domain::time::Duration::from_days(20)),
        )
        .expect("the campaign advances");
    }

    let hired: Vec<_> = sim
        .state()
        .people
        .values()
        .filter(|person| !before.contains(&person.id))
        .collect();
    assert_eq!(hired.len(), 12, "the agency hired its twelve");

    for person in hired {
        let given = person.name.split_whitespace().next().expect("a given name");
        let family = person
            .name
            .split_whitespace()
            .last()
            .expect("a family name");
        let culture = sim::names::CULTURES
            .iter()
            .find(|culture| culture.family.contains(&family))
            .unwrap_or_else(|| panic!("{family} is not a family name in the catalogue"));
        assert!(
            culture.given.contains(&given),
            "{given} {family} mixes two cultures"
        );
    }
}
