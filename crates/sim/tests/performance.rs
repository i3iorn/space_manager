//! The performance test.
//!
//! **The MVP's success criterion nine: "A full campaign resolves in under a
//! second."** The game is a real-time management game, and the interface has
//! to be able to re-simulate on demand — to fork a campaign, to replay an
//! order log, to answer "what would have happened if I had not?" — and none
//! of that is possible if a five-year campaign takes a minute.
//!
//! # The budget and the build
//!
//! **The criterion is about the shipped build.** A debug build is five to
//! twenty times slower, and a test that failed in CI and passed in a release
//! build would be a test nobody trusted. So the budget is the criterion in a
//! release build and a loose ceiling in a debug one, and the CI runs the
//! release check as a job of its own.

use std::time::{Duration, Instant};

use domain::time::Instant as GameInstant;
use sim::Scenario;
use sim::Sim;

/// The budget: **a full campaign resolves in under a second.**
///
/// **In the shipped build.** The debug ceiling is generous on purpose, and the
/// release job in CI is the one that holds the line.
fn budget() -> Duration {
    if cfg!(debug_assertions) {
        Duration::from_secs(15)
    } else {
        Duration::from_secs(1)
    }
}

/// How many months a full campaign is.
const MONTHS: u64 = 60;

/// Plays a full five-year campaign and says how long it took.
fn play_a_full_campaign() -> Duration {
    let scenario = Scenario::first_orbit();
    let mut sim = Sim::new(&scenario).expect("the campaign starts");
    let started = Instant::now();
    sim.advance_to(GameInstant::from_months(MONTHS))
        .expect("the campaign advances");
    started.elapsed()
}

#[test]
fn a_full_campaign_resolves_in_under_a_second() {
    // **The measurement is on the simulation, not on the setup**, because the
    // setup is the same every time and the simulation is the thing that has
    // to be fast.
    let elapsed = play_a_full_campaign();
    assert!(
        elapsed < budget(),
        "a five-year campaign took {elapsed:?}, and the budget is {:?}",
        budget()
    );
}

#[test]
fn the_interface_can_re_simulate_a_campaign_on_demand() {
    // **The interface forks campaigns**, and a fork replays the order log from
    // the beginning. If that takes a minute, the answer to "what if I had not
    // cancelled that programme?" is not worth having.
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    sim.advance_to(GameInstant::from_months(30))
        .expect("the campaign advances");
    for order in sim::harness::plan_for_debug(sim.state(), sim::harness::Policy::Fast) {
        let _ = sim.issue(order);
    }
    sim.advance_to(GameInstant::from_months(36))
        .expect("the campaign advances");

    // **The fork point is a month the campaign has reached.** The branch is
    // the same campaign up to that point - which the engine's own tests
    // assert - and what this test measures is how long the replay takes.
    let started = Instant::now();
    let branch = sim
        .fork(GameInstant::from_months(30))
        .expect("the campaign forks");
    let elapsed = started.elapsed();

    assert!(
        elapsed < budget(),
        "forking a campaign took {elapsed:?}, and the budget is {:?}",
        budget()
    );
    assert_eq!(
        branch.now(),
        GameInstant::from_months(30),
        "and the branch is at the fork point"
    );
}

#[test]
fn the_hourly_integration_grid_is_not_wasteful() {
    // **The grid is one hour**, and a five-year campaign is 43,800 of them.
    // Each one touches every person, every project, every facility and the
    // treasury, so the cost of the grid is the cost of the whole game.
    let elapsed = play_a_full_campaign();
    let quanta = MONTHS * 30 * 24;
    let per_quanta = elapsed.as_nanos() / u128::from(quanta);
    let ceiling = if cfg!(debug_assertions) {
        400_000
    } else {
        20_000
    };
    assert!(
        per_quanta < ceiling,
        "an hour of simulation took {per_quanta} nanoseconds, which is too slow"
    );
}
