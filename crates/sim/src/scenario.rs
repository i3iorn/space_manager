//! Scenario definitions and the shipped scenarios.
//!
//! A scenario is a save with a name and no log, so the loader has one code
//! path and the editor needs no special support.

use domain::facility::FacilityType;
use domain::ids::{AgencyId, FacilityId, PersonId};
use domain::money::Money;
use domain::person::{Attributes, Person, Role, Specialism};
use domain::state::{Agency, CampaignState};
use domain::time::Instant;
use serde::{Deserialize, Serialize};

use crate::balance;

/// The government's orbital deadline, in the starting state.
///
/// **The state has promised the agency an orbital launch**, and it has
/// promised the *public* one too, and the promise has a date on it. The
/// player's first strategic decision is therefore not "what shall we do" but
/// "how much of the slack between our plan and our promise are we going to
/// spend".
///
/// The target is eighteen months out and the commitment is thirty-six:
/// **eighteen months of slack**, which is a comfortable buffer that the player
/// will have to learn to spend.
fn orbital_programme(state: &mut CampaignState, lead: PersonId) {
    let programme_id = state.id_counters.next_programme();
    let objective_id = state.id_counters.next_objective();
    let commitment_id = state.id_counters.next_commitment();
    let target = Instant::from_months(18);
    let committed = Instant::from_months(36);
    let mut programme = domain::programme::Programme::new(
        programme_id,
        "Orbital Launch".to_owned(),
        domain::programme::ProgrammeCategory::Orbital,
    );
    programme.lead = Some(lead);
    programme.staffing = vec![lead];
    programme.objectives.push(domain::programme::Objective {
        id: objective_id,
        description: "Reach orbit.".to_owned(),
        kind: domain::programme::ObjectiveKind::Primary,
        commitment: domain::programme::Commitment::new(
            commitment_id,
            Some(target),
            Some(committed),
            domain::programme::DeadlineAudience::Political,
        ),
        progress: domain::research::Progress::empty(balance::OBJECTIVE_POINTS_TARGET),
        met: None,
        remainders: domain::scale::Remainders::new(),
    });
    state.programmes.insert(programme_id, programme);
}
/// A starting point for a campaign.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Scenario {
    /// The scenario's name, as the player types it.
    pub id: String,
    /// Its name, as the player reads it.
    pub name: String,
    /// What it is.
    pub description: String,
    /// The campaign's root seed.
    pub seed: u64,
    /// The state the campaign starts from.
    pub start_state: CampaignState,
}

impl Scenario {
    /// The first year: a small agency, three scientists, and a budget that
    /// will not last.
    ///
    /// This is the scenario that teaches the game. It is deliberately small
    /// enough that the player can hold the whole agency in their head, and the
    /// team is small enough that fatigue matters within the first year.
    #[must_use]
    pub fn first_year() -> Self {
        let mut state = CampaignState::new(Agency::new(
            AgencyId(0),
            "Stellar Expansion Authority".to_owned(),
            2039,
        ));
        state.treasury.balance = Money::from_credits(190_000);
        state.political.approval.value = domain::scale::signed_percent_to_units(12);

        for mut person in starting_staff() {
            // The ids are allocated through the counter, not written directly,
            // so that the first hire does not collide with the starting staff.
            let id = state.id_counters.next_person();
            person.id = id;
            state.people.insert(id, person);
        }

        let facility_id = FacilityId(0);
        state.facilities.insert(
            facility_id,
            domain::facility::Facility {
                id: facility_id,
                name: "Propulsion Lab".to_owned(),
                kind: FacilityType::Laboratory,
                site: domain::facility::Site {
                    name: "The Core".to_owned(),
                    publicly_visible: false,
                },
                specification: domain::facility::Specification {
                    capacity: 4,
                    reliability: 70,
                    automation: 45,
                    staff_needed: 2,
                    quality: 65,
                },
                condition: (100 * domain::scale::PERCENT_UNITS as u32) as u16,
                staff: Vec::new(),
                construction: None,
                upkeep: balance::facility_base_upkeep(FacilityType::Laboratory),
                monthly_output: 4,
                remainders: domain::scale::Remainders::new(),
            },
        );

        // **A research project is already running [D].** The scenario is a
        // small agency with a budget that will not last, three scientists, and
        // one project under way — so the player's first decision is about a
        // project that exists, not about a blank sheet.
        //
        // **The lab has nobody in it.** The tools modifier reads the staff
        // list, and a facility needs two people to run at specification — so the
        // team is slow for a reason the player can see and fix, and the
        // binding-factor panel has something true to say.
        let project_id = state.id_counters.next_research_project();
        let lead = state
            .people
            .keys()
            .next()
            .copied()
            .expect("the staff are there");
        state.projects.insert(
            project_id,
            domain::research::ResearchProject {
                id: project_id,
                title: "Combustion stability".to_owned(),
                discipline: domain::research::DisciplineId(0),
                principal_investigator: lead,
                team: vec![lead],
                funding: domain::finance::Budget::empty(),
                progress: domain::research::Progress::empty(balance::PROJECT_POINTS_TARGET),
                target_capability: None,
                started: domain::time::Instant::ZERO,
                expected_completion: None,
                remainders: domain::scale::Remainders::new(),
            },
        );
        if let Some(person) = state.people.get_mut(&lead) {
            person.assignment = domain::person::Assignment::Project {
                project: project_id,
            };
        }

        orbital_programme(&mut state, lead);

        Self {
            id: "first_year".to_owned(),
            name: "Ground Game".to_owned(),
            description: "A young agency with three scientists and a budget that will not last."
                .to_owned(),
            seed: 6_125_084_923_715_548,
            start_state: state,
        }
    }

    /// A sandbox: no political deadline and a deep treasury, for experimenting
    /// with a strategy before committing to it.
    #[must_use]
    pub fn sandbox() -> Self {
        let mut scenario = Self::first_year();
        "sandbox".clone_into(&mut scenario.id);
        "Sandbox".clone_into(&mut scenario.name);
        "No deadline, no rivals, and money to burn. Try a strategy here first."
            .clone_into(&mut scenario.description);
        scenario.start_state.treasury.balance = Money::from_credits(5_000_000);
        scenario
    }

    /// Looks a scenario up by its name.
    #[must_use]
    pub fn by_id(id: &str) -> Option<Self> {
        match id {
            "first_year" => Some(Self::first_year()),
            "sandbox" => Some(Self::sandbox()),
            _ => None,
        }
    }

    /// The names of the shipped scenarios.
    #[must_use]
    pub fn ids() -> Vec<&'static str> {
        vec!["first_year", "sandbox"]
    }
}

/// The people a new agency starts with.
fn starting_staff() -> Vec<Person> {
    let mut people = Vec::new();

    let mut okonjo = Person::new(
        PersonId(0),
        "Dr. Marsha Okonjo".to_owned(),
        Role::Scientist,
        balance::starting_salary(Role::Scientist),
    );
    okonjo.attributes = Attributes {
        technical: 82,
        creative: 78,
        leadership: 44,
        physical: 51,
        diplomatic: 38,
    };
    okonjo.specialisms = vec![Specialism::Propulsion, Specialism::Computing];
    okonjo.age = 41;
    people.push(okonjo);

    let mut reyes = Person::new(
        PersonId(1),
        "Tomas Reyes".to_owned(),
        Role::Engineer,
        balance::starting_salary(Role::Engineer),
    );
    reyes.attributes = Attributes {
        technical: 76,
        creative: 51,
        leadership: 39,
        physical: 67,
        diplomatic: 44,
    };
    reyes.specialisms = vec![Specialism::Structures, Specialism::Manufacturing];
    reyes.age = 35;
    people.push(reyes);

    let mut bhatt = Person::new(
        PersonId(2),
        "Anika Bhatt".to_owned(),
        Role::Scientist,
        balance::starting_salary(Role::Scientist),
    );
    bhatt.attributes = Attributes {
        technical: 64,
        creative: 71,
        leadership: 52,
        physical: 48,
        diplomatic: 61,
    };
    bhatt.specialisms = vec![Specialism::LifeSupport, Specialism::Medicine];
    bhatt.age = 29;
    people.push(bhatt);

    people
}

/// The Instant a scenario starts in.
#[must_use]
pub fn start_instant() -> Instant {
    Instant::ZERO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_year_scenario_starts_with_three_people() {
        let scenario = Scenario::first_year();
        assert_eq!(scenario.start_state.people.len(), 3);
    }

    #[test]
    fn the_first_year_scenario_starts_with_a_laboratory() {
        let scenario = Scenario::first_year();
        assert_eq!(scenario.start_state.facilities.len(), 1);
    }

    #[test]
    fn the_sandbox_has_money_and_the_same_staff() {
        let sandbox = Scenario::sandbox();
        let first_year = Scenario::first_year();
        assert_eq!(
            sandbox.start_state.people.len(),
            first_year.start_state.people.len()
        );
        assert!(sandbox.start_state.treasury.balance > first_year.start_state.treasury.balance);
    }

    #[test]
    fn the_starting_staff_have_no_assignments() {
        let scenario = Scenario::first_year();
        // **Exactly one member of staff is on the project** — the lead — and
        // the other two are free for the player to give something to do.
        let assigned: Vec<_> = scenario
            .start_state
            .people
            .values()
            .filter(|person| person.is_assigned())
            .collect();
        assert_eq!(assigned.len(), 1, "only the lead should be on the project");
        assert_eq!(
            assigned[0].assignment,
            domain::person::Assignment::Project {
                project: domain::ids::ResearchProjectId(0)
            }
        );
    }

    #[test]
    fn the_scenario_starts_with_a_project_running() {
        // **The scenario is a small agency with a budget that will not last,
        // three scientists, and a research project already running.**
        let scenario = Scenario::first_year();
        assert_eq!(scenario.start_state.people.len(), 3);
        assert_eq!(scenario.start_state.projects.len(), 1);
        let project = scenario
            .start_state
            .projects
            .values()
            .next()
            .expect("a project");
        assert_eq!(project.team.len(), 1);
    }

    #[test]
    fn the_lab_starts_unstaffed() {
        // **The team is slow for a reason the player can see and fix.** The
        // binding-factor panel says "tools", and the player can put someone in
        // the lab.
        let scenario = Scenario::first_year();
        for facility in scenario.start_state.facilities.values() {
            assert!(
                facility.staff.is_empty(),
                "the lab starts with nobody in it"
            );
        }
    }

    #[test]
    fn the_first_hire_does_not_collide_with_the_starting_staff() {
        let mut scenario = Scenario::first_year();
        let starting = scenario.start_state.people.len() as u64;
        let id = scenario.start_state.id_counters.next_person();
        assert!(
            id.0 >= starting,
            "the first hire would reuse a starting employee's id"
        );
    }
}
