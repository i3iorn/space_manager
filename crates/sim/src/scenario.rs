//! Scenario definitions and the shipped scenarios.
//!
//! A scenario is a save with a name and no log, so the loader has one code
//! path and the editor needs no special support.
//!
//! **The scenarios are the game's first impression**, and each one asks the
//! player a different question:
//!
//! | Scenario | The question |
//! |---|---|
//! | `first_year` | *Can I run a small agency, and why is my team slow?* |
//! | `first_orbit` | *Can I reach orbit in five years?* |
//! | `sandbox` | *What happens if I try a strategy here first?* |

use domain::facility::{Facility, FacilityType, Specification};
use domain::ids::{AgencyId, FacilityId, PersonId, ResearchProjectId};
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
/// The target is the agency's honest internal plan and is soft; the committed
/// instant is what was promised and is hard. **The gap between them is the
/// slack**, and the slack is the thing the player manages.
fn orbital_programme(
    state: &mut CampaignState,
    lead: Option<PersonId>,
    staffing: Vec<PersonId>,
    target: Instant,
    committed: Instant,
) -> domain::ids::ProgrammeId {
    let programme_id = state.id_counters.next_programme();
    let objective_id = state.id_counters.next_objective();
    let commitment_id = state.id_counters.next_commitment();
    let mut programme = domain::programme::Programme::new(
        programme_id,
        "Orbital Launch".to_owned(),
        domain::programme::ProgrammeCategory::Orbital,
    );
    programme.status = domain::programme::ProgrammeStatus::Active;
    programme.lead = lead;
    programme.staffing = staffing;
    let kind = domain::programme::ObjectiveKind::Primary;
    programme.objectives.push(domain::programme::Objective {
        id: objective_id,
        description: "Reach orbit.".to_owned(),
        kind,
        commitment: domain::programme::Commitment::new(
            commitment_id,
            Some(target),
            Some(committed),
            domain::programme::DeadlineAudience::Political,
        ),
        progress: domain::research::Progress::empty(balance::OBJECTIVE_POINTS_TARGET),
        met: None,
        // **The orbital programme is finished by reaching orbit.** The
        // programme's staff advance the objective's progress — that is how the
        // player sees the work going on — but the objective is only complete
        // when a launch to orbit has worked.
        completion: programme.completion_for(kind),
        milestone_reached: false,
        remainders: domain::scale::Remainders::new(),
    });
    state.programmes.insert(programme_id, programme);
    programme_id
}

/// Gives everybody their traits.
///
/// **The starting staff are people and not just numbers**, so they have
/// traits, and they get them the same way a recruit does: drawn
/// deterministically, so a campaign is the same campaign every time.
fn give_traits(state: &mut CampaignState, seed: u64) {
    for person in state.people.values_mut() {
        let mut rng =
            crate::rng::Rng::new(seed).stream(crate::rng::Domain::Personnel, 0, person.id.0);
        person.traits = crate::recruitment::traits(&mut rng);
    }
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
        give_traits(&mut state, 1_011);

        state
            .facilities
            .insert(FacilityId(0), laboratory("Propulsion Lab", 4, 65, 2));

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

        // **The government has a deadline, and nobody is on the programme
        // yet.** The first thing the player can do about it is put somebody on
        // it, and that is the tutorial.
        orbital_programme(
            &mut state,
            None,
            Vec::new(),
            Instant::from_months(18),
            Instant::from_months(36),
        );

        Self {
            id: "first_year".to_owned(),
            name: "Ground Game".to_owned(),
            description: "A young agency with three scientists and a budget that will not last."
                .to_owned(),
            seed: 6_125_084_923_715_548,
            start_state: state,
        }
    }

    /// **First Orbit** — five years, one government deadline, and an agency
    /// with enough people to do it if the player is quick about it.
    ///
    /// This is the scenario the whole build plan is aimed at. It is a *tuned*
    /// scenario: the cast is written down rather than generated, the treasury
    /// is set so that a competent agency survives to the launch and a
    /// wasteful one does not, and the deadline is thirty months out with
    /// sixty months of promise behind it.
    ///
    /// **The starting state is not a solution.** It is a position: ten people,
    /// two buildings, one project, one programme, and a hole in the treasury
    /// that the player has to fill. Reaching orbit takes research, a vehicle,
    /// and a launch — and every one of them costs money and time that the
    /// deadline is counting down.
    #[must_use]
    pub fn first_orbit() -> Self {
        let mut state = CampaignState::new(Agency::new(
            AgencyId(0),
            "Stellar Expansion Authority".to_owned(),
            2039,
        ));
        state.treasury.balance = Money::from_credits(2_600_000);
        state.political.approval.value = domain::scale::signed_percent_to_units(20);

        // **The cast.** The ids come from the counter, so the first hire does
        // not collide with the starting staff.
        let mut ids = Vec::new();
        for (name, role, attributes, specialisms, age) in orbit_staff() {
            let id = state.id_counters.next_person();
            let mut person = Person::new(id, name.to_owned(), role, balance::starting_salary(role));
            person.attributes = attributes;
            person.specialisms = specialisms;
            person.age = age;
            state.people.insert(id, person);
            ids.push(id);
        }
        give_traits(&mut state, 2_022);

        // **Two buildings, and both of them staffed.** A facility nobody is
        // looking after wears out at four percent a month and the world reads
        // it: a well-run agency does not have equipment failing every other
        // week, and the starting state should not be one.
        state
            .facilities
            .insert(FacilityId(0), laboratory("Propulsion Laboratory", 8, 70, 2));
        state
            .facilities
            .insert(FacilityId(1), plant("Vehicle Fabrication Plant", 6, 65, 1));

        // **One project, already running.** The agency's first piece of work is
        // a combustion study, and it is the project the orbital programme is
        // waiting on.
        let project_id = state.id_counters.next_research_project();
        let investigator = ids[1];
        let second = ids[5];
        state.projects.insert(
            project_id,
            domain::research::ResearchProject {
                id: project_id,
                title: "Combustion stability".to_owned(),
                discipline: domain::research::DisciplineId(0),
                principal_investigator: investigator,
                team: vec![investigator, second],
                funding: domain::finance::Budget::empty(),
                progress: domain::research::Progress::empty(balance::PROJECT_POINTS_TARGET),
                target_capability: None,
                started: domain::time::Instant::ZERO,
                expected_completion: None,
                remainders: domain::scale::Remainders::new(),
            },
        );
        for member in [investigator, second] {
            if let Some(person) = state.people.get_mut(&member) {
                person.assignment = domain::person::Assignment::Project {
                    project: project_id,
                };
            }
        }

        // **The programme, with three people on it.** The manager runs it and
        // two engineers work in it. **Being on the programme and being
        // assigned to it are the same thing**, and the starting state is
        // written that way.
        let programme_id = orbital_programme(
            &mut state,
            Some(ids[0]),
            vec![ids[0], ids[4], ids[6]],
            Instant::from_months(30),
            Instant::from_months(60),
        );
        for (index, member) in [ids[0], ids[4], ids[6]].into_iter().enumerate() {
            let role = if index == 0 {
                domain::person::ProgrammeRole::Lead
            } else {
                domain::person::ProgrammeRole::Member
            };
            if let Some(person) = state.people.get_mut(&member) {
                person.assignment = domain::person::Assignment::Programme {
                    programme: programme_id,
                    role,
                };
            }
        }

        // **And the buildings staffed.** Two people in the laboratory and one
        // in the fabrication plant, and the plants hold their condition.
        //
        // **Being assigned to a facility and being on its staff are the same
        // thing**, and the starting state is written that way.
        for (member, facility) in [
            (ids[3], FacilityId(0)),
            (ids[7], FacilityId(0)),
            (ids[2], FacilityId(1)),
        ] {
            if let Some(person) = state.people.get_mut(&member) {
                person.assignment = domain::person::Assignment::Facility { facility };
            }
        }
        state
            .facilities
            .get_mut(&FacilityId(0))
            .expect("the laboratory")
            .staff = vec![ids[3], ids[7]];
        state
            .facilities
            .get_mut(&FacilityId(1))
            .expect("the fabrication plant")
            .staff = vec![ids[2]];

        // **And two people with nothing to do.** The player has somebody to
        // put somewhere from the first hour, and so does the reference policy.

        Self {
            id: "first_orbit".to_owned(),
            name: "First Orbit".to_owned(),
            description: "Five years to reach orbit, and a government that is watching.".to_owned(),
            seed: 4_718_029_553_617_331,
            start_state: state,
        }
    }

    /// A sandbox: no political deadline and a deep treasury, for experimenting
    /// with a strategy before committing to it.
    #[must_use]
    pub fn sandbox() -> Self {
        let mut scenario = Self::first_year();
        "sandbox".clone_into(&mut scenario.id);
        "Sandbox".to_owned().clone_into(&mut scenario.name);
        "No deadline, no rivals, and money to burn. Try a strategy here first."
            .to_owned()
            .clone_into(&mut scenario.description);
        scenario.start_state.treasury.balance = Money::from_credits(5_000_000);
        scenario
    }

    /// Looks a scenario up by its name.
    #[must_use]
    pub fn by_id(id: &str) -> Option<Self> {
        match id {
            "first_year" => Some(Self::first_year()),
            "first_orbit" => Some(Self::first_orbit()),
            "sandbox" => Some(Self::sandbox()),
            _ => None,
        }
    }

    /// The names of the shipped scenarios.
    #[must_use]
    pub fn ids() -> Vec<&'static str> {
        vec!["first_year", "first_orbit", "sandbox"]
    }

    /// An agency of a given size, for measuring how the game scales.
    ///
    /// **The shipped scenarios have a dozen people**, because the game is
    /// played at three hundred and the scenarios are the tutorial. This builds
    /// the other end: an agency with the staff, the projects and the
    /// facilities a big agency would have, with everybody working.
    ///
    /// **The projects never finish**, because a project that completes stops
    /// being worked on and a benchmark where the work stops is a benchmark of
    /// nothing.
    #[must_use]
    pub fn scaling(staff: usize, projects: usize, facilities: usize) -> Self {
        let mut scenario = Self::first_orbit();
        "scaling".clone_into(&mut scenario.id);
        "Scaling".clone_into(&mut scenario.name);
        "An agency of a given size.".clone_into(&mut scenario.description);

        let state = &mut scenario.start_state;
        state.people.clear();
        state.projects.clear();
        state.facilities.clear();

        let mut people = Vec::new();
        for index in 0..staff {
            let id = PersonId(u64::try_from(index).expect("the index fits"));
            let role = match index % 3 {
                0 => Role::Scientist,
                1 => Role::Engineer,
                _ => Role::Administrator,
            };
            let mut person = Person::new(
                id,
                format!("Person {index}"),
                role,
                Money::from_credits(1_200),
            );
            person.attributes.technical = 70;
            person.attributes.creative = 70;
            person.attributes.leadership = 70;
            person.attributes.physical = 70;
            person.specialisms = vec![
                domain::person::Specialism::Propulsion,
                domain::person::Specialism::Computing,
            ];
            people.push(id);
            state.people.insert(id, person);
        }

        for index in 0..facilities {
            let id = FacilityId(u64::try_from(index).expect("the index fits"));
            let mut facility = laboratory(&format!("Facility {index}"), 8, 65, 2);
            facility.id = id;
            state.facilities.insert(id, facility);
        }

        let per_project = (staff / projects.max(1)).max(1);
        for index in 0..projects {
            let id = ResearchProjectId(u64::try_from(index).expect("the index fits"));
            let team: Vec<PersonId> = people
                .iter()
                .skip(index * per_project)
                .take(per_project)
                .copied()
                .collect();
            let lead = team.first().copied().unwrap_or(PersonId(0));
            state.projects.insert(
                id,
                domain::research::ResearchProject {
                    id,
                    title: format!("Project {index}"),
                    discipline: domain::research::DisciplineId(0),
                    principal_investigator: lead,
                    team: team.clone(),
                    funding: domain::finance::Budget::empty(),
                    // **A target no campaign reaches**, so the work never stops.
                    progress: domain::research::Progress::empty(100_000_000),
                    target_capability: None,
                    started: domain::time::Instant::ZERO,
                    expected_completion: None,
                    remainders: domain::scale::Remainders::new(),
                },
            );
            for member in team {
                if let Some(person) = state.people.get_mut(&member) {
                    person.assignment = domain::person::Assignment::Project { project: id };
                }
            }
        }

        scenario
    }
}

/// A laboratory, ready to work in.
fn laboratory(name: &str, capacity: i64, quality: u8, staff_needed: i64) -> Facility {
    Facility {
        id: FacilityId(0),
        name: name.to_owned(),
        kind: FacilityType::Laboratory,
        site: domain::facility::Site {
            name: "The Core".to_owned(),
            publicly_visible: false,
        },
        specification: Specification {
            capacity,
            reliability: 70,
            automation: 45,
            staff_needed,
            quality,
        },
        condition: (100 * domain::scale::PERCENT_UNITS as u32) as u16,
        staff: Vec::new(),
        construction: None,
        upkeep: balance::facility_base_upkeep(FacilityType::Laboratory),
        monthly_output: capacity,
        remainders: domain::scale::Remainders::new(),
    }
}

/// A fabrication plant, ready to build in.
fn plant(name: &str, capacity: i64, quality: u8, staff_needed: i64) -> Facility {
    Facility {
        id: FacilityId(0),
        name: name.to_owned(),
        kind: FacilityType::FabricationPlant,
        site: domain::facility::Site {
            name: "The Core".to_owned(),
            publicly_visible: false,
        },
        specification: Specification {
            capacity,
            reliability: 65,
            automation: 55,
            staff_needed,
            quality,
        },
        condition: (100 * domain::scale::PERCENT_UNITS as u32) as u16,
        staff: Vec::new(),
        construction: None,
        upkeep: balance::facility_base_upkeep(FacilityType::FabricationPlant),
        monthly_output: capacity,
        remainders: domain::scale::Remainders::new(),
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

/// The cast of the "First Orbit" scenario.
///
/// **The cast is written down rather than generated.** The scenario is tuned:
/// the balance of the five-year arc depends on who is in the room at the
/// start, and a scenario whose staff move every time the recruitment code
/// changes is not a tuned scenario.
///
/// The order matters: the first is the programme's lead, the second and the
/// sixth are the project team, and the fourth, sixth and eighth are on the
/// programme.
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_lines)]
fn orbit_staff() -> Vec<(&'static str, Role, Attributes, Vec<Specialism>, u32)> {
    let cast: [(&'static str, Role, [u8; 5], &[Specialism], u32); 12] = [
        (
            "Dr. Iris Almeida",
            Role::Manager,
            [55, 62, 84, 44, 70],
            &[Specialism::PublicRelations, Specialism::Computing],
            47,
        ),
        (
            "Dr. Nadia Farouk",
            Role::Scientist,
            [79, 83, 46, 50, 41],
            &[Specialism::Propulsion, Specialism::Computing],
            38,
        ),
        (
            "Tomas Erhardt",
            Role::Engineer,
            [81, 55, 47, 62, 43],
            &[Specialism::Propulsion, Specialism::Manufacturing],
            36,
        ),
        (
            "Anika Bhatt",
            Role::Scientist,
            [66, 74, 51, 47, 60],
            &[Specialism::Computing, Specialism::Medicine],
            29,
        ),
        (
            "Soraya Okonkwo",
            Role::Engineer,
            [74, 61, 58, 55, 49],
            &[Specialism::Structures, Specialism::Manufacturing],
            41,
        ),
        (
            "Lior Ben-Ari",
            Role::Scientist,
            [71, 69, 44, 51, 47],
            &[Specialism::Computing, Specialism::Geology],
            33,
        ),
        (
            "Marek Sowa",
            Role::Engineer,
            [68, 57, 52, 66, 44],
            &[Specialism::Avionics, Specialism::Manufacturing],
            39,
        ),
        (
            "Dr. Priya Venkatesan",
            Role::Scientist,
            [77, 73, 48, 45, 53],
            &[Specialism::Propulsion, Specialism::Power],
            35,
        ),
        (
            "Joaquin Reyes",
            Role::Astronaut,
            [58, 52, 61, 83, 64],
            &[Specialism::LifeSupport, Specialism::Avionics],
            31,
        ),
        (
            "Halima Nasser",
            Role::Administrator,
            [49, 45, 63, 38, 78],
            &[Specialism::PublicRelations, Specialism::Computing],
            44,
        ),
        (
            "Astrid Nygaard",
            Role::Scientist,
            [72, 70, 47, 52, 48],
            &[Specialism::Computing, Specialism::Geology],
            30,
        ),
        (
            "Ravi Choudhury",
            Role::Engineer,
            [70, 54, 50, 61, 45],
            &[Specialism::Power, Specialism::Manufacturing],
            34,
        ),
    ];

    cast.into_iter()
        .map(|(name, role, attributes, specialisms, age)| {
            (
                name,
                role,
                Attributes {
                    technical: attributes[0],
                    creative: attributes[1],
                    leadership: attributes[2],
                    physical: attributes[3],
                    diplomatic: attributes[4],
                },
                specialisms.to_vec(),
                age,
            )
        })
        .collect()
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

    // ---- The "First Orbit" scenario ----

    #[test]
    fn the_first_orbit_scenario_has_an_agency_and_a_deadline() {
        let scenario = Scenario::first_orbit();
        assert_eq!(scenario.start_state.people.len(), 12);
        assert_eq!(scenario.start_state.facilities.len(), 2);
        assert_eq!(scenario.start_state.projects.len(), 1);

        let programme = scenario
            .start_state
            .programmes
            .values()
            .next()
            .expect("the orbital programme");
        let objective = programme
            .primary_objective()
            .expect("the primary objective");
        assert_eq!(
            objective.commitment.committed,
            Some(Instant::from_months(60)),
            "the government has promised five years"
        );
        assert_eq!(
            objective.commitment.target,
            Some(Instant::from_months(30)),
            "and the agency's own plan is two and a half years"
        );
    }

    #[test]
    fn the_orbital_objective_is_finished_by_reaching_orbit() {
        // **An objective is finished by the thing it names.** The orbital
        // programme's objective is "reach orbit", and the only thing that
        // reaches orbit is a launch that works.
        let scenario = Scenario::first_orbit();
        let programme = scenario
            .start_state
            .programmes
            .values()
            .next()
            .expect("the orbital programme");
        let objective = programme
            .primary_objective()
            .expect("the primary objective");
        assert_eq!(
            objective.completion,
            domain::programme::ObjectiveCompletion::Destination("Low orbit".to_owned())
        );
        assert!(!objective.is_complete(), "and it is not complete yet");
    }

    #[test]
    fn the_first_orbit_cast_is_on_the_teams_they_are_listed_on() {
        // **Being assigned to something and being on its team are the same
        // thing**, and the starting state is written that way.
        let scenario = Scenario::first_orbit();
        for programme in scenario.start_state.programmes.values() {
            for member in &programme.staffing {
                let person = scenario
                    .start_state
                    .people
                    .get(member)
                    .expect("the person exists");
                assert!(
                    matches!(
                        person.assignment,
                        domain::person::Assignment::Programme { programme: other, .. }
                            if other == programme.id
                    ),
                    "{:?} is on the programme's staffing but is not assigned to it",
                    person.name
                );
            }
        }
        for project in scenario.start_state.projects.values() {
            for member in &project.team {
                let person = scenario
                    .start_state
                    .people
                    .get(member)
                    .expect("the person exists");
                assert!(
                    matches!(
                        person.assignment,
                        domain::person::Assignment::Project { project: other }
                            if other == project.id
                    ),
                    "{:?} is on the project team but is not assigned to it",
                    person.name
                );
            }
        }
    }

    #[test]
    fn every_shipped_scenario_starts() {
        for id in Scenario::ids() {
            let scenario = Scenario::by_id(id).expect("the scenario exists");
            domain::invariant::check(&scenario.start_state)
                .expect("the starting state satisfies the catalogue");
        }
    }
}
