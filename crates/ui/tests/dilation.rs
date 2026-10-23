//! The dilation-independence test.
//!
//! **The most important test in the project after the determinism test.** A
//! campaign played at the base rate and the same campaign played at the top rate
//! must be *the same campaign*: the same state, the same chronicle, the same
//! recruits.
//!
//! It is the test that catches a random draw inside an integration sub-step —
//! because the number of sub-steps is the number of pieces the host happened
//! to use, and the host uses different pieces at different dilations.
//!
//! It is also the test that catches an engine which behaves differently
//! depending on how finely the time was chopped up: a rate that depends on the
//! state, integrated over a gap that depends on the frame rate, gives a
//! different answer every time the frame rate changes.
//!
//! See `docs/SIMULATION.md` §6.3 and `docs/TESTING.md` §3.1.

use std::time::Duration as RealDuration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use domain::time::{Duration, Instant, TimeScale};
use sim::Scenario;
use ui::{App, clock::Clock};

/// The `Esc` key, with no modifiers.
fn esc() -> KeyEvent {
    KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
}

/// The host's frame time, in milliseconds.
///
/// The real interface uses sixteen milliseconds, but a month of simulated time
/// at the base rate is a hundred and sixty-two million frames, and a test
/// that takes a quarter of a minute is a test nobody runs. **The frame size is
/// a free parameter** — what matters is that the nine dilations use *very*
/// different ones, and the second test below checks exactly that.
const FRAME_MILLIS: u64 = 1_000;

/// How much simulated time the campaign covers.
const MONTHS: u64 = 1;

/// Runs the same campaign at one dilation, through the host's own path.
fn run_at(scale: TimeScale) -> (u64, Vec<String>) {
    let mut scenario = Scenario::by_id("first_year").expect("the scenario exists");
    scenario.seed = 4_242;

    let mut app = App::started(sim::Sim::new(&scenario).expect("the scenario starts"));
    app.clock_mut().set_scale(scale);

    // The host's frame: a fixed amount of *real* time, which the clock turns
    // into a fixed amount of *simulated* time. The nine dilations are nine
    // different sizes of simulated frame.
    let frame = RealDuration::from_millis(FRAME_MILLIS);
    let target = Instant::from_months(MONTHS);
    while app.sim().now() < target {
        let delta = app.clock_mut().simulated_for(frame);
        // **The request is clamped to the target.** A frame that would carry
        // the campaign past the target would leave the fast campaigns covering
        // more simulated time than the slow ones, and then they would be
        // different campaigns — which is not what this test is about. The
        // clamping is the *host's* business; the engine does exactly what it is
        // asked, and this asks for the same span either way.
        let delta = delta.min(target.duration_since(app.sim().effective_now()));
        app.advance(delta);
        // **A question on the table is put aside, not answered.** The world
        // stops while the dialog is up, and a loop that waits for the campaign
        // to move would wait for ever. Putting it aside is what a player who
        // is not watching does — the clock runs again, the window keeps
        // counting, and the expiry is part of the campaign either way.
        if app.decision().is_some() {
            app.on_key(esc());
        }
    }

    let chronicle = app
        .sim()
        .state()
        .chronicle
        .iter()
        .map(|entry| format!("{} {}", entry.date, entry.headline))
        .collect();
    (app.sim().state_hash(), chronicle)
}

#[test]
fn the_same_campaign_is_the_same_at_every_dilation() {
    let mut hashes = Vec::new();
    let mut chronicle = Vec::new();

    for scale in TimeScale::ALL {
        let (hash, entries) = run_at(scale);
        hashes.push((scale, hash));
        chronicle.push((scale, entries));
    }

    let first = hashes[0].1;
    for (scale, hash) in &hashes {
        assert_eq!(
            *hash,
            first,
            "the campaign at {}x hashed differently from the campaign at {}x",
            scale.multiplier(),
            hashes[0].0.multiplier()
        );
    }

    let first_chronicle = &chronicle[0].1;
    for (scale, entries) in &chronicle {
        assert_eq!(
            entries,
            first_chronicle,
            "the campaign at {}x produced a different history",
            scale.multiplier()
        );
    }
}

#[test]
fn the_dilations_really_do_use_different_sized_frames() {
    // The test above is only meaningful if the nine dilations really do chop
    // the time up differently. This checks that they do.
    let mut sizes = Vec::new();
    for scale in TimeScale::ALL {
        let clock = Clock::new(scale);
        let simulated = clock.simulated_for(RealDuration::from_millis(FRAME_MILLIS));
        sizes.push((scale, simulated));
    }

    // The first and the last differ by four orders of magnitude.
    assert!(
        sizes.last().expect("a dilation").1.millis() > sizes[0].1.millis() * 1_000,
        "the dilations must use very different frame sizes: {sizes:?}"
    );

    // And the simulated time is exactly the real time times the dilation.
    for (scale, simulated) in sizes {
        let expected = Duration::from_millis(FRAME_MILLIS.saturating_mul(scale.multiplier()));
        assert_eq!(simulated, expected);
    }
}
