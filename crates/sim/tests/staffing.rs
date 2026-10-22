//! Staffing a research project.
//!
//! **A project is a team, and the points are summed.** Putting two people on
//! one project has to work, and it has to be visible: the player is told how
//! many are on it, the research screen shows it, and the project finishes
//! faster for it.

use domain::ids::PersonId;
use domain::order::OrderKind;
use domain::person::Assignment;
use domain::time::Instant;
use sim::{Scenario, Sim};

fn campaign() -> Sim {
    Sim::new(&Scenario::first_year()).expect("the campaign starts")
}

/// The people who are free to be given work.
fn free_people(sim: &Sim) -> Vec<PersonId> {
    sim.state()
        .people
        .values()
        .filter(|person| !person.is_assigned())
        .map(|person| person.id)
        .collect()
}

#[test]
fn two_people_can_work_on_one_project() {
    // **The bug this guards.** A project with one person on it finishes in
    // half the time of a project with two, and the points are summed, so a
    // player who cannot put a second scientist on a project cannot do the
    // thing the whole research model is built for.
    let mut sim = campaign();
    let project = *sim
        .state()
        .projects
        .keys()
        .next()
        .expect("there is a project");
    let people = free_people(&sim);
    assert!(
        people.len() >= 2,
        "the scenario needs two people who are free"
    );

    for person in people.iter().take(2) {
        sim.issue(OrderKind::Assign {
            person: *person,
            to: Assignment::Project { project },
        })
        .expect("the assignment is accepted");
    }
    sim.advance_to(Instant::from_days(5))
        .expect("the campaign advances");

    let team = &sim.state().projects[&project].team;
    assert_eq!(
        team.len(),
        3,
        "the two new people and the one who was already there: {team:?}"
    );
    for person in people.iter().take(2) {
        assert!(
            team.contains(person),
            "{person:?} is on the team, and the team is {team:?}"
        );
        assert!(
            matches!(
                sim.state().people[person].assignment,
                Assignment::Project { project: other } if other == project
            ),
            "and the person is assigned to it"
        );
    }
}
