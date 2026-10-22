//! The domain's edges.
//!
//! **Every rule has two tests: one that it holds, and one that it catches.**
//! A rule with only the first is a rule nobody has checked, and the whole
//! point of the invariant catalogue is that a broken state is caught before it
//! becomes a strange campaign.
//!
//! The positive cases are as important as the negative ones. A catalogue that
//! catches everything is a catalogue that catches a *valid* campaign, and the
//! valid case is what proves the catalogue is not merely noisy.

use domain::ids::{AgencyId, FacilityId, PersonId, ProgrammeId};
use domain::invariant::{InvariantViolation, check};
use domain::money::Money;
use domain::person::{Person, Role};
use domain::state::{Agency, CampaignState};
use domain::time::{Duration, Instant};

/// A valid campaign.
fn state() -> CampaignState {
    let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
    state.people.insert(
        PersonId(0),
        Person::new(
            PersonId(0),
            "Marsha Okonjo".to_owned(),
            Role::Scientist,
            Money::from_credits(1_200),
        ),
    );
    state
}

/// The same, with a facility and a programme.
fn state_with_things() -> CampaignState {
    let mut state = state();
    state.facilities.insert(
        FacilityId(0),
        domain::facility::Facility {
            id: FacilityId(0),
            name: "Propulsion Lab".to_owned(),
            kind: domain::facility::FacilityType::Laboratory,
            site: domain::facility::Site {
                name: "The Core".to_owned(),
                publicly_visible: false,
            },
            specification: domain::facility::Specification {
                capacity: 4,
                reliability: 70,
                automation: 45,
                staff_needed: 2,
                quality: 65,
            },
            condition: (100 * domain::scale::PERCENT_UNITS as u32) as u16,
            staff: Vec::new(),
            construction: None,
            upkeep: Money::from_credits(4_000),
            monthly_output: 4,
            remainders: domain::scale::Remainders::new(),
        },
    );
    state
}

// ---------------------------------------------------------------- the catalogue

#[test]
fn a_valid_campaign_passes_the_catalogue() {
    // **The positive case of every rule at once.** A catalogue that rejected a
    // valid campaign would be a catalogue nobody could trust.
    let state = state_with_things();
    assert!(
        check(&state).is_ok(),
        "and it is valid: {:?}",
        check(&state)
    );
}

#[test]
fn the_catalogue_catches_an_out_of_range_attribute() {
    // **A negative case.** An attribute above a hundred is not a person.
    let mut state = state();
    state
        .people
        .get_mut(&PersonId(0))
        .expect("the person")
        .attributes
        .technical = 200;
    let violation = check(&state).expect_err("the catalogue catches it");
    assert!(
        matches!(violation, InvariantViolation::AttributeOutOfRange { .. }),
        "and names the attribute: {violation}"
    );
}

#[test]
fn the_catalogue_catches_an_out_of_range_state() {
    // **A negative case.** Morale, fatigue and health are all scaled, and all
    // of them have a range.
    for (morale, fatigue, health) in [(30_000_i16, 0_u16, 0_u16), (0, 30_000, 0), (0, 0, 30_000)] {
        let mut state = state();
        let person = state.people.get_mut(&PersonId(0)).expect("the person");
        person.state.morale = morale;
        person.state.fatigue = fatigue;
        person.state.health = health;
        let violation = check(&state).expect_err("the catalogue catches it");
        assert!(
            matches!(violation, InvariantViolation::StateOutOfRange { .. }),
            "and names the state: {violation}"
        );
    }
}

#[test]
fn the_catalogue_catches_an_inverted_commitment() {
    // **A negative case.** A commitment whose target is after its promise is
    // a commitment that was broken before it was made.
    let mut state = state_with_things();
    let mut programme = domain::programme::Programme::new(
        ProgrammeId(0),
        "Orbital Launch".to_owned(),
        domain::programme::ProgrammeCategory::Orbital,
    );
    programme.objectives.push(domain::programme::Objective::new(
        domain::ids::ObjectiveId(0),
        "Reach orbit.".to_owned(),
        domain::programme::ObjectiveKind::Primary,
        domain::programme::Commitment::new(
            domain::ids::CommitmentId(0),
            Some(Instant::from_months(24)),
            Some(Instant::from_months(12)),
            domain::programme::DeadlineAudience::Political,
        ),
    ));
    state.programmes.insert(ProgrammeId(0), programme);

    let violation = check(&state).expect_err("the catalogue catches it");
    assert!(
        matches!(violation, InvariantViolation::CommitmentInverted { .. }),
        "and names the commitment: {violation}"
    );
}

#[test]
fn the_catalogue_catches_a_budget_overrun() {
    // **A negative case.** A budget cannot spend more than it has.
    let mut state = state_with_things();
    let mut programme = domain::programme::Programme::new(
        ProgrammeId(0),
        "Orbital Launch".to_owned(),
        domain::programme::ProgrammeCategory::Orbital,
    );
    programme.budget.allocation = Money::from_credits(1_000);
    programme.budget.spent = Money::from_credits(10_000);
    state.programmes.insert(ProgrammeId(0), programme);

    let violation = check(&state).expect_err("the catalogue catches it");
    assert!(
        matches!(violation, InvariantViolation::BudgetOverrun { .. }),
        "and names the budget: {violation}"
    );
}

#[test]
fn the_catalogue_catches_a_half_built_facility() {
    // **A negative case.** A facility under construction has no specification
    // yet, and one with a specification is a building with a building on top.
    let mut state = state_with_things();
    {
        let facility = state
            .facilities
            .get_mut(&FacilityId(0))
            .expect("the facility");
        facility.construction = Some(domain::facility::Construction {
            plan: domain::facility::ConstructionPlan {
                method: domain::facility::BuildMethod::Standard,
                scale: domain::facility::BuildScale::Standard,
                site: domain::facility::Site {
                    name: "The Core".to_owned(),
                    publicly_visible: false,
                },
                financing: domain::facility::Financing::StateBudget,
                total_cost: Money::from_credits(100_000),
                build_months: 6,
            },
            progress: domain::research::Progress::empty(1_000),
            started: Instant::ZERO,
            expected_completion: Some(Instant::from_months(6)),
            remainders: domain::scale::Remainders::new(),
        });
    }
    let violation = check(&state).expect_err("the catalogue catches it");
    assert!(
        matches!(violation, InvariantViolation::FacilityHalfBuilt { .. }),
        "and names the facility: {violation}"
    );
}

#[test]
fn the_catalogue_catches_two_people_with_the_same_name() {
    // **A negative case.** Two people with the same name is a campaign where
    // the player cannot tell them apart.
    let mut state = state();
    let mut twin = Person::new(
        PersonId(1),
        "Marsha Okonjo".to_owned(),
        Role::Scientist,
        Money::from_credits(1_200),
    );
    twin.id = PersonId(1);
    state.people.insert(PersonId(1), twin);

    let violation = check(&state).expect_err("the catalogue catches it");
    assert!(
        matches!(violation, InvariantViolation::DuplicateName { .. }),
        "and names the name: {violation}"
    );
}

#[test]
fn the_catalogue_catches_a_dangling_subject() {
    // **A negative case.** A chronicle entry about somebody who is not there
    // is a record of the impossible.
    let mut state = state_with_things();
    let entry = domain::chronicle::ChronicleEntry::new(
        domain::ids::ChronicleId(0),
        Instant::ZERO,
        domain::chronicle::Category::Personnel,
        "Somebody did something.".to_owned(),
        domain::chronicle::Severity::Info,
    )
    .about(domain::chronicle::SubjectRef::Person(PersonId(999)));
    state.chronicle.push(entry);

    let violation = check(&state).expect_err("the catalogue catches it");
    assert!(
        matches!(violation, InvariantViolation::DanglingSubject { .. }),
        "and names the subject: {violation}"
    );
}

// -------------------------------------------------------------------- the money

#[test]
fn the_money_survives_being_divided() {
    // **The positive case.** A sum of a hundred and one credits is a hundred
    // and one credits, and the remainder is not lost.
    let money = Money::from_minor(101);
    let parts = money.split(2);
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], Money::from_minor(51));
    assert_eq!(parts[1], Money::from_minor(50));
    let total = parts
        .iter()
        .fold(Money::ZERO, |sum, part| sum.checked_add(*part).unwrap());
    assert_eq!(total, money, "and the parts add back to the whole");
}

#[test]
fn the_money_reports_an_overflow_rather_than_wrapping() {
    // **The negative case.** A ledger that wrapped around would be a ledger
    // that lied, and it must say so instead.
    let huge = Money::from_minor(i64::MAX);
    assert!(
        huge.checked_add(Money::from_credits(1)).is_none(),
        "and the addition is refused"
    );
    assert!(
        huge.checked_sub(Money::from_credits(-1)).is_none(),
        "and so is the subtraction"
    );
}

#[test]
fn the_money_of_nothing_is_nothing() {
    // **The positive case of zero**, which is where a division by zero would
    // otherwise live.
    assert!(Money::ZERO.is_zero());
    assert_eq!(Money::ZERO.minor(), 0);
    assert!(Money::ZERO.checked_add(Money::ZERO).is_some());
}

// ---------------------------------------------------------------------- the time

#[test]
fn a_duration_adds_and_subtracts() {
    let month = Duration::from_months(1);
    let year = Duration::from_months(12);
    assert_eq!(
        Instant::from_months(1).saturating_add(year),
        Instant::from_months(13)
    );
    assert_eq!(
        Instant::from_months(12).saturating_sub(month),
        Instant::from_months(11)
    );
}

#[test]
fn a_duration_never_goes_below_zero() {
    // **The negative case.** A duration cannot be negative, and the
    // subtraction that would make it so stops at zero.
    assert_eq!(
        Instant::from_hours(1).saturating_sub(Duration::from_hours(2)),
        Instant::ZERO
    );
}

#[test]
fn an_instant_before_another_is_a_shorter_distance() {
    let early = Instant::from_months(3);
    let late = Instant::from_months(9);
    assert_eq!(
        late.duration_since(early),
        Duration::from_months(6),
        "the distance between them"
    );
}

#[test]
fn the_civil_calendar_projects_an_instant_onto_a_date() {
    // **The positive case.** The agency was founded in 2039, and the calendar
    // knows it, and a date is what the player reads.
    let calendar = domain::time::Calendar::new(2039);
    let date = calendar.date(Instant::from_months(14));
    assert_eq!(date.year, 2040);
    assert_eq!(date.month, 3);
    assert_eq!(date.day, 1);
}

// ------------------------------------------------------------------ the person

#[test]
fn a_person_starts_in_reasonable_condition() {
    // **The positive case.** A new employee is not exhausted and not
    // miserable.
    let person = Person::new(
        PersonId(0),
        "Marsha Okonjo".to_owned(),
        Role::Scientist,
        Money::from_credits(1_200),
    );
    assert_eq!(person.state.fatigue, 0);
    assert!(person.state.morale > 0);
    assert_eq!(
        person.state.health,
        (100 * domain::scale::PERCENT_UNITS as u32) as u16
    );
    assert_eq!(
        person.state.shift, None,
        "and on whatever the agency policy says, which is a full load when it says nothing"
    );
}

#[test]
fn the_shift_is_clamped_to_what_a_person_can_be_put_on() {
    // **The negative case.** A player who asks for four hundred percent gets
    // two hundred.
    let maximum = (200 * domain::scale::PERCENT_UNITS as u32) as u16;
    assert_eq!(
        domain::person::PersonState::clamped_shift(u16::MAX),
        maximum
    );
    assert_eq!(domain::person::PersonState::clamped_shift(0), 0);
}

// --------------------------------------------------------------- the programme

#[test]
fn a_commitment_with_a_gap_has_slack() {
    // **The positive case.** The slack is the gap, and it is a duration.
    let commitment = domain::programme::Commitment::new(
        domain::ids::CommitmentId(0),
        Some(Instant::from_months(18)),
        Some(Instant::from_months(36)),
        domain::programme::DeadlineAudience::Political,
    );
    assert_eq!(commitment.slack(), Some(Duration::from_months(18)));
    assert_eq!(commitment.due(), Some(Instant::from_months(36)));
}

#[test]
fn a_commitment_with_no_gap_has_no_slack() {
    // **The negative case.** The plan and the promise on the same day leaves
    // nothing between them.
    let instant = Instant::from_months(12);
    let commitment = domain::programme::Commitment::new(
        domain::ids::CommitmentId(0),
        Some(instant),
        Some(instant),
        domain::programme::DeadlineAudience::Political,
    );
    assert_eq!(commitment.slack(), Some(Duration::ZERO));
}

#[test]
fn a_commitment_with_only_a_promise_has_no_slack() {
    // **The negative case.** A promise with no plan has nothing to measure the
    // slack against.
    let commitment = domain::programme::Commitment::committed(
        domain::ids::CommitmentId(0),
        Instant::from_months(12),
    );
    assert_eq!(commitment.slack(), None);
    assert_eq!(commitment.due(), Some(Instant::from_months(12)));
}

#[test]
fn a_commitment_due_in_the_past_is_due() {
    // **The edge case.** A commitment that is already due is due, and the
    // interface has to show it rather than pretend it is not.
    let past = Instant::from_months(1);
    let commitment = domain::programme::Commitment::committed(domain::ids::CommitmentId(0), past);
    assert!(commitment.is_due(Instant::from_months(2)));
    assert!(!commitment.is_due(Instant::ZERO));
}

// -------------------------------------------------------------- the facilities

#[test]
fn a_worn_facility_produces_less_than_a_new_one() {
    // **The positive case.** The specification is what it was built to be, and
    // the condition is what it is worth.
    let mut state = state_with_things();
    let output = state.facilities[&FacilityId(0)].effective_output();
    state
        .facilities
        .get_mut(&FacilityId(0))
        .expect("the facility")
        .condition = 0;
    assert_eq!(state.facilities[&FacilityId(0)].effective_output(), 0);
    assert!(output > 0, "and a new one produces something");
}
