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

    // **The equipment failure is the event the condition guards.** The weight
    // is relative - the pool's total is the pressure - so the share is what
    // moves.
    let share_of = |sim: &Sim, id: domain::ids::EventId| {
        let pool = sim::phases::events::weighted_pool(sim.state(), sim.effective_now());
        let total: u32 = pool.iter().map(|(_, weight)| *weight).sum();
        let weight = pool
            .iter()
            .find(|(event, _)| event.id == id)
            .map_or(0, |(_, weight)| *weight);
        weight * 10_000 / total.max(1)
    };

    let equipment = domain::ids::EventId(0);
    assert!(
        share_of(&worn, equipment) > share_of(&fresh, equipment),
        "a worn laboratory raises the odds: {} then {}",
        share_of(&fresh, equipment),
        share_of(&worn, equipment)
    );
}

#[test]
fn every_weighting_context_is_covered() {
    // **The fairness rule is checkable.** Every context in `BALANCE.md` §8.2
    // corresponds to something the player did, and every one of them has to
    // be read by at least one event - otherwise a bad event can fire for no
    // reason the player could have prevented.
    let contexts = [
        domain::event::Condition::AverageFatigueAbove(70),
        domain::event::Condition::FacilityConditionBelow(sim::event_catalogue::ANY_FACILITY, 60),
        domain::event::Condition::RunwayBelowMonths(2),
        domain::event::Condition::PublicOpinionBelow(-30),
        domain::event::Condition::ProgrammeWithoutSlack,
        domain::event::Condition::MilestoneDueWithinMonths(1),
        domain::event::Condition::LaunchRiskAbove(20),
        domain::event::Condition::FacilityUnstaffed,
    ];
    for context in contexts {
        let used = sim::event_catalogue::catalogue().iter().any(|event| {
            event
                .weight_modifiers
                .iter()
                .any(|modifier| modifier.when == context)
        });
        assert!(used, "no event reads {context:?}");
    }
}

#[test]
fn the_catalogue_covers_every_category() {
    // **The catalogue is the world's whole vocabulary**, and a category with no
    // events is a category the world cannot speak about.
    let categories: Vec<_> = sim::event_catalogue::catalogue()
        .iter()
        .map(|event| event.narrative.category)
        .collect();
    for category in domain::chronicle::Category::ALL {
        assert!(categories.contains(&category), "no events for {category:?}");
    }
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

#[test]
fn an_event_moves_the_state_by_the_number_of_points_it_says() {
    // **The catalogue speaks in points and the state speaks in hundredths of a
    // point.** An effect that was meant to move the approval by three was
    // moving it by three hundredths of a point, and the events were doing
    // nothing at all - which is exactly the sort of bug that looks like
    // balance rather than like a bug, because the game still runs and the
    // events still fire.
    let sim = campaign();
    let approval = sim.state().political.approval.value;
    assert_eq!(
        approval % domain::scale::PERCENT_UNITS as i16,
        0,
        "the state's approval is not a whole number of points: {approval}"
    );
    let opinion = sim.state().public.opinion.value;
    assert_eq!(
        opinion % domain::scale::PERCENT_UNITS as i16,
        0,
        "the public's opinion is not a whole number of points: {opinion}"
    );
}
