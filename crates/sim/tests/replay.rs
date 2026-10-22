//! Replay and branching.
//!
//! **The MVP's most expensive feature is the one that lets a player ask
//! "what if I had not?"** and it is expensive because it depends on the
//! order log being a complete record of the campaign: a campaign rebuilt from
//! the log alone has to be the same campaign, or every comparison the player
//! makes is noise.
//!
//! These tests are about that property, and about the one thing the player
//! actually wants from it: a fork at a decision, a different decision in the
//! branch, and two campaigns that then differ for a reason the player can
//! name.

use domain::order::OrderKind;
use domain::person::{Assignment, ProgrammeRole};
use domain::time::Instant;
use sim::Scenario;
use sim::Sim;

/// A campaign with a few decisions in it.
fn campaign() -> Sim {
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    for month in 0..6_u32 {
        for order in sim::harness::plan_for_debug(sim.state(), sim::harness::Policy::Cautious) {
            let _ = sim.issue(order);
        }
        sim.advance_to(Instant::from_months(u64::from(month) + 1))
            .expect("the campaign advances");
    }
    sim
}

#[test]
fn a_replayed_campaign_is_the_same_campaign() {
    // **The log is the campaign, and everything else is a cache of it.** This
    // is the property everything else in the feature rests on.
    let sim = campaign();
    let replayed = sim.replay().expect("the campaign replays");
    assert_eq!(
        replayed.state_hash(),
        sim.state_hash(),
        "a campaign rebuilt from its log is the same campaign"
    );
    assert_eq!(replayed.now(), sim.now());
    assert_eq!(
        replayed.state().chronicle.len(),
        sim.state().chronicle.len(),
        "and it has the same history"
    );
}

#[test]
fn a_replayed_campaign_is_the_same_campaign_again() {
    // **Replay is idempotent**: replaying a replay gives the same campaign.
    let sim = campaign();
    let once = sim.replay().expect("the campaign replays");
    let twice = once.replay().expect("the replay replays");
    assert_eq!(twice.state_hash(), sim.state_hash());
}

#[test]
fn a_replay_reproduces_the_chronicle() {
    // **The chronicle is the durable record**, and a replay that produced a
    // different chronicle would be a different campaign.
    let sim = campaign();
    let replayed = sim.replay().expect("the campaign replays");
    let original: Vec<&str> = sim
        .state()
        .chronicle
        .iter()
        .map(|entry| entry.headline.as_str())
        .collect();
    let copy: Vec<&str> = replayed
        .state()
        .chronicle
        .iter()
        .map(|entry| entry.headline.as_str())
        .collect();
    assert_eq!(original, copy);
}

#[test]
fn a_branch_is_the_same_campaign_up_to_the_fork_point() {
    // **The comparison is only meaningful if the two are the same campaign
    // first.** A branch that differed before the fork would make every later
    // difference noise.
    let sim = campaign();
    let point = Instant::from_months(3);
    let branch = sim.fork(point).expect("the campaign forks");
    assert_eq!(branch.now(), point);
    assert!(
        branch.log().len() < sim.log().len(),
        "the branch's log stops at the fork point"
    );
}

#[test]
fn a_different_decision_makes_a_different_campaign() {
    // **This is the point of the whole feature.** The player forks at a
    // decision, makes a different one in the branch, and watches the two
    // campaigns diverge for a reason they can name.
    let mut sim = campaign();
    let point = Instant::from_months(3);
    let mut branch = sim.fork(point).expect("the campaign forks");

    // **A different decision.** The original puts somebody on the programme;
    // the branch hires somebody instead.
    let programme = sim
        .state()
        .programmes
        .keys()
        .next()
        .copied()
        .expect("a programme");
    let someone = sim.state().people.keys().next().copied().expect("a person");

    let _ = sim.issue(OrderKind::Assign {
        person: someone,
        to: Assignment::Programme {
            programme,
            role: ProgrammeRole::Member,
        },
    });
    let _ = branch.issue(OrderKind::Hire {
        role: domain::person::Role::Scientist,
        salary: domain::money::Money::from_credits(1_200),
    });

    sim.advance_to(Instant::from_months(9)).expect("advances");
    branch
        .advance_to(Instant::from_months(9))
        .expect("advances");

    assert_ne!(
        branch.state_hash(),
        sim.state_hash(),
        "two campaigns that made different decisions are different campaigns"
    );

    // **And the difference is nameable.** The branch has one more person and
    // the original has one more assignment.
    let differences = sim::branch::divergence(sim.state(), branch.state());
    assert!(
        differences
            .iter()
            .any(|difference| difference.subject == "Staff"),
        "the branches differ in staff: {differences:?}"
    );
}

#[test]
fn a_fork_keeps_the_original_log() {
    // **The original's decisions are the original's**, and the branch has its
    // own. A fork that rewrote the original's history would be a fork the
    // player could not trust.
    let sim = campaign();
    let before = sim.log().len();
    let branch = sim
        .fork(Instant::from_months(3))
        .expect("the campaign forks");
    assert_eq!(sim.log().len(), before, "the original's log is untouched");
    assert!(branch.log().len() <= before);
}

#[test]
fn a_branch_advances_with_the_clock() {
    // **A comparison between two campaigns at different instants says
    // nothing**, so the branch advances with the original.
    let mut sim = campaign();
    let mut branch = sim
        .fork(Instant::from_months(3))
        .expect("the campaign forks");
    sim.advance_to(Instant::from_months(6)).expect("advances");
    branch
        .advance_to(Instant::from_months(6))
        .expect("advances");
    assert_eq!(branch.now(), sim.now());
}
