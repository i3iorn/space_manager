//! The fork, through the interface.
//!
//! **Forking is the answer to "what if I had not cancelled that programme?"**
//! The player looks back at a decision, forks the campaign at that instant, and
//! watches the two branches play out side by side.

use domain::order::OrderKind;
use domain::person::Role;
use domain::time::Duration;
use sim::Scenario;
use ui::App;

#[test]
fn the_app_can_fork_and_the_branch_advances_with_the_clock() {
    let scenario = Scenario::first_year();
    let mut app = App::new(sim::Sim::new(&scenario).expect("the scenario starts"));

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
    let mut app = App::new(sim::Sim::new(&scenario).expect("the scenario starts"));
    app.fork();
    assert!(app.branch().is_none());
    assert!(app.notice().is_some(), "the player should be told");
}
