//! The fork, through the interface.
//!
//! **Forking is the answer to "what if I had not cancelled that programme?"**
//! The player looks back at a decision, forks the campaign at that instant, and
//! watches the two branches play out side by side.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use domain::order::OrderKind;
use domain::person::Role;
use domain::time::Duration;
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn the_app_can_fork_and_the_branch_advances_with_the_clock() {
    let scenario = Scenario::first_year();
    let mut app = App::started(sim::Sim::new(&scenario).expect("the scenario starts"));

    // The player gives an order — that instant becomes a fork point.
    app.sim_mut()
        .issue(OrderKind::Hire {
            role: Role::Scientist,
            salary: domain::money::Money::from_credits(1_200),
        })
        .expect("the order is accepted");
    app.refresh_fork_points();
    assert_eq!(app.fork_points().len(), 1);

    // And the branch is the same campaign up to the fork point.
    app.fork();
    let branch = app.branch().expect("the campaign forked");
    assert_eq!(branch.state_hash(), app.sim().state_hash());

    // The branch is a second campaign, and it advances with the clock.
    let before = branch.now();
    app.advance(Duration::from_hours(2));
    let branch = app.branch().expect("the campaign forked");
    assert!(
        branch.now() > before,
        "the branch should advance with the clock"
    );
}

#[test]
fn forking_with_nothing_to_fork_says_so() {
    let scenario = Scenario::first_year();
    let mut app = App::started(sim::Sim::new(&scenario).expect("the scenario starts"));
    app.fork();
    assert!(app.branch().is_none());
    assert!(app.notice().is_some(), "the player should be told");
}

#[test]
fn the_fork_happens_at_the_decision_the_player_chose() {
    // **The player picks the decision to fork at**, so the arrows have to move
    // the selection on the branches screen and the fork has to happen there.
    let scenario = Scenario::first_year();
    let mut app = App::started(sim::Sim::new(&scenario).expect("the campaign starts"));
    let person = *app
        .sim()
        .state()
        .people
        .keys()
        .next()
        .expect("there is a person");
    app.sim_mut()
        .issue(OrderKind::Hire {
            role: Role::Scientist,
            salary: domain::money::Money::from_credits(1_200),
        })
        .expect("the order is accepted");
    app.refresh_fork_points();
    app.advance(Duration::from_days(30));
    // **The month report is a modal**, and it takes the first key.
    app.on_key(press(KeyCode::Char('x')));
    app.sim_mut()
        .issue(OrderKind::Fire { person })
        .expect("the order is accepted");
    app.refresh_fork_points();
    assert_eq!(app.fork_points().len(), 2);

    // The player goes to the branches screen and picks the second decision.
    app.on_key(press(KeyCode::Char('7')));
    app.on_key(press(KeyCode::Down));
    assert_eq!(app.selected(), 1, "the second decision is selected");

    app.fork();
    let branch = app.branch().expect("the campaign forked");
    assert_eq!(
        branch.now(),
        app.fork_points()[1],
        "the branch is the campaign up to the decision the player chose"
    );
}
