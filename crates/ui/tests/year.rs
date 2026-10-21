//! A year of play.
//!
//! **The campaign runs whether or not the player is touching anything**, and a
//! year of simulated time is a year of the agency living: the project advances,
//! the treasury drains, and the chronicle fills up.

use std::time::Duration as RealDuration;

use domain::ids::PersonId;
use domain::order::OrderKind;
use domain::person::Assignment;
use domain::time::TimeScale;
use sim::Scenario;
use ui::App;

/// Plays the campaign at a dilation, for a span of simulated time.
///
/// The host's frame is a fixed amount of *real* time, and the clock turns it
/// into simulated time — which is the whole of the dilation.
fn play(app: &mut App, seconds: u64) {
    let frame = RealDuration::from_millis(16);
    while app.sim().now().millis() < seconds * 1_000 {
        let delta = app.clock_mut().simulated_for(frame);
        app.advance(delta);
    }
}

#[test]
fn a_year_of_play_finishes_a_project() {
    let scenario = Scenario::first_year();
    let mut app = App::new(sim::Sim::new(&scenario).expect("the campaign starts"));
    app.clock_mut().set_scale(TimeScale::SixHundredThousand);

    // The player puts the team on the project. The order takes a lead time, and
    // the campaign does not wait for it.
    app.sim_mut()
        .issue(OrderKind::Assign {
            person: PersonId(1),
            to: Assignment::Project {
                project: domain::ids::ResearchProjectId(0),
            },
        })
        .expect("the order is accepted");
    app.sim_mut()
        .issue(OrderKind::Assign {
            person: PersonId(2),
            to: Assignment::Project {
                project: domain::ids::ResearchProjectId(0),
            },
        })
        .expect("the order is accepted");

    // A year and a half of simulated time.
    play(&mut app, 47_000_000);

    let chronicle: Vec<String> = app
        .sim()
        .state()
        .chronicle
        .iter()
        .map(|entry| entry.headline.clone())
        .collect();
    assert!(
        chronicle
            .iter()
            .any(|headline| headline.contains("Research completed")),
        "the project should have finished: {chronicle:?}"
    );
}

#[test]
fn the_clock_moves_in_minutes_and_not_just_on_the_hour() {
    // **The grid is an implementation detail.** The player's clock moves
    // smoothly, and the minute is not always zero.
    let scenario = Scenario::first_year();
    let mut app = App::new(sim::Sim::new(&scenario).expect("the campaign starts"));
    app.clock_mut().set_scale(TimeScale::SixHundredThousand);
    play(&mut app, 3_600);
    let date = app
        .sim()
        .calendar()
        .date(app.sim().effective_now())
        .to_string();
    assert!(
        !date.ends_with("00:00"),
        "the clock should show the minutes: {date}"
    );
}
