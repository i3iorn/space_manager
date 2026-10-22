//! Research, testing, and the two halves of the central tension.
//!
//! **The tension is a staffing tension.** Research and testing want the same
//! people, and the numbers here are what make that choice cost something
//! rather than being free.

use domain::order::OrderKind;
use domain::person::Assignment;
use sim::{Scenario, Sim};

/// Advances the campaign by a number of months.
fn advance(sim: &mut Sim, months: u64) {
    let target = sim
        .effective_now()
        .saturating_add(domain::time::Duration::from_months(months));
    sim.advance_to(target).expect("the campaign advances");
}

/// A campaign with a research project running, long enough for a capability
/// to have been developed.
///
/// **A project needs a team of four, and it needs them fresh.** One person
/// alone tires out inside a year, and a tired person's performance falls, so a
/// solo project never finishes - which is itself a lesson the game teaches.
/// The shift is the tool for it: a team on light duties recovers while it
/// works, and the project still gets done.
fn with_a_developed_capability() -> (Sim, domain::research::CapabilityId) {
    let mut sim = Sim::new(&Scenario::first_year()).expect("the campaign starts");
    let people: Vec<_> = sim.state().people.keys().copied().take(4).collect();
    let principal = *people.first().expect("there is a person");
    let _ = sim.issue(OrderKind::StartProject {
        discipline: domain::research::DisciplineId(0),
        principal_investigator: principal,
        title: "Combustion stability".to_owned(),
    });
    // **The order lands in a fortnight**, and only then is there a project to
    // put anybody else on.
    advance(&mut sim, 1);
    let project = *sim
        .state()
        .projects
        .keys()
        .next()
        .expect("the project has started");
    for person in people.iter().skip(1) {
        let _ = sim.issue(OrderKind::Assign {
            person: *person,
            to: Assignment::Project { project },
        });
    }
    // **And the whole team goes on light duties**, so that they are still
    // worth having in two years' time.
    for person in &people {
        let _ = sim.issue(OrderKind::SetShift {
            person: *person,
            shift: Some(50 * domain::scale::PERCENT_UNITS as u16),
        });
    }
    advance(&mut sim, 72);
    let capability = *sim
        .state()
        .unlocked
        .iter()
        .next()
        .expect("a capability has been developed by now");
    (sim, capability)
}

/// A campaign with a test campaign running and staffed.
fn with_a_running_campaign() -> (Sim, domain::research::CapabilityId) {
    let (mut sim, capability) = with_a_developed_capability();
    let _ = sim.issue(OrderKind::StartTestCampaign { capability });
    advance(&mut sim, 1);
    let campaign = *sim
        .state()
        .test_campaigns
        .keys()
        .next()
        .expect("the campaign has started");
    let people: Vec<_> = sim.state().people.keys().copied().take(4).collect();
    for person in people.iter().skip(1) {
        let _ = sim.issue(OrderKind::Assign {
            person: *person,
            to: Assignment::TestCampaign { campaign },
        });
    }
    (sim, capability)
}

#[test]
fn a_test_campaign_is_not_finished_in_its_first_month() {
    // **The regression this file exists for.** The campaign's target was a
    // hundred points and its progress was scaled, so the first month's points
    // passed the target on their own and every test campaign finished
    // immediately. Testing has to cost time as well as money, or the player
    // will always test and the research half of the game is pointless.
    let (mut sim, _capability) = with_a_running_campaign();
    advance(&mut sim, 2);

    let campaign = sim
        .state()
        .test_campaigns
        .values()
        .next()
        .expect("the campaign is running");
    assert!(
        !campaign.progress.is_complete(),
        "a test campaign finished in its first months: {campaign:?}"
    );
    assert!(
        campaign.progress.points < campaign.progress.target,
        "the campaign is at {} of {} points",
        campaign.progress.points,
        campaign.progress.target
    );
}

#[test]
fn a_test_campaign_with_a_team_finishes_in_five_months() {
    // **The arithmetic the manual quotes.** Four people, five points a month
    // each, and a hundred points to do: five months.
    let target = sim::balance::TEST_CAMPAIGN_POINTS_TARGET;
    let per_head = sim::balance::TEST_CAMPAIGN_POINTS_PER_HEAD;
    let scaled = |points: u32| points * domain::scale::PERCENT_UNITS as u32;
    assert_eq!(scaled(target), 25_600);
    assert_eq!(scaled(per_head) * 4, 5_120);
    // **Five months of four people reaches the target; four does not.**
    assert!(5_120 * 5 >= scaled(target), "five months is enough");
    assert!(5_120 * 4 < scaled(target), "four months is not");
}

#[test]
fn the_risk_reduction_outlives_the_campaign() {
    // **Testing is remembered.** The people who found it can leave; the risk
    // they took out stays taken out. It is the only investment in the game
    // whose value does not decay.
    let (mut sim, capability) = with_a_running_campaign();
    advance(&mut sim, 24);
    let reduction = sim
        .state()
        .tested
        .get(&capability)
        .copied()
        .expect("the capability has been tested");
    assert!(reduction > 0, "a completed campaign takes risk out");
    // **And the campaign itself is gone** - the testing is remembered, not the
    // people who did it.
    assert!(
        sim.state()
            .test_campaigns
            .values()
            .all(|campaign| campaign.target_capability != capability),
        "the finished campaign was not removed"
    );
}
