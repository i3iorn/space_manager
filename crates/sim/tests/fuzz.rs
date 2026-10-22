//! The fuzz test: random order sequences, and what they must never do.
//!
//! **The MVP's success criterion ten: "No crash, no soft-lock, no
//! unrecoverable state, from any order sequence."** A player is going to try
//! things the designer did not think of, and the engine has to survive all of
//! them. The orders are validated, but validation is the *first* line of
//! defence and not the only one.
//!
//! The sequences are **deterministic**: the same seed gives the same orders
//! every time, so a failure found here can be reproduced exactly.

use domain::ids::PersonId;
use domain::order::OrderKind;
use domain::person::{Assignment, ProgrammeRole, Role};
use domain::research::{CapabilityId, DisciplineId};
use domain::time::Instant;
use domain::vehicle::{ArchetypeId, SliderValues};
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use sim::Scenario;
use sim::Sim;

/// How many campaigns the fuzz test plays.
///
/// **A number, not a range.** A nightly job can afford thousands, and a test
/// has to be quick, so the test plays forty and the nightly job plays ten
/// thousand.
const CAMPAIGNS: u64 = 40;

/// How many months each campaign runs for.
const MONTHS: u64 = 24;

/// A random order, drawn from the kinds the engine has handlers for.
///
/// **The orders are nonsense as often as they are not.** The point is to reach
/// the engine with things a player might try and the designer did not plan
/// for, and the ids are often wrong on purpose.
fn random_order(rng: &mut ChaCha12Rng, sim: &Sim) -> OrderKind {
    let people: Vec<PersonId> = sim.state().people.keys().copied().collect();
    let person = |rng: &mut ChaCha12Rng| -> PersonId {
        if people.is_empty() {
            PersonId(0)
        } else {
            people[sim::rng::draw_below(rng, people.len() as u32) as usize]
        }
    };
    let sometimes = |rng: &mut ChaCha12Rng| -> bool { sim::rng::draw_basis_points(rng) < 5_000 };

    match sim::rng::draw_below(rng, 14) {
        0 => OrderKind::Hire {
            role: Role::ALL[sim::rng::draw_below(rng, Role::ALL.len() as u32) as usize],
            salary: domain::money::Money::from_credits(
                900 + i64::from(sim::rng::draw_below(rng, 2_000)),
            ),
        },
        1 => OrderKind::Fire {
            person: person(rng),
        },
        2 => OrderKind::Assign {
            person: person(rng),
            to: Assignment::Programme {
                programme: domain::ids::ProgrammeId(u64::from(sim::rng::draw_below(rng, 4))),
                role: ProgrammeRole::Member,
            },
        },
        3 => OrderKind::Assign {
            person: person(rng),
            to: Assignment::Project {
                project: domain::ids::ResearchProjectId(u64::from(sim::rng::draw_below(rng, 4))),
            },
        },
        4 => OrderKind::Assign {
            person: person(rng),
            to: Assignment::Facility {
                facility: domain::ids::FacilityId(u64::from(sim::rng::draw_below(rng, 4))),
            },
        },
        5 => OrderKind::Unassign {
            person: person(rng),
        },
        6 => OrderKind::SetShift {
            person: person(rng),
            shift: Some(u16::try_from(sim::rng::draw_below(rng, 400)).unwrap_or(0)),
        },
        7 => OrderKind::Promote {
            person: person(rng),
        },
        8 => OrderKind::Train {
            person: person(rng),
            course: "Orbital mechanics".to_owned(),
        },
        9 => OrderKind::StartProject {
            discipline: DisciplineId(u8::try_from(sim::rng::draw_below(rng, 3)).unwrap_or(0)),
            principal_investigator: person(rng),
            title: "A project".to_owned(),
        },
        10 => OrderKind::StartTestCampaign {
            capability: CapabilityId(u8::try_from(sim::rng::draw_below(rng, 6)).unwrap_or(0)),
        },
        11 => OrderKind::DesignVehicle {
            archetype: ArchetypeId(u8::try_from(sim::rng::draw_below(rng, 3)).unwrap_or(0)),
            name: "A vehicle".to_owned(),
            sliders: SliderValues {
                propulsion: u8::try_from(sim::rng::draw_below(rng, 101)).unwrap_or(0),
                payload: u8::try_from(sim::rng::draw_below(rng, 101)).unwrap_or(0),
                life_support: u8::try_from(sim::rng::draw_below(rng, 101)).unwrap_or(0),
                redundancy: u8::try_from(sim::rng::draw_below(rng, 101)).unwrap_or(0),
                power: u8::try_from(sim::rng::draw_below(rng, 101)).unwrap_or(0),
                structure: u8::try_from(sim::rng::draw_below(rng, 101)).unwrap_or(0),
                avionics: u8::try_from(sim::rng::draw_below(rng, 101)).unwrap_or(0),
                heat_shield: u8::try_from(sim::rng::draw_below(rng, 101)).unwrap_or(0),
            },
        },
        12 => OrderKind::AuthorizeLaunch {
            vehicle: domain::ids::VehicleId(u64::from(sim::rng::draw_below(rng, 3))),
            destination: "Low orbit".to_owned(),
            crew: Vec::new(),
        },
        _ => {
            if sometimes(rng) {
                OrderKind::SetBudget {
                    programme: domain::ids::ProgrammeId(u64::from(sim::rng::draw_below(rng, 4))),
                    allocation: domain::money::Money::from_credits(i64::from(
                        sim::rng::draw_below(rng, 100_000),
                    )),
                }
            } else {
                OrderKind::PostponeDeadline {
                    programme: domain::ids::ProgrammeId(u64::from(sim::rng::draw_below(rng, 4))),
                    objective: domain::ids::ObjectiveId(u64::from(sim::rng::draw_below(rng, 4))),
                    to: Instant::from_months(u64::from(sim::rng::draw_below(rng, 72))),
                }
            }
        }
    }
}

/// Plays a campaign of random orders, and checks that it survives.
fn fuzz_one(seed: u64) {
    let mut scenario = Scenario::first_orbit();
    scenario.seed = seed;
    let mut sim = Sim::new(&scenario).expect("the campaign starts");
    let mut rng = ChaCha12Rng::seed_from_u64(seed);

    for month in 0..MONTHS {
        // **A handful of orders a month**, because a player does not give
        // fifty orders at once.
        for _ in 0..4 {
            let order = random_order(&mut rng, &sim);
            // **A refused order is information, not a failure.** The engine
            // has to survive it, and the reason has to be a sentence.
            if let Err(reason) = sim.issue(order) {
                let message = reason.to_string();
                assert!(
                    !message.is_empty(),
                    "a refusal has to say something: {message:?}"
                );
            }
        }
        sim.advance_to(Instant::from_months(month + 1))
            .expect("the campaign advances");

        // **The state has to be intact.** A fuzzed campaign that breaks an
        // invariant is a bug that a player would find.
        domain::invariant::check(sim.state()).unwrap_or_else(|violation| {
            panic!("the fuzzed campaign broke an invariant: {violation}")
        });
    }

    // **And the campaign can be saved and loaded.** A campaign that cannot be
    // saved is a campaign the player can lose.
    let text = sim.save().expect("the campaign saves");
    let restored = Sim::load(&text).expect("the campaign loads");
    assert_eq!(
        restored.state_hash(),
        sim.state_hash(),
        "a saved and loaded campaign is the same campaign"
    );
}

#[test]
fn random_order_sequences_never_break_the_campaign() {
    for seed in 0..CAMPAIGNS {
        fuzz_one(seed.wrapping_mul(2_654_435_761).wrapping_add(1));
    }
}

#[test]
#[ignore = "the nightly run plays two thousand campaigns"]
fn a_long_night_of_fuzzing() {
    // **The nightly run is the real fuzz test.** The one on every push is a
    // smoke test, and this is the one that finds the bug that matters.
    for seed in 0..2_000_u64 {
        fuzz_one(seed.wrapping_mul(2_654_435_761).wrapping_add(1));
    }
}

#[test]
fn a_fuzzed_campaign_is_deterministic() {
    // **The same seed gives the same campaign**, or a failure found by the
    // fuzzer could not be reproduced.
    let mut hashes = Vec::new();
    for _ in 0..2 {
        let mut scenario = Scenario::first_orbit();
        scenario.seed = 424_242;
        let mut sim = Sim::new(&scenario).expect("the campaign starts");
        let mut rng = ChaCha12Rng::seed_from_u64(99);
        for month in 0..12_u64 {
            for _ in 0..3 {
                let order = random_order(&mut rng, &sim);
                let _ = sim.issue(order);
            }
            sim.advance_to(Instant::from_months(month + 1))
                .expect("the campaign advances");
        }
        hashes.push(sim.state_hash());
    }
    assert_eq!(
        hashes[0], hashes[1],
        "the same orders give the same campaign"
    );
}
