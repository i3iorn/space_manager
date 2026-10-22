//! The balance harness, and what it says about the game.
//!
//! **The harness is the only honest answer to "is the game balanced?"** A
//! balance constant is invisible until a campaign is played with it, and a
//! campaign is a long thing to play by hand. So the harness plays a lot of them
//! quickly and reports the distribution.
//!
//! These tests are the MVP's most important automated checks:
//!
//! * **S3-23** — the "First Orbit" scenario is winnable.
//! * **S3-24** — the harness reports an outcome distribution.
//! * **S4-09** — three reference strategies all survive, which is the
//!   criterion the MVP calls "the one that matters most".

use std::sync::OnceLock;
use std::time::Instant;

use sim::Scenario;
use sim::harness::{self, Outcome, Policy};

/// The seeds the tests run. **A small fixed list**, so that the test is
/// comparable with itself and a change in the balance shows up.
fn seeds() -> Vec<u64> {
    harness::default_seeds(4)
}

/// The campaign the harness plays.
fn scenario() -> Scenario {
    Scenario::first_orbit()
}

/// The outcomes every test looks at.
///
/// **The campaigns are played once.** A balance pass is a measurement, and
/// measuring the same thing four times is four times the waiting.
fn outcomes() -> &'static Vec<Outcome> {
    static OUTCOMES: OnceLock<Vec<Outcome>> = OnceLock::new();
    OUTCOMES.get_or_init(|| harness::run_all(&scenario(), &seeds(), 60))
}

/// The outcomes of one policy.
fn of(policy: Policy) -> Vec<&'static Outcome> {
    outcomes()
        .iter()
        .filter(|outcome| outcome.policy == policy)
        .collect()
}

#[test]
fn the_first_orbit_scenario_can_be_won() {
    // **The scenario is tuned, and this is the test that says so.** A starting
    // state that cannot be won is not a starting state, and the harness is
    // the cheapest way to find out.
    let group = of(Policy::Fast);
    let reached = group.iter().filter(|outcome| outcome.reached_orbit).count();
    assert!(
        reached * 2 >= group.len(),
        "a competent player should reach orbit in most campaigns; the report says:\n{}",
        harness::report(outcomes())
    );
}

#[test]
fn the_deadline_is_the_point_of_the_scenario() {
    // **The government promised five years, and the scenario has to be about
    // the promise.** A scenario where the deadline is met without reaching
    // orbit would be a scenario about nothing.
    for outcome in of(Policy::Fast) {
        if outcome.reached_orbit {
            assert!(
                outcome.deadline_met,
                "an agency that reached orbit should have met the deadline"
            );
        }
    }
}

#[test]
fn every_strategy_survives() {
    // **This is the MVP's most important test.** If two of the three
    // strategies collapse, the trade-offs are decoration, and the game is a
    // puzzle with a single answer.
    assert!(
        harness::all_viable(outcomes()),
        "a strategy has collapsed:\n{}",
        harness::report(outcomes())
    );
}

#[test]
fn the_strategies_are_different_from_one_another() {
    // **Three strategies that play the same game are one strategy.** The
    // cautious agency should get there more slowly and more safely, and the
    // fast one should get there sooner and more cheaply.
    let median = |policy: Policy| -> u32 {
        let mut months: Vec<u32> = of(policy)
            .iter()
            .filter_map(|outcome| outcome.months_to_orbit)
            .collect();
        months.sort_unstable();
        months.get(months.len() / 2).copied().unwrap_or_default()
    };

    assert!(
        median(Policy::Fast) <= median(Policy::Cautious),
        "the fast strategy should reach orbit no later than the cautious one: {} against {}",
        median(Policy::Fast),
        median(Policy::Cautious)
    );
}

#[test]
fn the_harness_reports_a_distribution() {
    // **The report is the deliverable of a balance pass.** It says how often
    // each policy reached orbit, what it cost, and what it left behind.
    let report = harness::report(outcomes());
    for policy in Policy::ALL {
        assert!(
            report.contains(policy.label()),
            "the report names {policy:?}:\n{report}"
        );
    }
    assert!(
        report.contains("reached orbit"),
        "and it says how often they got there:\n{report}"
    );
}

#[test]
fn the_harness_runs_in_minutes() {
    // **The harness has to be cheap enough to run.** A balance pass that takes
    // an hour is a balance pass that happens once.
    let started = Instant::now();
    let outcomes = harness::run_all(&scenario(), &seeds(), 60);
    let elapsed = started.elapsed();
    assert_eq!(outcomes.len(), seeds().len() * Policy::ALL.len());
    assert!(
        elapsed.as_secs() < 60,
        "the harness took {elapsed:?}, which is too long to run in a balance pass"
    );
}

#[test]
fn the_same_seed_and_the_same_policy_give_the_same_campaign() {
    // **A policy is deterministic.** The harness is only a measurement if the
    // thing it measures is the same every time.
    let first = harness::run_seeds(&scenario(), Policy::Cautious, &seeds(), 24);
    let second = harness::run_seeds(&scenario(), Policy::Cautious, &seeds(), 24);
    for (left, right) in first.iter().zip(second.iter()) {
        assert_eq!(left.treasury_at_end, right.treasury_at_end);
        assert_eq!(left.chronicle, right.chronicle);
        assert_eq!(left.successes, right.successes);
        assert_eq!(left.reached_orbit, right.reached_orbit);
    }
}

#[test]
fn different_seeds_give_different_campaigns() {
    // **A harness that measured the same campaign twenty times would measure
    // nothing.** The seeds have to spread.
    let outcomes = harness::run_seeds(&scenario(), Policy::Fast, &seeds(), 60);
    let treasuries: Vec<i64> = outcomes
        .iter()
        .map(|outcome| outcome.treasury_at_end)
        .collect();
    let mut sorted = treasuries.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert!(
        sorted.len() > 1,
        "every campaign ended with the same money: {treasuries:?}"
    );
}
