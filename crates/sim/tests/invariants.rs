//! The cross-system invariants.
//!
//! **A reference that points at nothing is a bug no single system can see.**
//! The campaign is one aggregate, and the things that point at each other -
//! a person's assignment and the team's list, a commitment and the scheduler,
//! a mission and its next phase - are the places where a mistake is invisible
//! from inside the system that made it.
//!
//! These are the checks that span the systems, and they are the ones the
//! Slice 3 work added after the work itself found two of the same bug.

use domain::ids::{CommitmentId, MissionId, PersonId};
use domain::person::Assignment;
use domain::state::CampaignState;
use domain::time::Instant;
use sim::Scenario;
use sim::Sim;
use sim::scheduler::ScheduledKind;

/// Every person a team list names is a person the agency employs.
fn no_orphaned_people(state: &CampaignState) {
    let check = |team: &[PersonId], what: &str| {
        for member in team {
            assert!(
                state.people.contains_key(member),
                "{what} names a person the agency does not employ"
            );
        }
    };
    for project in state.projects.values() {
        check(&project.team, "a project team");
    }
    for campaign in state.test_campaigns.values() {
        check(&campaign.team, "a test campaign team");
    }
    for facility in state.facilities.values() {
        check(&facility.staff, "a facility's staff");
    }
    for programme in state.programmes.values() {
        check(&programme.staffing, "a programme's staffing");
    }
}

/// Every assignment points at something that exists.
fn no_orphaned_assignments(state: &CampaignState) {
    for person in state.people.values() {
        match &person.assignment {
            Assignment::Unassigned | Assignment::Division(_) => {}
            Assignment::Programme { programme, .. } => assert!(
                state.programmes.contains_key(programme),
                "{} is assigned to a programme that does not exist",
                person.name
            ),
            Assignment::Project { project } => assert!(
                state.projects.contains_key(project),
                "{} is assigned to a project that does not exist",
                person.name
            ),
            Assignment::Facility { facility } => assert!(
                state.facilities.contains_key(facility),
                "{} is assigned to a facility that does not exist",
                person.name
            ),
            Assignment::TestCampaign { campaign } => assert!(
                state.test_campaigns.contains_key(campaign),
                "{} is assigned to a test campaign that does not exist",
                person.name
            ),
        }
    }
}

/// **Being assigned to something and being on its team are the same thing.**
///
/// This is the invariant the Slice 1 work found two bugs behind, and it is
/// the one the Slice 3 work found a third of.
fn teams_and_assignments_agree(state: &CampaignState) {
    for person in state.people.values() {
        let id = person.id;
        match &person.assignment {
            Assignment::Project { project } => {
                let on_the_team = state
                    .projects
                    .get(project)
                    .is_some_and(|project| project.team.contains(&id));
                assert!(
                    on_the_team,
                    "{} is assigned to a project but is not on its team",
                    person.name
                );
            }
            Assignment::TestCampaign { campaign } => {
                let on_the_team = state
                    .test_campaigns
                    .get(campaign)
                    .is_some_and(|campaign| campaign.team.contains(&id));
                assert!(
                    on_the_team,
                    "{} is assigned to a test campaign but is not on its team",
                    person.name
                );
            }
            Assignment::Facility { facility } => {
                let on_the_staff = state
                    .facilities
                    .get(facility)
                    .is_some_and(|facility| facility.staff.contains(&id));
                assert!(
                    on_the_staff,
                    "{} is assigned to a facility but is not on its staff",
                    person.name
                );
            }
            Assignment::Programme { programme, role } => {
                let on_the_team = state
                    .programmes
                    .get(programme)
                    .is_some_and(|programme| programme.staffing.contains(&id));
                assert!(
                    on_the_team,
                    "{} is assigned to a programme but is not on its staffing",
                    person.name
                );
                if *role == domain::person::ProgrammeRole::Lead {
                    assert_eq!(
                        state.programmes.get(programme).and_then(|p| p.lead),
                        Some(id),
                        "{} is the programme's lead and the programme does not know it",
                        person.name
                    );
                }
            }
            Assignment::Unassigned | Assignment::Division(_) => {}
        }
    }
}

/// Every commitment with a date is on the scheduler.
///
/// **A commitment that is not on the scheduler never falls due**, and a
/// deadline the game does not know about is not a deadline.
fn every_commitment_is_scheduled(state: &CampaignState, sim: &Sim) {
    let scheduled: Vec<CommitmentId> = sim
        .scheduler()
        .items()
        .iter()
        .filter_map(|item| match &item.kind {
            ScheduledKind::CommitmentDue { commitment } => Some(*commitment),
            _ => None,
        })
        .collect();

    for programme in state.programmes.values() {
        for objective in &programme.objectives {
            let Some(due) = objective.commitment.committed else {
                continue;
            };
            if objective.met.is_some() {
                continue;
            }
            assert!(
                scheduled.contains(&objective.commitment.id),
                "a commitment due at {due} is not on the scheduler"
            );
        }
    }
}

/// Every mission that is running has its next phase scheduled.
fn every_mission_has_a_phase_scheduled(state: &CampaignState, sim: &Sim) {
    let scheduled: Vec<MissionId> = sim
        .scheduler()
        .items()
        .iter()
        .filter_map(|item| match &item.kind {
            ScheduledKind::MissionPhase { mission, .. } => Some(*mission),
            _ => None,
        })
        .collect();

    for mission in state.missions.values() {
        if !mission.phase.is_running() {
            continue;
        }
        assert!(
            scheduled.contains(&mission.id),
            "a mission in the {} phase has no phase scheduled",
            mission.phase.label()
        );
    }
}

/// Every project has a principal investigator who exists.
///
/// **The lead may have been moved off the project** — that is the player's
/// decision, and the rate says so — but the project may not name somebody who
/// has left the agency.
fn every_project_has_its_lead(state: &CampaignState) {
    for project in state.projects.values() {
        assert!(
            state.people.contains_key(&project.principal_investigator),
            "a project has a principal investigator who has left"
        );
    }
}

/// Every test campaign has a capability the agency developed.
fn every_test_campaign_tests_something_real(state: &CampaignState) {
    for campaign in state.test_campaigns.values() {
        assert!(
            state.unlocked.contains(&campaign.target_capability),
            "a test campaign is running on a capability the agency has not developed"
        );
    }
}

/// The chronicle is append-only, and its entries are numbered in order.
fn the_chronicle_is_in_order(state: &CampaignState) {
    let mut previous: Option<u64> = None;
    for entry in &state.chronicle {
        if let Some(previous) = previous {
            assert!(
                entry.id.0 > previous,
                "the chronicle's entries are numbered in order"
            );
        }
        previous = Some(entry.id.0);
    }
}

/// Everything at once.
#[track_caller]
fn check_the_campaign(state: &CampaignState, sim: &Sim) {
    no_orphaned_people(state);
    no_orphaned_assignments(state);
    teams_and_assignments_agree(state);
    every_commitment_is_scheduled(state, sim);
    every_mission_has_a_phase_scheduled(state, sim);
    every_project_has_its_lead(state);
    every_test_campaign_tests_something_real(state);
    the_chronicle_is_in_order(state);
}

#[test]
fn a_fresh_campaign_satisfies_every_invariant() {
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    check_the_campaign(sim.state(), &sim);
    sim.advance_to(Instant::from_months(1))
        .expect("the campaign advances");
    check_the_campaign(sim.state(), &sim);
}

#[test]
fn a_campaign_with_work_in_flight_satisfies_every_invariant() {
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    // **Give the agency something to do**, so that there are teams to be
    // consistent about.
    for order in sim::harness::plan_for_debug(sim.state(), sim::harness::Policy::Fast) {
        let _ = sim.issue(order);
    }
    sim.advance_to(Instant::from_months(6))
        .expect("the campaign advances");
    check_the_campaign(sim.state(), &sim);
    sim.advance_to(Instant::from_months(18))
        .expect("the campaign advances");
    check_the_campaign(sim.state(), &sim);
}

#[test]
fn the_invariants_hold_through_a_whole_campaign() {
    let mut sim = Sim::new(&Scenario::first_orbit()).expect("the campaign starts");
    for month in 0..24_u32 {
        for order in sim::harness::plan_for_debug(sim.state(), sim::harness::Policy::Cautious) {
            let _ = sim.issue(order);
        }
        sim.advance_to(Instant::from_months(u64::from(month) + 1))
            .expect("the campaign advances");
        check_the_campaign(sim.state(), &sim);
    }
}
