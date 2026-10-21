//! The shift, in the campaign.
//!
//! **The shift is a mechanic, not a number on a screen.** These tests are
//! about what it does to the campaign: the fatigue a person accrues, the
//! points a project earns, and the fact that overtime is never free.

use domain::ids::{DivisionId, PersonId};
use domain::person::Assignment;
use domain::time::Duration;
use sim::{balance, rates};

/// A campaign with one person on it.
fn state_with_one_person() -> domain::state::CampaignState {
    let mut state = domain::state::CampaignState::new(domain::state::Agency::new(
        domain::ids::AgencyId(0),
        "SEA".to_owned(),
        2039,
    ));
    state.people.insert(
        PersonId(0),
        domain::person::Person::new(
            PersonId(0),
            "Marsha Okonjo".to_owned(),
            domain::person::Role::Scientist,
            domain::money::Money::from_credits(1_200),
        ),
    );
    state
}

/// A shift, as a scaled percentage.
fn shift(percent: u32) -> u16 {
    (percent * domain::scale::PERCENT_UNITS as u32) as u16
}

#[test]
fn a_full_load_tires_at_the_ordinary_rate() {
    // **A full load is a full load.** The shift must not change the ordinary
    // case, or every number in the game moves.
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.assignment = Assignment::Division(DivisionId(0));
    person.state.shift = shift(100);
    rates::integrate(&mut state, Duration::from_months(1));
    assert_eq!(
        state.people[&PersonId(0)].state.fatigue,
        balance::FATIGUE_ON_ASSIGNMENT
    );
}

#[test]
fn a_light_load_recovers_while_the_person_works() {
    // **The point of the shift.** A person on light duties recovers while
    // they are still on the team, which is what the old way of easing off
    // could not do.
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.assignment = Assignment::Division(DivisionId(0));
    person.state.shift = shift(50);
    person.state.fatigue = (50 * domain::scale::PERCENT_UNITS as u32) as u16;
    rates::integrate(&mut state, Duration::from_months(1));
    assert!(
        state.people[&PersonId(0)].state.fatigue
            < (50 * domain::scale::PERCENT_UNITS as u32) as u16,
        "a person on light duties gets less tired, not more"
    );
}

#[test]
fn overtime_tires_faster_than_a_full_load() {
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.assignment = Assignment::Division(DivisionId(0));
    person.state.shift = shift(200);
    rates::integrate(&mut state, Duration::from_months(1));
    assert!(
        state.people[&PersonId(0)].state.fatigue > balance::FATIGUE_ON_ASSIGNMENT,
        "a person worked past a full load tires faster than one on a full load"
    );
}

#[test]
fn a_shift_change_is_an_order_with_a_one_hour_lead_time() {
    // **A shift change lands at the next hour**, because a shift change is a
    // rostering decision and not a project.
    let kind = domain::order::OrderKind::SetShift {
        person: PersonId(0),
        shift: shift(120),
    };
    let lead = sim::orders::base_lead_time(&kind, domain::time::Instant::ZERO);
    assert_eq!(lead, Duration::from_hours(balance::SHIFT_LEAD_TIME_HOURS));
    assert!(sim::orders::has_handler(&kind), "and it has a handler");
}

#[test]
fn a_shift_change_costs_nothing() {
    // **The player must be able to make a hundred of them** without the
    // state's patience running out.
    let kind = domain::order::OrderKind::SetShift {
        person: PersonId(0),
        shift: shift(120),
    };
    assert_eq!(sim::orders::goodwill(&kind), 0);
    let state = state_with_one_person();
    assert_eq!(sim::orders::cost(&kind, &state), domain::money::Money::ZERO);
}
