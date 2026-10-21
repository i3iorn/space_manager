//! The determinism test.
//!
//! **The most important test in the project.** The same campaign, run twice,
//! must produce the same state — every time, on every machine.
//!
//! It is fast, and it catches nearly every determinism bug before anything else
//! does: a `HashMap` iteration, a stray wall-clock read, a float comparison, an
//! unseeded generator.
//!
//! See `docs/TESTING.md` §3.

use std::collections::BTreeMap;

use sim::{Scenario, Sim};

/// The scenarios the determinism test runs.
const SCENARIOS: [&str; 2] = ["first_year", "sandbox"];

/// How many months the determinism test resolves.
const MONTHS: u32 = 24;

/// A campaign run, for comparison.
fn run(scenario_id: &str, seed: u64) -> (u64, Vec<String>) {
    let mut scenario = Scenario::by_id(scenario_id).expect("the scenario exists");
    scenario.seed = seed;

    let mut sim = Sim::new(&scenario).expect("the scenario starts");
    for month in 0..MONTHS {
        // A hire in the first month, so that the seed reaches the random source
        // and the recruits differ between seeds.
        let orders = if month == 0 {
            vec![domain::order::OrderKind::Hire {
                role: domain::person::Role::Scientist,
                salary: domain::money::Money::from_credits(1_200),
            }]
        } else {
            Vec::new()
        };
        sim.step(orders).expect("the month resolves");
    }

    let chronicle = sim
        .state()
        .chronicle
        .iter()
        .map(|entry| format!("{} {}", entry.date, entry.headline))
        .collect();

    (sim.state_hash(), chronicle)
}

#[test]
fn determinism_the_same_campaign_twice_is_identical() {
    for scenario in SCENARIOS {
        let (first_hash, first_chronicle) = run(scenario, 12_345);
        let (second_hash, second_chronicle) = run(scenario, 12_345);

        assert_eq!(
            first_hash, second_hash,
            "the {scenario} campaign hashed differently on a second run"
        );
        assert_eq!(
            first_chronicle, second_chronicle,
            "the {scenario} campaign produced a different history on a second run"
        );
    }
}

#[test]
fn determinism_the_same_campaign_is_identical_after_a_save_and_load() {
    let mut scenario = Scenario::by_id("first_year").expect("the scenario exists");
    scenario.seed = 99;

    let mut original = Sim::new(&scenario).expect("the scenario starts");
    for _ in 0..12 {
        original.step(Vec::new()).expect("the month resolves");
    }

    let text = original.save().expect("the campaign saves");
    let mut restored = Sim::load(&text).expect("the campaign loads");

    for _ in 0..12 {
        original.step(Vec::new()).expect("the month resolves");
        restored.step(Vec::new()).expect("the month resolves");
    }

    assert_eq!(original.state_hash(), restored.state_hash());
}

#[test]
fn determinism_different_seeds_give_different_campaigns() {
    let (first, _) = run("first_year", 1);
    let (second, _) = run("first_year", 2);
    assert_ne!(
        first, second,
        "two different seeds produced the same campaign, so the seed is not being used"
    );
}

#[test]
fn determinism_the_state_is_stable_under_reordering_of_the_collections() {
    // The state's collections are `BTreeMap`s, so their iteration order does not
    // depend on insertion order. This test builds the same state twice with the
    // entities inserted in different orders and checks the hashes match.
    let mut first = Scenario::by_id("first_year").expect("the scenario exists");
    first.seed = 4_242;
    let mut second = Scenario::by_id("first_year").expect("the scenario exists");
    second.seed = 4_242;

    let mut sim_first = Sim::new(&first).expect("the scenario starts");
    let mut sim_second = Sim::new(&second).expect("the scenario starts");

    for _ in 0..6 {
        sim_first.step(Vec::new()).expect("the month resolves");
        sim_second.step(Vec::new()).expect("the month resolves");
    }

    let mut first_people: BTreeMap<u64, String> = BTreeMap::new();
    for (id, person) in &sim_first.state().people {
        first_people.insert(id.0, person.name.clone());
    }
    let mut second_people: BTreeMap<u64, String> = BTreeMap::new();
    for (id, person) in &sim_second.state().people {
        second_people.insert(id.0, person.name.clone());
    }

    assert_eq!(first_people, second_people);
    assert_eq!(sim_first.state_hash(), sim_second.state_hash());
}

/// The same campaign, advanced in different-sized chunks.
///
/// **This is the dilation-independence test, and it is the one that matters.**
/// A campaign played at 1x is advanced in tiny pieces; the same campaign
/// played at 10,000x is advanced in huge ones. If the two come out different,
/// then something in the engine depends on how the host chopped up the time —
/// and the usual culprit is a random draw inside an integration sub-step,
/// because the number of sub-steps is the number of pieces the host happened
/// to use.
///
/// The full test at all nine dilations is M1-14. This is the mechanism, and it
/// is here because it is what M1-06 and M1-07 are for.
#[test]
fn determinism_the_campaign_does_not_depend_on_how_the_time_was_chopped_up() {
    /// A chunk, in simulated hours.
    const CHUNKS: [u64; 5] = [1, 7, 24, 168, 720];

    let mut hashes = Vec::new();
    for hours in CHUNKS {
        let mut scenario = Scenario::by_id("first_year").expect("the scenario exists");
        scenario.seed = 4_242;

        let mut sim = Sim::new(&scenario).expect("the scenario starts");
        sim.step(Vec::new()).expect("the month resolves");

        let chunk = domain::time::Duration::from_hours(hours);
        let target = domain::time::Instant::from_months(18);
        while sim.now() < target {
            // **The request is clamped to the target**, so every chunking
            // covers the same span. A chunk that carried the campaign past
            // the target would leave the fast runs covering more time than the
            // slow ones, and they would be different campaigns.
            let next = sim.effective_now().saturating_add(chunk).min(target);
            sim.advance_to(next).expect("the campaign advances");
        }

        hashes.push((hours, sim.state_hash()));
    }

    let first = hashes[0].1;
    for (hours, hash) in &hashes {
        assert_eq!(
            *hash, first,
            "advancing in {hours}-hour pieces gave a different campaign"
        );
    }
}

/// The same campaign, with the same orders, at two very different tempi.
///
/// The orders are issued at the same *simulated* instants either way. What
/// differs is the size of the pieces the host advances in, which is exactly
/// what a change of dilation does.
#[test]
fn determinism_the_campaign_does_not_depend_on_the_tempo() {
    let mut scenario = Scenario::by_id("first_year").expect("the scenario exists");
    scenario.seed = 77;

    let mut slow = Sim::new(&scenario).expect("the scenario starts");
    let mut fast = Sim::new(&scenario).expect("the scenario starts");

    for month in 0..12_u64 {
        let at = domain::time::Instant::from_months(month);
        // The slow host advances an hour at a time; the fast one advances a
        // week at a time. The orders land on the same simulated instants.
        while slow.now() < at {
            let next = slow
                .now()
                .saturating_add(domain::time::Duration::from_hours(1));
            slow.advance_to(next.min(at)).expect("advances");
        }
        while fast.now() < at {
            let next = fast
                .now()
                .saturating_add(domain::time::Duration::from_hours(168));
            fast.advance_to(next.min(at)).expect("advances");
        }
        slow.step(Vec::new()).expect("the month resolves");
        fast.step(Vec::new()).expect("the month resolves");
    }

    assert_eq!(slow.state_hash(), fast.state_hash());
}
