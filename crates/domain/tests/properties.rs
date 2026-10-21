//! The property tests for the invariant catalogue.
//!
//! **The catalogue is only worth what it catches.** A checker that never fails is
//! decoration, so these tests do two things: they generate a great many states
//! and check that the catalogue is satisfied by all of them, and they break one
//! invariant at a time and check that the catalogue notices.
//!
//! The generator is **deterministic** — a fixed seed, a fixed sequence — so a
//! failure is reproducible and a passing run means the same thing tomorrow.
//! There is no property-testing framework here on purpose: the rules in
//! `AGENTS.md` say not to add one until there is a concrete need, and a
//! hundred lines of generator is cheaper than a dependency.

use domain::chronicle::{Category, ChronicleEntry, Severity, SubjectRef};
use domain::facility::{Construction, FacilityType, Specification};
use domain::finance::Budget;
use domain::ids::{
    AgencyId, ChronicleId, CommitmentId, FacilityId, ObjectiveId, PersonId, ProgrammeId,
};
use domain::invariant::{InvariantViolation, check};
use domain::money::Money;
use domain::person::{Person, Role};
use domain::programme::{
    Commitment, DeadlineAudience, Objective, ObjectiveKind, Programme, ProgrammeCategory,
};
use domain::state::{Agency, CampaignState};
use domain::time::Instant;

/// A small deterministic generator.
///
/// It is a xorshift, not a random source: **the test must be reproducible**, and
/// a failure that cannot be reproduced is a failure that cannot be fixed.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// A number in `0..=bound`.
    fn below(&mut self, bound: u64) -> u64 {
        self.next() % (bound + 1)
    }

    /// A number in `low..=high`.
    fn between(&mut self, low: i64, high: i64) -> i64 {
        let span = (high - low + 1) as u64;
        low + i64::try_from(self.below(span - 1)).unwrap_or(0)
    }
}

/// A campaign that satisfies every invariant.
/// A campaign that satisfies every invariant.
fn random_state(seed: u64) -> CampaignState {
    let mut rng = Rng::new(seed);
    let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
    add_people(&mut state, &mut rng);
    add_programmes(&mut state, &mut rng);
    add_facilities(&mut state, &mut rng);
    add_chronicle(&mut state, &mut rng);
    state
}

/// People, with attributes and state in range and distinct names.
fn add_people(state: &mut CampaignState, rng: &mut Rng) {
    let people = rng.below(6) as u32;
    for index in 0..people {
        let id = PersonId(u64::from(index));
        let mut person = Person::new(
            id,
            format!("Person {index}"),
            Role::ALL[(index as usize) % Role::ALL.len()],
            Money::from_credits(i64::try_from(rng.below(2_000)).unwrap_or(0)),
        );
        person.attributes.technical = rng.below(100) as u8;
        person.attributes.creative = rng.below(100) as u8;
        person.attributes.leadership = rng.below(100) as u8;
        person.attributes.physical = rng.below(100) as u8;
        person.attributes.diplomatic = rng.below(100) as u8;
        person.state.morale = rng.between(-25_600, 25_600) as i16;
        person.state.loyalty = rng.between(-25_600, 25_600) as i16;
        person.state.fame = rng.between(-25_600, 25_600) as i16;
        person.state.fatigue = rng.below(25_600) as u16;
        person.state.health = rng.below(25_600) as u16;
        state.people.insert(id, person);
    }
}

/// Programmes, with targets at or before their commitments, and budgets in range.
fn add_programmes(state: &mut CampaignState, rng: &mut Rng) {
    let programmes = rng.below(4) as u32;
    for index in 0..programmes {
        let id = ProgrammeId(u64::from(index));
        let mut programme =
            Programme::new(id, format!("Programme {index}"), ProgrammeCategory::Lunar);
        programme.budget = Budget::of(Money::from_credits(100_000));
        programme.budget.spent = Money::from_credits(i64::try_from(rng.below(99_999)).unwrap_or(0));

        let objectives = rng.below(3) as u32;
        for objective_index in 0..objectives {
            let target = Instant::from_months(rng.below(24));
            let committed = Instant::from_months(rng.below(30));
            // The target must not be later than the commitment.
            let (target, committed) = if target > committed {
                (committed, committed)
            } else {
                (target, committed)
            };
            programme.objectives.push(Objective::new(
                ObjectiveId(u64::from(objective_index)),
                format!("Objective {objective_index}"),
                ObjectiveKind::Primary,
                Commitment::new(
                    CommitmentId(0),
                    Some(target),
                    Some(committed),
                    DeadlineAudience::Political,
                ),
            ));
        }
        state.programmes.insert(id, programme);
    }
}

/// Facilities: either built, with a specification, or under construction
/// without one.
fn add_facilities(state: &mut CampaignState, rng: &mut Rng) {
    let facilities = rng.below(4) as u32;
    for index in 0..facilities {
        let id = FacilityId(u64::from(index));
        let mut facility = domain::facility::Facility {
            id,
            name: format!("Facility {index}"),
            kind: FacilityType::Laboratory,
            site: domain::facility::Site {
                name: "The Core".to_owned(),
                publicly_visible: false,
            },
            specification: Specification::empty(),
            condition: (100 * domain::scale::PERCENT_UNITS as u32) as u16,
            staff: Vec::new(),
            construction: None,
            upkeep: Money::from_credits(4_000),
            monthly_output: 0,
            remainders: domain::scale::Remainders::new(),
        };
        if rng.below(2) == 0 {
            // Under construction, and so without a specification.
            facility.construction = Some(Construction {
                plan: domain::facility::ConstructionPlan {
                    method: domain::facility::BuildMethod::Standard,
                    scale: domain::facility::BuildScale::Small,
                    site: domain::facility::Site {
                        name: "The Core".to_owned(),
                        publicly_visible: false,
                    },
                    financing: domain::facility::Financing::StateBudget,
                    total_cost: Money::from_credits(350_000),
                    build_months: 15,
                },
                started: Instant::ZERO,
                progress: domain::research::Progress::empty(25_600),
                expected_completion: None,
                remainders: domain::scale::Remainders::new(),
            });
        } else {
            facility.specification = Specification {
                capacity: 4,
                reliability: 70,
                automation: 45,
                staff_needed: 2,
                quality: 65,
            };
        }
        state.facilities.insert(id, facility);
    }
}

/// Chronicle entries, about things that exist.
fn add_chronicle(state: &mut CampaignState, rng: &mut Rng) {
    let entries = rng.below(5) as u32;
    for index in 0..entries {
        let mut entry = ChronicleEntry::new(
            ChronicleId(u64::from(index)),
            Instant::from_months(rng.below(24)),
            Category::Technical,
            format!("Something happened {index}"),
            Severity::Minor,
        );
        if let Some(id) = state.people.keys().next() {
            entry = entry.about(SubjectRef::Person(*id));
        }
        state.chronicle.push(entry);
    }
}

#[test]
fn every_generated_state_satisfies_the_catalogue() {
    for seed in 1..=500 {
        let state = random_state(seed);
        assert!(
            check(&state).is_ok(),
            "the generated state for seed {seed} broke an invariant: {:?}",
            check(&state)
        );
    }
}

#[test]
fn the_catalogue_catches_an_out_of_range_attribute() {
    for seed in 1..=50 {
        let mut state = random_state(seed);
        let Some(id) = state.people.keys().next().copied() else {
            continue;
        };
        state
            .people
            .get_mut(&id)
            .expect("the person")
            .attributes
            .technical = 200;
        assert!(matches!(
            check(&state),
            Err(InvariantViolation::AttributeOutOfRange { .. })
        ));
    }
}

#[test]
fn the_catalogue_catches_an_out_of_range_state() {
    for seed in 1..=50 {
        let mut state = random_state(seed);
        let Some(id) = state.people.keys().next().copied() else {
            continue;
        };
        state.people.get_mut(&id).expect("the person").state.morale = 30_000;
        assert!(matches!(
            check(&state),
            Err(InvariantViolation::StateOutOfRange { .. })
        ));
    }
}

#[test]
fn the_catalogue_catches_an_out_of_range_fatigue() {
    // **Fatigue is bounded, not signed** - it is a scale from nothing to
    // exhausted. A value past the top of the scale makes every comparison
    // against it wrong, and the symptom is a campaign where nobody is ever
    // tired.
    for seed in 1..=50 {
        let mut state = random_state(seed);
        let Some(id) = state.people.keys().next().copied() else {
            continue;
        };
        state.people.get_mut(&id).expect("the person").state.fatigue = 30_000;
        let violation = check(&state);
        assert!(
            matches!(violation, Err(InvariantViolation::StateOutOfRange { .. })),
            "seed {seed} gave {violation:?}"
        );
    }
}

#[test]
fn the_catalogue_catches_an_out_of_range_health() {
    for seed in 1..=50 {
        let mut state = random_state(seed);
        let Some(id) = state.people.keys().next().copied() else {
            continue;
        };
        state.people.get_mut(&id).expect("the person").state.health = 30_000;
        let violation = check(&state);
        assert!(
            matches!(violation, Err(InvariantViolation::StateOutOfRange { .. })),
            "seed {seed} gave {violation:?}"
        );
    }
}

#[test]
fn the_catalogue_catches_two_people_with_the_same_name() {
    for seed in 1..=50 {
        let mut state = random_state(seed);
        let Some(id) = state.people.keys().next().copied() else {
            continue;
        };
        let name = state.people[&id].name.clone();
        state.people.insert(PersonId(999), {
            let mut person = Person::new(
                PersonId(999),
                name,
                Role::Scientist,
                Money::from_credits(1_000),
            );
            person.state.morale = 0;
            person
        });
        let violation = check(&state);
        assert!(
            matches!(violation, Err(InvariantViolation::DuplicateName { .. })),
            "seed {seed} gave {violation:?}"
        );
    }
}

#[test]
fn the_catalogue_catches_an_inverted_commitment() {
    for seed in 1..=50 {
        let mut state = random_state(seed);
        let Some(id) = state.programmes.keys().next().copied() else {
            continue;
        };
        let programme = state.programmes.get_mut(&id).expect("the programme");
        if programme.objectives.is_empty() {
            programme.objectives.push(Objective::new(
                ObjectiveId(0),
                "An objective".to_owned(),
                ObjectiveKind::Primary,
                Commitment::new(
                    CommitmentId(0),
                    Some(Instant::from_months(30)),
                    Some(Instant::from_months(10)),
                    DeadlineAudience::Political,
                ),
            ));
        }
        programme.objectives[0].commitment = Commitment::new(
            CommitmentId(0),
            Some(Instant::from_months(30)),
            Some(Instant::from_months(10)),
            DeadlineAudience::Political,
        );
        let violation = check(&state);
        assert!(
            matches!(
                violation,
                Err(InvariantViolation::CommitmentInverted { .. })
            ),
            "seed {seed} gave {violation:?}"
        );
    }
}

#[test]
fn the_catalogue_catches_an_overspent_budget() {
    for seed in 1..=50 {
        let mut state = random_state(seed);
        let Some(id) = state.programmes.keys().next().copied() else {
            continue;
        };
        let programme = state.programmes.get_mut(&id).expect("the programme");
        programme.budget = Budget::of(Money::from_credits(1_000));
        programme.budget.spent = Money::from_credits(2_000);
        assert!(matches!(
            check(&state),
            Err(InvariantViolation::BudgetOverrun { .. })
        ));
    }
}

#[test]
fn the_catalogue_catches_a_half_built_facility() {
    for seed in 1..=50 {
        let mut state = random_state(seed);
        let Some(id) = state.facilities.keys().next().copied() else {
            continue;
        };
        let facility = state.facilities.get_mut(&id).expect("the facility");
        facility.specification = Specification {
            capacity: 4,
            reliability: 70,
            automation: 45,
            staff_needed: 2,
            quality: 65,
        };
        if facility.construction.is_none() {
            facility.construction = Some(Construction {
                plan: domain::facility::ConstructionPlan {
                    method: domain::facility::BuildMethod::Standard,
                    scale: domain::facility::BuildScale::Small,
                    site: domain::facility::Site {
                        name: "The Core".to_owned(),
                        publicly_visible: false,
                    },
                    financing: domain::facility::Financing::StateBudget,
                    total_cost: Money::from_credits(350_000),
                    build_months: 15,
                },
                started: Instant::ZERO,
                progress: domain::research::Progress::empty(25_600),
                expected_completion: None,
                remainders: domain::scale::Remainders::new(),
            });
        }
        let violation = check(&state);
        assert!(
            matches!(violation, Err(InvariantViolation::FacilityHalfBuilt { .. })),
            "seed {seed} gave {violation:?}"
        );
    }
}

#[test]
fn the_catalogue_catches_a_dangling_chronicle_subject() {
    for seed in 1..=50 {
        let mut state = random_state(seed);
        state.chronicle.push(
            ChronicleEntry::new(
                ChronicleId(999),
                Instant::ZERO,
                Category::Technical,
                "Something happened".to_owned(),
                Severity::Minor,
            )
            .about(SubjectRef::Person(PersonId(9_999))),
        );
        let violation = check(&state);
        assert!(
            matches!(violation, Err(InvariantViolation::DanglingSubject { .. })),
            "seed {seed} gave {violation:?}"
        );
    }
}
