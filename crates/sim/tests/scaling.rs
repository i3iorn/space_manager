//! The scaling guard.
//!
//! **The game is meant to reach three hundred people**, and the shipped
//! scenarios have twelve. This is the test that the cost of a campaign grows
//! with the size of the agency and not with the *square* of it — because the
//! difference between those two is the difference between a game that plays and
//! a game that needs a rewrite.

use std::time::{Duration, Instant};

use domain::time::Instant as GameInstant;
use sim::{Scenario, Sim};

/// How many months the benchmark campaigns run for.
///
/// **Two years is enough to measure the shape of the cost** and it keeps the
/// test quick in a debug build, which is five to twenty times slower than the
/// shipped one.
const MONTHS: u64 = 24;

/// How long a campaign of the given size takes.
fn campaign_time(staff: usize) -> Duration {
    let projects = (staff / 5).max(1);
    let facilities = (staff / 30).max(1);
    let scenario = Scenario::scaling(staff, projects, facilities);
    let mut sim = Sim::new(&scenario).expect("the campaign starts");
    let started = Instant::now();
    sim.advance_to(GameInstant::from_months(MONTHS))
        .expect("the campaign advances");
    started.elapsed()
}

#[test]
fn the_cost_of_a_campaign_grows_with_the_agency_and_not_with_its_square() {
    // **The guard.** A quadratic campaign is the thing that makes a project
    // need a rewrite, and the only way to catch it is to measure two sizes and
    // compare the ratio.
    //
    // The ratio is compared rather than a ceiling, so the test is about the
    // *shape* of the cost and not about how fast the machine is.
    let small = campaign_time(50);
    let large = campaign_time(200);
    let ratio = large.as_secs_f64() / small.as_secs_f64().max(0.001);

    // **Four times the staff.** Linear would be about four; quadratic would
    // be about sixteen. The ceiling is eight, which is between the two and
    // leaves room for a constant cost that does not scale.
    assert!(
        ratio < 8.0,
        "a campaign of two hundred took {large:?} and one of fifty took {small:?}, \
         which is a ratio of {ratio:.1} — the cost is growing faster than the agency"
    );
}

#[test]
fn a_big_agency_still_finishes_a_campaign_in_a_reasonable_time() {
    // **The absolute ceiling**, so that a change which makes everything
    // slower is noticed even if the shape is right.
    let elapsed = campaign_time(300);
    let ceiling = if cfg!(debug_assertions) {
        Duration::from_secs(30)
    } else {
        Duration::from_secs(4)
    };
    assert!(
        elapsed < ceiling,
        "a campaign of three hundred people took {elapsed:?}, and the ceiling is {ceiling:?}"
    );
}
