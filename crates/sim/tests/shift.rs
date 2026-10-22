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
    person.state.shift = Some(shift(100));
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
    person.state.shift = Some(shift(50));
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
    person.state.shift = Some(shift(200));
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
        shift: Some(shift(120)),
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
        shift: Some(shift(120)),
    };
    assert_eq!(sim::orders::goodwill(&kind), 0);
    let state = state_with_one_person();
    assert_eq!(sim::orders::cost(&kind, &state), domain::money::Money::ZERO);
}

#[test]
fn a_policy_with_no_threshold_moves_everybody() {
    // **The tool for an agency of three hundred people.** One standing
    // instruction, and everybody works what it says.
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.assignment = Assignment::Division(DivisionId(0));
    state.shift_policy = Some(domain::person::Policy::Shift {
        shift: shift(50),
        only_over: None,
    });
    assert_eq!(
        sim::derived::effective_shift(&state, &state.people[&PersonId(0)]),
        shift(50)
    );
}

#[test]
fn a_policy_with_a_threshold_moves_only_the_tired() {
    // **The thermostat.** The people who are fresher than the threshold keep
    // their own shift, and the people who are tireder than it are eased off.
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.assignment = Assignment::Division(DivisionId(0));
    state.shift_policy = Some(domain::person::Policy::Shift {
        shift: shift(50),
        only_over: Some(60),
    });

    // **A person who is fresher than the threshold is not moved.**
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.state.fatigue = (40 * domain::scale::PERCENT_UNITS as u32) as u16;
    sim::shift::update_easing(person, state.shift_policy);
    assert_eq!(
        sim::derived::effective_shift(&state, &state.people[&PersonId(0)]),
        shift(100),
        "a person who is fresher than the threshold is not moved"
    );

    // **And a person who is tireder than it is eased off.**
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.state.fatigue = (70 * domain::scale::PERCENT_UNITS as u32) as u16;
    sim::shift::update_easing(person, state.shift_policy);
    assert_eq!(
        sim::derived::effective_shift(&state, &state.people[&PersonId(0)]),
        shift(50),
        "a person who is tireder than the threshold is eased off"
    );
}

#[test]
fn an_eased_person_stays_eased_until_they_have_recovered() {
    // **The margin is the whole of it.** A person who recovers to the limit
    // and tires again crosses it every few hours, and the agency gets half of
    // neither shift.
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.assignment = Assignment::Division(DivisionId(0));
    state.shift_policy = Some(domain::person::Policy::Shift {
        shift: shift(50),
        only_over: Some(60),
    });

    // **Eased**, because they are over the limit.
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.state.fatigue = (65 * domain::scale::PERCENT_UNITS as u32) as u16;
    sim::shift::update_easing(person, state.shift_policy);
    assert!(person.state.policy_eased, "the policy has eased them");

    // **Still eased**, because they have not recovered by the margin.
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.state.fatigue = (59 * domain::scale::PERCENT_UNITS as u32) as u16;
    sim::shift::update_easing(person, state.shift_policy);
    assert!(
        person.state.policy_eased,
        "they are still on light duties at fifty-nine percent"
    );

    // **Released**, because they have.
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    let release = 60 - balance::SHIFT_POLICY_BAND_PERCENT;
    person.state.fatigue = ((release - 1) * domain::scale::PERCENT_UNITS as u32) as u16;
    sim::shift::update_easing(person, state.shift_policy);
    assert!(
        !person.state.policy_eased,
        "and they are back on a full load at {release} percent"
    );
}

#[test]
fn the_policy_does_not_flip_on_and_off_around_the_limit() {
    // **The complaint that started this.** A person whose fatigue settles on
    // the limit had their shift flip on and off every hour, and the agency's
    // output flickered for a reason the player could not see.
    //
    // **A year of the policy is measured here**, and the number of changes is
    // the number that matters. Before the thermostat it was six hundred and
    // sixty-four changes a person a year; now it is a handful.
    let mut scenario = sim::Scenario::first_orbit();
    scenario.start_state.shift_policy = Some(domain::person::Policy::Shift {
        shift: shift(50),
        only_over: Some(60),
    });
    let mut sim = sim::Sim::new(&scenario).expect("the scenario starts");
    let mut last: std::collections::BTreeMap<PersonId, u16> = std::collections::BTreeMap::default();
    let mut changes = 0usize;
    for hour in 0..(24 * 365) {
        sim.advance_to(domain::time::Instant::from_hours(hour + 1))
            .expect("the campaign advances");
        for person in sim.state().people.values() {
            let shift = sim::derived::effective_shift(sim.state(), person);
            if let Some(previous) = last.insert(person.id, shift) {
                if previous != shift {
                    changes += 1;
                }
            }
        }
    }
    assert!(
        changes < 40,
        "the policy's shift changed {changes} times in a year, and it should be a handful"
    );
}

#[test]
fn a_person_the_manager_has_moved_is_not_the_policy_s_business() {
    // **The individual decides better than the policy**, and the policy's
    // memory must not outlive the decision it was made about.
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.assignment = Assignment::Division(DivisionId(0));
    person.state.shift = Some(shift(150));
    person.state.fatigue = (90 * domain::scale::PERCENT_UNITS as u32) as u16;
    state.shift_policy = Some(domain::person::Policy::Shift {
        shift: shift(50),
        only_over: Some(60),
    });
    sim::shift::update_easing(person, state.shift_policy);
    assert!(
        !person.state.policy_eased,
        "the policy does not ease somebody the manager has moved"
    );
    assert_eq!(
        sim::derived::effective_shift(&state, &state.people[&PersonId(0)]),
        shift(150),
        "and the manager's shift stands"
    );
}

#[test]
fn an_individual_shift_is_moved_off_the_policy() {
    // **The individual decides better than the policy.** A person the manager
    // has put on full duties is on full duties, whatever the agency says.
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.state.shift = Some(shift(150));
    state.shift_policy = Some(domain::person::Policy::Shift {
        shift: shift(50),
        only_over: None,
    });
    assert_eq!(
        sim::derived::effective_shift(&state, &state.people[&PersonId(0)]),
        shift(150)
    );
}

#[test]
fn no_policy_means_a_full_load() {
    // **The default must not move.** A campaign with no policy behaves exactly
    // as it did before the policy existed.
    let state = state_with_one_person();
    assert_eq!(
        sim::derived::effective_shift(&state, &state.people[&PersonId(0)]),
        shift(100)
    );
}

#[test]
fn the_policy_order_has_a_lead_time_and_a_price() {
    // **A policy is a standing instruction, and a standing instruction is
    // taken seriously.** It has a week of bureaucracy and it costs goodwill.
    let kind = domain::order::OrderKind::SetPolicy {
        policy: domain::person::Policy::Shift {
            shift: shift(50),
            only_over: Some(60),
        },
    };
    assert!(
        sim::orders::has_handler(&kind),
        "and it has a handler, because an order with no handler is refused"
    );
    assert_eq!(
        sim::orders::base_lead_time(&kind, domain::time::Instant::ZERO),
        Duration::from_days(balance::SET_POLICY_LEAD_TIME_DAYS)
    );
    assert_eq!(sim::orders::goodwill(&kind), balance::GOODWILL_POLICY);
    let state = state_with_one_person();
    assert_eq!(sim::orders::cost(&kind, &state), domain::money::Money::ZERO);
}

#[test]
fn a_person_can_be_put_back_on_the_policy() {
    // **The other half of the override.** A player who has pushed somebody to
    // a hundred and ten percent has to be able to say "she works flat out
    // until she is tired, and then she eases off", and that is what putting
    // her back on the policy means.
    let mut state = state_with_one_person();
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.state.shift = Some(shift(110));
    state.shift_policy = Some(domain::person::Policy::Shift {
        shift: shift(50),
        only_over: Some(60),
    });
    // **Put back on the policy**, which is the same as having no override.
    let person = state.people.get_mut(&PersonId(0)).expect("the person");
    person.state.shift = None;
    assert_eq!(person.state.shift_override(), None);
    assert_eq!(
        sim::derived::effective_shift(&state, &state.people[&PersonId(0)]),
        shift(100),
        "and the policy decides again"
    );
}

#[test]
fn a_shift_that_is_not_a_shift_is_refused() {
    // **An order that lands and does nothing is an order the player was lied
    // to about.** A shift outside the range is refused at issue.
    let kind = domain::order::OrderKind::SetShift {
        person: PersonId(0),
        shift: Some(u16::MAX),
    };
    let state = state_with_one_person();
    let error = sim::orders::validate(&kind, &state).expect_err("the order is refused");
    assert!(
        error.to_string().contains("shift"),
        "and it says why: {error}"
    );
}

#[test]
fn putting_somebody_back_on_the_policy_is_always_legal() {
    // **There is nothing illegal about letting the policy decide.**
    let kind = domain::order::OrderKind::SetShift {
        person: PersonId(0),
        shift: None,
    };
    let state = state_with_one_person();
    assert!(sim::orders::validate(&kind, &state).is_ok());
}

#[test]
fn the_policy_lands_and_takes_effect() {
    // **The whole feature, end to end.** The order is given, it lands after a
    // week of bureaucracy, and the shift it names is the shift the agency
    // works.
    let mut sim = sim::Sim::new(&sim::Scenario::first_year()).expect("the scenario starts");
    let person = *sim.state().people.keys().next().expect("a person");
    sim.issue(domain::order::OrderKind::SetPolicy {
        policy: domain::person::Policy::Shift {
            shift: shift(50),
            only_over: None,
        },
    })
    .expect("the order is accepted");
    assert_eq!(sim.pending_orders().len(), 1, "and it is in flight");

    sim.advance_to(domain::time::Instant::from_days(10))
        .expect("the campaign advances");
    assert_eq!(sim.pending_orders().len(), 0, "and it has landed");
    assert!(
        sim.state().shift_policy.is_some(),
        "and the policy is in force"
    );

    let state = sim.state();
    let person = &state.people[&person];
    assert_eq!(
        sim::derived::effective_shift(state, person),
        shift(50),
        "and the agency works what the policy says"
    );
}

#[test]
fn a_person_can_be_put_back_on_the_policy_by_an_order() {
    // **The other half of the override, as an order.** The player says "she
    // works flat out until she is tired", and the order says so.
    let mut sim = sim::Sim::new(&sim::Scenario::first_year()).expect("the scenario starts");
    let person = *sim.state().people.keys().next().expect("a person");
    sim.issue(domain::order::OrderKind::SetShift {
        person,
        shift: Some(shift(120)),
    })
    .expect("the order is accepted");
    sim.advance_to(domain::time::Instant::from_days(2))
        .expect("the campaign advances");
    assert_eq!(
        sim.state().people[&person].state.shift,
        Some(shift(120)),
        "and the person is on the shift the manager gave them"
    );

    sim.issue(domain::order::OrderKind::SetShift {
        person,
        shift: None,
    })
    .expect("the order is accepted");
    sim.advance_to(domain::time::Instant::from_days(4))
        .expect("the campaign advances");
    assert_eq!(
        sim.state().people[&person].state.shift,
        None,
        "and then they are back on the policy"
    );
}
