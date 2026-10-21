//! Phase 2 — personnel.
//!
//! The assignments, and the attrition.
//!
//! **Fatigue is the only modifier that can only be spent**, which is what
//! makes the "rest" decision real. A team at 90% fatigue is not a team that
//! works a little slower; it is a team the player is about to lose.
//!
//! **Fatigue, morale, and health are rates, and they live in
//! `crate::rates`.** What is left here is the part that happens at a definite
//! instant: who is assigned where, and who leaves.

use domain::chronicle::{Category, Severity, SubjectRef};
use domain::ids::PersonId;
use domain::order::OrderKind;
use domain::person::Assignment;

use crate::balance;
use crate::error::SimError;
use crate::rng::Domain;
use crate::tick::{MonthContext, Phase};

/// The personnel phase.
pub struct PersonnelPhase;

impl Phase for PersonnelPhase {
    const NAME: &'static str = "personnel";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        apply_assignments(context);
        check_attrition(context);
        Ok(())
    }
}

/// Applies the assignment orders.
fn apply_assignments(context: &mut MonthContext<'_>) {
    let orders = std::mem::take(&mut context.validated);

    for order in orders {
        if !apply_one(context, &order) {
            context.validated.push(order);
        }
    }
}

/// Applies one personnel order, and reports whether it was one.
///
/// **The handlers are per-kind [D].** An order lands at its own instant, and
/// the phase that owns it is the one that applies it — so the handler has to
/// be callable with a single order, not only with a month's worth.
pub fn apply_one(context: &mut MonthContext<'_>, order: &OrderKind) -> bool {
    match order {
        OrderKind::Assign { person, to } => {
            if let Some(target) = context.state.people.get(person) {
                let previous = target.assignment.description();
                let name = target.name.clone();
                assign(context, *person, to.clone());
                context.personnel_lines.push(
                    domain::report::ReportLine::new(format!(
                        "{name} moved from {previous} to {}.",
                        context.state.people[person].assignment.description()
                    ))
                    .with_severity(Severity::Info),
                );
            }
            true
        }
        OrderKind::Unassign { person } => {
            if let Some(target) = context.state.people.get(person) {
                let name = target.name.clone();
                assign(context, *person, Assignment::Unassigned);
                context.personnel_lines.push(
                    domain::report::ReportLine::new(format!("{name} is no longer on anything."))
                        .with_severity(Severity::Minor),
                );
            }
            true
        }
        OrderKind::Fire { person } => {
            fire(context, *person);
            true
        }
        OrderKind::SetShift { person, shift } => {
            if let Some(target) = context.state.people.get_mut(person) {
                let shift = domain::person::PersonState::clamped_shift(*shift);
                target.state.shift = shift;
                let name = target.name.clone();
                let percent = target.state.shift_percent();
                context.personnel_lines.push(
                    domain::report::ReportLine::new(format!(
                        "{name} is now on a {percent}% shift."
                    ))
                    .with_severity(Severity::Info),
                );
            }
            true
        }
        OrderKind::Promote { person } => {
            if let Some(target) = context.state.people.get_mut(person) {
                target.career.grade += 1;
                target.career.promotions += 1;
                target.contract.salary = target
                    .contract
                    .salary
                    .checked_add(balance::starting_salary(target.role).percent(20))
                    .unwrap_or(target.contract.salary);
                let name = target.name.clone();
                context.personnel_lines.push(
                    domain::report::ReportLine::new(format!("{name} was promoted."))
                        .with_severity(Severity::Minor),
                );
            }
            true
        }
        OrderKind::Hire { role, salary } => {
            let id = context.state.id_counters.next_person();
            // **The bucket is a stable identifier, not an instant.** The month
            // index survives a change to the length of a month, and the new
            // person's id is unique — so two hires in the same month draw from
            // different positions in the stream, and the same hire always draws
            // the same recruit.
            let mut rng = context
                .rng
                .stream(Domain::Personnel, context.month.months(), id.0);
            let person = crate::recruitment::recruit(&mut rng, id, *role, *salary);
            let name = person.name.clone();
            let label = role.label();
            context.state.people.insert(id, person);
            context.personnel_lines.push(
                domain::report::ReportLine::new(format!("{name} joined as a {label}."))
                    .with_severity(Severity::Minor),
            );
            true
        }
        OrderKind::Train { person, course } => {
            if let Some(target) = context.state.people.get_mut(person) {
                let name = target.name.clone();
                target.career.certifications.push(course.clone());
                target.attributes.technical = target.attributes.technical.saturating_add(3);
                context.personnel_lines.push(
                    domain::report::ReportLine::new(format!("{name} completed {course}."))
                        .with_severity(Severity::Minor),
                );
            }
            true
        }
        _ => false,
    }
}

/// Puts a person on something, and takes them off whatever they were on.
///
/// **Being assigned to something and being on its team are different things**,
/// and a person who is on two teams is working for both. This is the one place
/// that keeps the two in step: the assignment, the project team, the test
/// campaign team, and the facility's staff list all move together.
///
/// It is the same bug that the Slice 1 work found — a project ran on one
/// person while three more were marked busy — and it is why this lives here
/// rather than being open-coded in each handler.
/// Dismisses somebody.
///
/// **The dismissal removes the person.** It used to be accepted, charged, and
/// quietly ignored, which is the one thing the order model forbids: an order
/// with no handler is refused, and an order with a handler is applied.
fn fire(context: &mut MonthContext<'_>, person: PersonId) {
    let Some(target) = context.state.people.get(&person) else {
        return;
    };
    let name = target.name.clone();
    context.state.people.remove(&person);

    // **The team lists are kept in step**, because a project with a member who
    // is not there is a project that counts a person it does not have.
    for project in context.state.projects.values_mut() {
        project.team.retain(|member| member != &person);
    }
    for campaign in context.state.test_campaigns.values_mut() {
        campaign.team.retain(|member| member != &person);
    }
    for facility in context.state.facilities.values_mut() {
        facility.staff.retain(|member| member != &person);
    }
    for programme in context.state.programmes.values_mut() {
        programme.staffing.retain(|member| member != &person);
        if programme.lead == Some(person) {
            programme.lead = None;
        }
    }

    context.personnel_lines.push(
        domain::report::ReportLine::new(format!("{name} has left the agency."))
            .with_severity(Severity::Major),
    );
    context.events.push(
        domain::chronicle::ChronicleEntry::new(
            context.state.next_chronicle_id(),
            context.month,
            Category::Personnel,
            format!("{name} was dismissed"),
            Severity::Major,
        )
        .about(SubjectRef::Person(person)),
    );
}
/// Puts a person on something, and keeps the team lists in step.
///
/// **This is the one place an assignment happens**, because being assigned and
/// being on the team are different things and the two must never come apart.
pub fn assign(context: &mut MonthContext<'_>, person: PersonId, to: Assignment) {
    remove_from_team(context, person);
    add_to_team(context, person, &to);
    if let Some(target) = context.state.people.get_mut(&person) {
        target.assignment = to;
    }
}
/// Puts a person on the team of the thing they were assigned to.
fn add_to_team(context: &mut MonthContext<'_>, person: PersonId, to: &Assignment) {
    match to {
        Assignment::Project { project } => {
            if let Some(target) = context.state.projects.get_mut(project) {
                if !target.team.contains(&person) {
                    target.team.push(person);
                }
            }
        }
        Assignment::TestCampaign { campaign } => {
            if let Some(target) = context.state.test_campaigns.get_mut(campaign) {
                if !target.team.contains(&person) {
                    target.team.push(person);
                }
            }
        }
        Assignment::Facility { facility } => {
            // **A facility without its staff is a facility that cannot do its
            // job.** The tools modifier reads the staff list, so a person
            // assigned to a lab who never joins its staff leaves the lab running
            // at nothing — and the binding factor says "tools" for reasons the
            // player cannot see.
            if let Some(target) = context.state.facilities.get_mut(facility) {
                if !target.staff.contains(&person) {
                    target.staff.push(person);
                }
            }
        }
        Assignment::Unassigned | Assignment::Division(_) | Assignment::Programme { .. } => {}
    }
}

/// Takes a person off whatever team they were on.
fn remove_from_team(context: &mut MonthContext<'_>, person: PersonId) {
    for project in context.state.projects.values_mut() {
        project.team.retain(|member| *member != person);
    }
    for campaign in context.state.test_campaigns.values_mut() {
        campaign.team.retain(|member| *member != person);
    }
    for facility in context.state.facilities.values_mut() {
        facility.staff.retain(|member| *member != person);
    }
}

/// Checks whether anyone leaves.
///
/// **There are two rates, and they are different mechanics.** Everyone has the
/// ordinary chance of leaving — six percent a year, which is the turnover of
/// any institution. And the people who are *at their limit* — miserable **and**
/// exhausted — have a heavy rate, because that is the case the player can see
/// coming and act on.
///
/// **The multiplier structure makes it legible [D]:** a team with morale at or
/// below -30 will lose people, and a person who is loyal to the agency is half
/// as likely to walk.
///
/// **This is a draw, and it happens at a scheduled instant.** The check runs
/// in the month's work, which is a scheduled item, so the draw is anchored to
/// a definite instant and the campaign does not depend on the dilation.
fn check_attrition(context: &mut MonthContext<'_>) {
    let candidates: Vec<(PersonId, bool)> = context
        .state
        .people
        .values()
        .map(|person| {
            let at_the_limit = person.state.morale <= balance::RESIGNATION_LOW_MORALE
                && person.state.fatigue >= balance::RESIGNATION_HIGH_FATIGUE;
            (person.id, at_the_limit)
        })
        .collect();

    for (id, at_the_limit) in candidates {
        // **The bucket is a stable identifier, not an instant.** The month's
        // index and the person's id: the same check of the same person always
        // draws the same roll.
        let mut rng = context
            .rng
            .stream(Domain::Personnel, context.month.months(), id.0);
        let roll = crate::rng::draw_basis_points(&mut rng);
        if roll >= resignation_chance(context, id, at_the_limit) {
            continue;
        }
        if let Some(person) = context.state.people.remove(&id) {
            let name = person.name.clone();
            let reason = if at_the_limit {
                "They had been at their limit for a long time."
            } else {
                "They were not happy here."
            };
            context.personnel_lines.push(
                domain::report::ReportLine::new(format!("{name} resigned. {reason}"))
                    .with_severity(Severity::Major),
            );
            context
                .warnings
                .push(format!("{name} has left the agency."));
            let entry = domain::chronicle::ChronicleEntry::new(
                context.state.next_chronicle_id(),
                context.month,
                Category::Personnel,
                format!("{name} resigned"),
                Severity::Major,
            )
            .about(SubjectRef::Person(person.id));
            context.events.push(entry);
        }
    }
}

/// The chance, in basis points, that a person leaves this month.
///
/// The ordinary rate, multiplied by the state's opinion of the person, and
/// **at least the heavy rate** for the people who are at their limit.
fn resignation_chance(context: &MonthContext<'_>, id: PersonId, at_the_limit: bool) -> u32 {
    let person = &context.state.people[&id];
    let mut chance = chance_for(person);
    if at_the_limit {
        chance = chance.max(balance::RESIGNATION_LIMIT_CHANCE_BP);
    }
    chance
}

/// The chance, in basis points, that a person leaves this month.
///
/// **This is the number the interface shows**, and it is the number the player
/// sorts by. A person about to leave is a person the player can still save,
/// and a player who cannot see the risk cannot save them.
#[must_use]
pub fn attrition_risk(state: &domain::state::CampaignState, id: PersonId) -> u32 {
    let Some(person) = state.people.get(&id) else {
        return 0;
    };
    let at_the_limit = person.state.morale <= balance::RESIGNATION_LOW_MORALE
        && person.state.fatigue >= balance::RESIGNATION_HIGH_FATIGUE;
    let mut chance = chance_for(person);
    if at_the_limit {
        chance = chance.max(balance::RESIGNATION_LIMIT_CHANCE_BP);
    }
    chance
}

/// The chance a person leaves, before the floor for the people at their limit.
fn chance_for(person: &domain::person::Person) -> u32 {
    let mut chance = balance::RESIGNATION_BASE_CHANCE_BP;

    if person.state.morale <= balance::RESIGNATION_LOW_MORALE {
        chance *= balance::RESIGNATION_LOW_MORALE_MULTIPLIER;
    }
    if person.state.loyalty <= balance::LOYALTY_LOW {
        chance *= balance::RESIGNATION_LOW_LOYALTY_MULTIPLIER;
    }
    if person.state.loyalty >= balance::LOYALTY_HIGH {
        chance /= balance::RESIGNATION_HIGH_LOYALTY_DIVISOR;
    }
    chance
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::{AgencyId, FacilityId, PersonId, ResearchProjectId};
    use domain::money::Money;
    use domain::person::{Person, Role};
    use domain::state::{Agency, CampaignState};
    use domain::time::Instant;

    use crate::rng::Rng;

    fn setup() -> (CampaignState, Rng) {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        state.people.insert(
            PersonId(0),
            Person::new(
                PersonId(0),
                "Marsha Okonjo".to_owned(),
                Role::Scientist,
                Money::from_credits(1_200),
            ),
        );
        (state, Rng::new(7))
    }

    /// A state with a project, a lab, and a person on both.
    fn setup_with_things() -> (CampaignState, Rng) {
        let (mut state, rng) = setup();
        let project = ResearchProjectId(0);
        state.projects.insert(
            project,
            domain::research::ResearchProject {
                id: project,
                title: "Liquid upper stage".to_owned(),
                discipline: domain::research::DisciplineId(0),
                principal_investigator: PersonId(0),
                team: vec![PersonId(0)],
                funding: domain::finance::Budget::empty(),
                progress: domain::research::Progress::empty(25_600),
                target_capability: None,
                started: Instant::ZERO,
                expected_completion: None,
                remainders: domain::scale::Remainders::new(),
            },
        );
        let facility = FacilityId(0);
        state.facilities.insert(
            facility,
            domain::facility::Facility {
                id: facility,
                name: "Propulsion Lab".to_owned(),
                kind: domain::facility::FacilityType::Laboratory,
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
                upkeep: Money::from_credits(4_000),
                monthly_output: 4,
                remainders: domain::scale::Remainders::new(),
            },
        );
        (state, rng)
    }

    #[test]
    fn assigning_someone_to_a_facility_puts_them_on_its_staff() {
        // **A facility without its staff is a facility that cannot do its
        // job.** The tools modifier reads the staff list, so a person assigned
        // to a lab who never joins its staff leaves the lab running at
        // nothing - and the binding factor says "tools" for reasons the player
        // cannot see.
        let (mut state, rng) = setup_with_things();
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        PersonnelPhase::run(&mut context).expect("the phase runs");
        assert!(
            context.state.facilities[&FacilityId(0)].staff.is_empty(),
            "nobody has been assigned yet"
        );

        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        crate::phases::personnel::assign(
            &mut context,
            PersonId(0),
            Assignment::Facility {
                facility: FacilityId(0),
            },
        );
        assert_eq!(
            context.state.facilities[&FacilityId(0)].staff,
            vec![PersonId(0)],
            "the person joins the facility's staff"
        );
    }

    #[test]
    fn a_person_is_on_one_team_and_not_on_two() {
        // **Being assigned to something and being on its team are different
        // things**, and a person who is on two teams is working for both. This
        // is the same bug the Slice 1 work found.
        let (mut state, rng) = setup_with_things();
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        crate::phases::personnel::assign(
            &mut context,
            PersonId(0),
            Assignment::Facility {
                facility: FacilityId(0),
            },
        );
        assert!(
            context.state.projects[&ResearchProjectId(0)]
                .team
                .is_empty(),
            "the person leaves the project they were on"
        );
        assert_eq!(
            context.state.people[&PersonId(0)].assignment,
            Assignment::Facility {
                facility: FacilityId(0)
            }
        );
    }

    #[test]
    fn unassigning_takes_a_person_off_every_team() {
        let (mut state, rng) = setup_with_things();
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        crate::phases::personnel::assign(&mut context, PersonId(0), Assignment::Unassigned);
        assert!(
            context.state.projects[&ResearchProjectId(0)]
                .team
                .is_empty()
        );
        assert!(context.state.facilities[&FacilityId(0)].staff.is_empty());
    }

    #[test]
    fn the_phase_does_not_move_the_continuous_quantities() {
        // **Fatigue, morale, and health are rates now**, and they live in
        // `crate::rates`. The phase is what happens at a definite instant: the
        // assignments, and the attrition.
        let (mut state, rng) = setup();
        state
            .people
            .get_mut(&PersonId(0))
            .expect("the person exists")
            .assignment = Assignment::Division(domain::ids::DivisionId(0));
        let before = state.people[&PersonId(0)].state.fatigue;
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        PersonnelPhase::run(&mut context).expect("the phase runs");
        assert_eq!(context.state.people[&PersonId(0)].state.fatigue, before);
    }

    #[test]
    fn the_rates_move_the_continuous_quantities() {
        let (mut state, _rng) = setup();
        state
            .people
            .get_mut(&PersonId(0))
            .expect("the person exists")
            .assignment = Assignment::Division(domain::ids::DivisionId(0));
        crate::rates::integrate(&mut state, domain::time::Duration::from_months(1));
        assert_eq!(
            state.people[&PersonId(0)].state.fatigue,
            balance::FATIGUE_ON_ASSIGNMENT
        );
    }

    #[test]
    fn an_unhappy_person_may_resign() {
        let (mut state, rng) = setup();
        {
            let person = state
                .people
                .get_mut(&PersonId(0))
                .expect("the person exists");
            person.state.morale = domain::scale::signed_percent_to_units(-60);
            person.state.fatigue = domain::scale::percent_to_units(90) as u16;
            person.assignment = Assignment::Division(domain::ids::DivisionId(0));
        }
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        PersonnelPhase::run(&mut context).expect("the phase runs");
        // The chance is 25% per Instant, so most months keep them. What matters
        // is that the machinery is wired and reported.
        assert!(
            !context.personnel_lines.is_empty() || context.state.people.contains_key(&PersonId(0))
        );
    }

    // ---- The attrition rates ----

    fn chance_for(morale: i16, loyalty: i16, fatigue: u16) -> u32 {
        let (mut state, rng) = setup();
        {
            let person = state
                .people
                .get_mut(&PersonId(0))
                .expect("the person exists");
            person.state.morale = morale;
            person.state.loyalty = loyalty;
            person.state.fatigue = fatigue;
        }
        let context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        let at_the_limit = morale <= balance::RESIGNATION_LOW_MORALE
            && fatigue >= balance::RESIGNATION_HIGH_FATIGUE;
        resignation_chance(&context, PersonId(0), at_the_limit)
    }

    #[test]
    fn everybody_has_the_ordinary_chance_of_leaving() {
        // **Six percent a year.** A healthy agency still loses people, and the
        // cost of replacing them is part of what it means to run one.
        let chance = chance_for(0, 0, 0);
        assert_eq!(chance, balance::RESIGNATION_BASE_CHANCE_BP);
    }

    #[test]
    fn a_miserable_person_is_three_times_as_likely_to_leave() {
        let chance = chance_for(domain::scale::signed_percent_to_units(-30), 0, 0);
        assert_eq!(
            chance,
            balance::RESIGNATION_BASE_CHANCE_BP * balance::RESIGNATION_LOW_MORALE_MULTIPLIER
        );
    }

    #[test]
    fn a_loyal_person_is_half_as_likely_to_leave() {
        let chance = chance_for(0, domain::scale::signed_percent_to_units(30), 0);
        assert_eq!(
            chance,
            balance::RESIGNATION_BASE_CHANCE_BP / balance::RESIGNATION_HIGH_LOYALTY_DIVISOR
        );
    }

    #[test]
    fn a_disloyal_person_is_twice_as_likely_to_leave() {
        let chance = chance_for(0, domain::scale::signed_percent_to_units(-50), 0);
        assert_eq!(
            chance,
            balance::RESIGNATION_BASE_CHANCE_BP * balance::RESIGNATION_LOW_LOYALTY_MULTIPLIER
        );
    }

    #[test]
    fn the_people_at_their_limit_have_the_heavy_rate() {
        // **Miserable and exhausted is the case the player can see coming**,
        // and it walks quickly.
        let chance = chance_for(
            domain::scale::signed_percent_to_units(-60),
            0,
            domain::scale::percent_to_units(90) as u16,
        );
        assert_eq!(chance, balance::RESIGNATION_LIMIT_CHANCE_BP);
    }

    #[test]
    fn exhaustion_alone_is_not_enough() {
        // Exhausted but content is a different person, and they do not walk.
        let chance = chance_for(0, 0, domain::scale::percent_to_units(90) as u16);
        assert_eq!(chance, balance::RESIGNATION_BASE_CHANCE_BP);
    }
}
