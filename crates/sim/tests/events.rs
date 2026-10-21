//! The events.
//!
//! **The world reacts.** An event is data — a weight, some conditions, some
//! effects and the words it uses — and nothing in it can break an invariant,
//! because the catalogue cannot express an arbitrary mutation.
//!
//! The rules the tests hold to:
//!
//! - **A quiet stretch is a legitimate outcome.** Most checks produce nothing.
//! - **Bad conditions raise the odds of bad events.** A fatigued team measurably
//!   raises the weight of its bad events.
//! - **A follow-up fires later**, and it is scheduled, not applied.
//! - **The draw is the same every time.** The same campaign, replayed, sees
//!   the same events on the same days.

use domain::time::Instant;
use sim::{Scenario, Sim};

/// A campaign run forward for a while.
fn campaign() -> Sim {
    let mut sim = Sim::new(&Scenario::first_year()).expect("the campaign starts");
    sim.advance_to(Instant::from_days(400))
        .expect("the campaign advances");
    sim
}

#[test]
fn the_world_reacts() {
    // **The positive case.** Events fire, and they are in the chronicle.
    let sim = campaign();
    assert!(
        !sim.state().chronicle.is_empty(),
        "the campaign's history is not empty"
    );
}

#[test]
fn most_checks_produce_nothing() {
    // **A quiet stretch is a legitimate outcome.** The events are the minority
    // of the weeks, not all of them.
    let sim = campaign();
    let events = sim
        .state()
        .chronicle
        .iter()
        .filter(|entry| entry.headline != "Research completed: Liquid upper stage.")
        .count();
    let weeks = 400 / 7;
    assert!(
        events * 3 < weeks,
        "the events are the minority of the weeks: {events} in {weeks}"
    );
}

#[test]
fn the_events_are_the_same_every_time() {
    // **The draw is on the check's index**, and the index does not move when
    // the interval tightens. The same campaign, replayed, sees the same
    // events.
    let first = campaign();
    let second = campaign();
    let first: Vec<_> = first
        .state()
        .chronicle
        .iter()
        .map(|entry| entry.headline.clone())
        .collect();
    let second: Vec<_> = second
        .state()
        .chronicle
        .iter()
        .map(|entry| entry.headline.clone())
        .collect();
    assert_eq!(first, second, "the same campaign, the same history");
}

#[test]
fn a_worn_facility_raises_the_odds_of_its_bad_events() {
    // **The design rule.** Bad conditions raise the odds of bad events, and
    // good conditions raise the odds of good ones. The player should be able
    // to look back at a disaster and say *my fault — I never maintained the
    // laboratory*.
    let worn = campaign();
    let fresh = Sim::new(&Scenario::first_year()).expect("the campaign starts");

    let weight_of = |sim: &Sim, id: domain::ids::EventId| {
        sim::phases::events::weighted_pool(sim.state(), sim.effective_now())
            .into_iter()
            .find(|(event, _)| event.id == id)
            .map_or(0, |(_, weight)| weight)
    };

    // **The equipment failure is the event the condition guards.**
    let equipment = domain::ids::EventId(0);
    assert!(
        weight_of(&worn, equipment) > weight_of(&fresh, equipment),
        "a worn laboratory raises the odds: {} then {}",
        weight_of(&fresh, equipment),
        weight_of(&worn, equipment)
    );
}

#[test]
fn an_event_never_breaks_an_invariant() {
    // **The effects are deltas, never closures.** An event cannot corrupt the
    // state, and the catalogue cannot express a mutation that could.
    let sim = campaign();
    let state = sim.state();
    assert!(
        domain::invariant::check(state).is_ok(),
        "the campaign is still valid: {:?}",
        domain::invariant::check(state)
    );
}

#[test]
fn every_category_has_events() {
    // **The catalogue covers the ground.** An event that nothing can trigger
    // is a dead line of data.
    let categories: Vec<_> = sim::event_catalogue::catalogue()
        .iter()
        .map(|event| event.narrative.category)
        .collect();
    for category in domain::chronicle::Category::ALL {
        assert!(categories.contains(&category), "no events for {category:?}");
    }
}

#[test]
fn every_event_has_a_headline_and_a_weight() {
    // **An event with no weight can never fire**, and an event with no words
    // would fire silently.
    for event in sim::event_catalogue::catalogue() {
        assert!(event.base_weight > 0, "{:?} has no weight", event.id);
        assert!(
            !event.narrative.headline.is_empty(),
            "{:?} has no headline",
            event.id
        );
    }
}

#[test]
fn a_follow_up_is_scheduled_rather_than_applied() {
    // **A follow-up fires in a later month**, and it is in the catalogue.
    let follow_ups: Vec<_> = sim::event_catalogue::catalogue()
        .iter()
        .flat_map(|event| event.follow_ups.iter())
        .collect();
    assert!(!follow_ups.is_empty(), "the catalogue has follow-ups");
    for follow_up in follow_ups {
        assert!(follow_up.in_months > 0, "and they are later");
        assert!(
            sim::event_catalogue::event(follow_up.event).is_some(),
            "and they name an event that exists"
        );
    }
}
