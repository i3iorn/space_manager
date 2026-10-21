//! The golden test.
//!
//! A golden test pins a **known campaign** and asserts the result. The
//! expected values live in files, not in the test code, and the expected
//! chronicle is stored as readable text — so when a golden test fails, the
//! developer can diff two chronicles and see exactly what changed in the world.
//!
//! **A failing golden test is a change, not an error.** The build fails, the
//! developer reads the diff, and either the change was intended (regenerate
//! the fixture and say so in the commit) or it was a bug (fix it). What must
//! never happen is a behaviour change landing quietly.
//!
//! See `docs/TESTING.md` §4.

use std::path::Path;

use domain::order::OrderKind;
use sim::{Scenario, Sim};

/// Where the fixtures live.
fn fixture_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("first_year")
}

/// Reads a fixture file.
fn read_fixture(name: &str) -> String {
    let path = fixture_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("the fixture {} could not be read: {error}", path.display()))
}

/// The orders the fixture campaign issues.
fn fixture_orders() -> Vec<Vec<OrderKind>> {
    let raw = read_fixture("log.json");
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).expect("the fixture log is valid JSON");

    parsed
        .as_array()
        .expect("the fixture log is an array of months")
        .iter()
        .map(|month| {
            month
                .as_array()
                .expect("each month is an array of orders")
                .iter()
                .map(|order| {
                    serde_json::from_value::<OrderKind>(order.clone())
                        .expect("each entry is a known order kind")
                })
                .collect()
        })
        .collect()
}

/// Runs the fixture campaign.
fn run_fixture() -> (u64, Vec<String>) {
    let mut scenario = Scenario::by_id("first_year").expect("the scenario exists");
    scenario.seed = read_fixture("seed.txt")
        .trim()
        .parse()
        .expect("the seed is a number");

    let mut sim = Sim::new(&scenario).expect("the scenario starts");
    for month in fixture_orders() {
        sim.step(month).expect("the month resolves");
    }

    let calendar = sim.calendar();
    let chronicle = sim
        .state()
        .chronicle
        .iter()
        .map(|entry| {
            format!(
                "{} | {} | {}",
                calendar.date(entry.date),
                entry.severity.label(),
                entry.headline
            )
        })
        .collect();

    (sim.state_hash(), chronicle)
}

#[test]
fn the_first_year_campaign_matches_the_golden_fixture() {
    let (hash, chronicle) = run_fixture();

    let expected_hash = read_fixture("expected_state_hash.txt");
    assert_eq!(
        hash.to_string(),
        expected_hash.trim(),
        "the campaign state changed. If the change was intended, regenerate the \
         fixture and review the chronicle diff."
    );

    let expected_chronicle = read_fixture("expected_chronicle.txt");
    assert_eq!(
        chronicle.join("\n"),
        expected_chronicle.trim_end(),
        "the campaign history changed. If the change was intended, regenerate the \
         fixture and review the diff."
    );
}

#[test]
fn the_golden_fixture_is_not_empty() {
    // A golden test that pins nothing is worse than no golden test, because it
    // looks like coverage.
    let chronicle = read_fixture("expected_chronicle.txt");
    assert!(
        !chronicle.trim().is_empty(),
        "the golden chronicle fixture is empty, so the test would pass for any campaign"
    );
    assert!(!read_fixture("expected_state_hash.txt").trim().is_empty());
}
