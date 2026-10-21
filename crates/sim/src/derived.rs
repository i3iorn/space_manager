//! Every derived value: performance, morale, approval, risk, slack, burn rate.
//!
//! Derived values are computed, never stored. If recomputation is cheap, do it;
//! if it is not, cache it with an explicit dirty flag. **Never cache by time** —
//! a time-based cache is a non-determinism source. See `docs/DOMAIN_MODEL.md` §19.

use domain::money::Money;
use domain::performance::{Modifier, ModifierKind, PerformanceInput, PerformanceResult};
use domain::person::{Person, Specialism};
use domain::state::CampaignState;

use crate::balance;
use crate::performance;

/// What kind of work is being done.
///
/// The kind decides which ability and which specialism matter, and it is what
/// makes the same person good at one thing and poor at another.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskKind {
    /// Scientific investigation.
    Research,
    /// Design and construction.
    Engineering,
    /// Running a programme and its people.
    Management,
    /// Preparing for and flying missions.
    Flight,
}

impl TaskKind {
    /// The specialism this kind of work calls for, if any.
    #[must_use]
    pub const fn specialism(self) -> Option<Specialism> {
        match self {
            Self::Research => Some(Specialism::Computing),
            Self::Engineering => Some(Specialism::Propulsion),
            Self::Management => Some(Specialism::PublicRelations),
            Self::Flight => Some(Specialism::LifeSupport),
        }
    }
}

/// The performance of a person on a piece of work.
///
/// This is the C+B hybrid, with the modifiers filled in from the person's state
/// and their surroundings.
#[must_use]
pub fn person_performance(
    state: &CampaignState,
    person: &Person,
    task: TaskKind,
    team_size: usize,
    lead_present: bool,
) -> PerformanceResult {
    let attribute = match task {
        TaskKind::Research => person.attributes.creative,
        TaskKind::Engineering => person.attributes.technical,
        TaskKind::Management => person.attributes.leadership,
        TaskKind::Flight => person.attributes.physical,
    };

    let required = task.specialism();
    let matches = required.map_or(0, |needed| {
        person.specialisms.iter().filter(|s| **s == needed).count()
    });
    let has_any = required.is_some_and(|needed| person.specialisms.contains(&needed));

    let input = PerformanceInput {
        skill: balance::skill_modifier(attribute),
        role_fit: balance::role_fit_modifier(matches, has_any),
        morale: balance::morale_modifier(person.state.morale),
        support: balance::support_modifier(team_size, lead_present),
        tools: tools_modifier(state),
        fatigue: balance::fatigue_modifier(person.state.fatigue),
        creativity: balance::creativity_modifier(person.attributes.creative),
        health: balance::health_modifier(person.state.health),
    };

    performance::compute(&input)
}

/// The tools factor, with the industrial capacity taken into account.
///
/// **Over capacity is a real constraint.** Every member of staff working at
/// once needs a place to work, and the facilities are the places. An agency
/// with more people working than it has room for works with worse tools, and
/// the factor says so.
#[must_use]
pub fn tools_modifier(state: &CampaignState) -> Modifier {
    let base = balance::tools_modifier(tools_quality(state));
    let (capacity, demand) = industrial_capacity(state);
    if capacity <= 0 || demand <= capacity {
        return base;
    }
    // **The worse the overload, the worse the tools**, and the fall is
    // proportional to how far over the agency is.
    let over = (demand - capacity) * balance::CAPACITY_OVERLOAD_TOOLS_BP / capacity;
    Modifier::new(ModifierKind::Tools, base.value() - over as i16)
}

/// How likely a mission is to work, in basis points.
///
/// **The vehicle's reliability is the whole of it**: a mission on a reliable
/// vehicle works, and a mission on a shaky one does not. The chance is capped
/// below a certainty, because a campaign in which nothing ever fails is not a
/// campaign.
#[must_use]
pub fn mission_probability(state: &CampaignState, vehicle: domain::ids::VehicleId) -> u16 {
    let reliability = state
        .vehicles
        .get(&vehicle)
        .map_or(0, |vehicle| u32::from(vehicle.derived.reliability));
    let basis_points = (reliability * crate::balance::MISSION_BASE_PROBABILITY_BP) / 100;
    u16::try_from(basis_points.min(9_500)).unwrap_or(0)
}

/// The chance of a mission to a destination working, in basis points.
///
/// **The testing is what makes the difference.** A capability that has been
/// tested is a safer capability, and the risk the testing took out comes off the
/// top of the launch's chances.
#[must_use]
pub fn mission_probability_for(
    state: &CampaignState,
    vehicle: domain::ids::VehicleId,
    destination: &str,
) -> u16 {
    let base = u32::from(mission_probability(state, vehicle));
    let reduction = tested_reduction(state, destination);
    let reduction = u32::try_from(reduction.max(0)).unwrap_or(0);
    let reduced = base.saturating_sub(reduction * 100);
    u16::try_from(reduced.min(9_500)).unwrap_or(0)
}

/// The risk a destination's capability has had taken out of it, in points.
#[must_use]
pub fn tested_reduction(state: &CampaignState, destination: &str) -> i16 {
    domain::research::capability_for_destination(destination)
        .and_then(|capability| state.tested.get(&capability).copied())
        .unwrap_or(0)
}

/// The agency's industrial capacity, and the demand on it.
///
/// **The capacity is what the facilities can hold** and the demand is how many
/// people are working at once. The two are compared by the interface and by
/// the tools factor, and they are the reason a bigger laboratory is worth
/// building.
#[must_use]
pub fn industrial_capacity(state: &CampaignState) -> (i64, i64) {
    let capacity: i64 = state
        .facilities
        .values()
        .filter(|facility| facility.is_operational())
        .map(|facility| facility.specification.capacity)
        .sum();
    let demand = state
        .people
        .values()
        .filter(|person| {
            matches!(
                person.assignment,
                domain::person::Assignment::Project { .. }
                    | domain::person::Assignment::Facility { .. }
                    | domain::person::Assignment::TestCampaign { .. }
            )
        })
        .fold(0_i64, |count, _| count.saturating_add(1));
    (capacity, demand)
}
/// The quality of the best facility the agency has for this kind of work.
///
/// A project done in a good lab goes faster than the same project done in a
/// bad one, and the difference is worth showing the player.
#[must_use]
pub fn tools_quality(state: &CampaignState) -> u8 {
    state
        .facilities
        .values()
        .filter(|facility| facility.is_operational())
        .map(|facility| facility.specification.quality)
        .max()
        .unwrap_or(35)
}

/// The modifier describing how well a person fits their assignment.
///
/// A person on a project that suits them is content; a person doing
/// administration when they are a rocket scientist is not.
#[must_use]
pub fn assignment_satisfaction(_state: &CampaignState, person: &Person) -> i16 {
    match &person.assignment {
        domain::person::Assignment::Unassigned => domain::scale::signed_percent_to_units(-10),
        domain::person::Assignment::Project { .. } => {
            if person.specialisms.contains(&Specialism::Propulsion)
                || person.specialisms.contains(&Specialism::Computing)
            {
                balance::MORALE_TARGET_SATISFIED
            } else {
                0
            }
        }
        domain::person::Assignment::Programme { .. }
        | domain::person::Assignment::TestCampaign { .. } => balance::MORALE_TARGET_SATISFIED / 2,
        domain::person::Assignment::Facility { .. }
        | domain::person::Assignment::Division { .. } => 0,
    }
}

/// The number of people assigned to a research project.
#[must_use]
pub fn project_team_size(state: &CampaignState, project: domain::ids::ResearchProjectId) -> usize {
    state
        .projects
        .get(&project)
        .map_or(0, |project| project.team.len())
}

/// The points a programme's objectives gain in a month, at the current rate.
///
/// **This is the same number the rate applies**, and it is here because the
/// interface has to project a finish date and the projection must not
/// disagree with the simulation.
#[must_use]
pub fn programme_rate(state: &CampaignState, programme: domain::ids::ProgrammeId) -> i64 {
    let Some(programme) = state.programmes.get(&programme) else {
        return 0;
    };
    if programme.staffing.is_empty() {
        return 0;
    }
    let team_size = programme.staffing.len().max(1);
    let lead_present = programme.lead.is_some();
    let total: i64 = programme
        .staffing
        .iter()
        .filter_map(|member| state.people.get(member))
        .map(|person| {
            i64::from(
                person_performance(state, person, TaskKind::Management, team_size, lead_present)
                    .effective,
            )
        })
        .sum();
    if total <= 0 {
        return 0;
    }
    // **The points a month, at the target of six hundred points.** The same
    // arithmetic the rate applies, so the projection and the progress cannot
    // disagree.
    let target = i64::from(crate::balance::OBJECTIVE_POINTS_TARGET);
    total * target / 100_000
}

/// When an objective will be finished, at the current rate.
///
/// **The projection is a rate and a remaining distance**, and it is a plan
/// rather than a promise: it is what happens if nothing changes.
#[must_use]
pub fn objective_projection(
    state: &CampaignState,
    now: domain::time::Instant,
    programme: domain::ids::ProgrammeId,
    objective: &domain::programme::Objective,
) -> Option<domain::time::Instant> {
    let rate = programme_rate(state, programme);
    if rate <= 0 {
        return None;
    }
    let remaining = i64::from(
        objective
            .progress
            .target
            .saturating_sub(objective.progress.points),
    );
    if remaining <= 0 {
        return None;
    }
    // **The months round up**, because a projection that finished early would
    // be a lie.
    let months = (remaining + rate - 1) / rate;
    let months = u64::try_from(months).unwrap_or(u64::MAX);
    // **The distance is measured from *now***, because a projection that
    // counted the time already spent would be a projection of the past.
    Some(now.saturating_add(domain::time::Duration::from_months(months)))
}
/// What the agency spends in a month, by line.
///
/// This is the same figure the finance phase reports, and it is the *rate* the
/// treasury drains at. It lives here because two callers need it and they must
/// not disagree.
#[must_use]
pub fn monthly_outgoings_detail(state: &CampaignState) -> Vec<(String, Money)> {
    let mut outgoings = Vec::new();

    let payroll: Money = state
        .people
        .values()
        .map(|person| person.contract.salary)
        .fold(Money::ZERO, |total, salary| {
            total.checked_add(salary).unwrap_or(total)
        });
    if !payroll.is_zero() {
        outgoings.push(("Payroll".to_owned(), payroll));
    }

    let research: Money = state
        .projects
        .values()
        .map(|project| balance::research_cost_per_month(project.team.len()))
        .fold(Money::ZERO, |total, cost| {
            total.checked_add(cost).unwrap_or(total)
        });
    if !research.is_zero() {
        outgoings.push(("Research".to_owned(), research));
    }

    let testing: Money = state
        .test_campaigns
        .values()
        .map(|campaign| balance::test_cost_per_month(campaign.team.len()))
        .fold(Money::ZERO, |total, cost| {
            total.checked_add(cost).unwrap_or(total)
        });
    if !testing.is_zero() {
        outgoings.push(("Testing".to_owned(), testing));
    }

    let upkeep: Money = state
        .facilities
        .values()
        .filter(|facility| facility.is_operational())
        .map(|facility| facility.upkeep)
        .fold(Money::ZERO, |total, cost| {
            total.checked_add(cost).unwrap_or(total)
        });
    if !upkeep.is_zero() {
        outgoings.push(("Upkeep".to_owned(), upkeep));
    }

    outgoings
}

/// How many things need the player's attention.
///
/// **This is the count the top bar shows.** It is a *derived* value and never
/// stored, and it is the one number that answers "what should I be doing
/// about?".
#[must_use]
pub fn needs_attention(state: &CampaignState, now: domain::time::Instant) -> usize {
    let mut count = 0;

    // A commitment that falls due within the month.
    let horizon = now.saturating_add(domain::time::Duration::from_months(1));
    for programme in state.programmes.values() {
        for objective in &programme.objectives {
            if objective.met == Some(true) {
                continue;
            }
            if objective.commitment.due().is_some_and(|due| due <= horizon) {
                count += 1;
            }
        }
    }

    // A promise the state is waiting on.
    count += state.political.outstanding().count();

    // A project with nobody on it.
    count += state
        .projects
        .values()
        .filter(|project| project.team.is_empty())
        .count();

    // A treasury with less than a month left.
    if state.treasury.runway_months() < 1 {
        count += 1;
    }

    // A state that has stopped accommodating.
    if state.political.goodwill.is_obstructive() {
        count += 1;
    }

    count
}

/// What the agency spends in a month, in total.
#[must_use]
pub fn monthly_outgoings(state: &CampaignState) -> Money {
    monthly_outgoings_detail(state)
        .into_iter()
        .fold(Money::ZERO, |total, (_, amount)| {
            total.checked_add(amount).unwrap_or(total)
        })
}

/// The modifier for a person working on a team of a given size.
#[must_use]
pub fn team_support_modifier(team_size: usize, lead_present: bool) -> Modifier {
    balance::support_modifier(team_size, lead_present)
}

/// The single most binding constraint across a team, if any.
///
/// The team is held back by its *weakest* member, which is what makes a single
/// exhausted engineer a management problem rather than a rounding error.
#[must_use]
pub fn team_binding_factor(state: &CampaignState, task: TaskKind) -> Option<ModifierKind> {
    let mut worst: Option<(ModifierKind, u32)> = None;
    for person in state.people.values() {
        if !person.is_assigned() {
            continue;
        }
        let result = person_performance(state, person, task, 1, false);
        for modifier in &result.modifiers {
            let cost = modifier.cost_bp();
            if cost < crate::performance::BINDING_FACTOR_THRESHOLD_BP {
                continue;
            }
            match worst {
                Some((_, worst_cost)) if worst_cost >= cost => {}
                _ => worst = Some((modifier.kind(), cost)),
            }
        }
    }
    worst.map(|(kind, _)| kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::{AgencyId, PersonId};
    use domain::money::Money;
    use domain::person::Assignment;
    use domain::person::Role;
    use domain::state::Agency;

    /// A laboratory with four places in it.
    fn a_facility() -> domain::facility::Facility {
        domain::facility::Facility {
            id: domain::ids::FacilityId(0),
            name: "The Lab".to_owned(),
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
            upkeep: Money::ZERO,
            monthly_output: 4,
            remainders: domain::scale::Remainders::new(),
        }
    }
    fn state_with(person: Person) -> CampaignState {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        state.people.insert(person.id, person);
        state
    }

    fn person() -> Person {
        let mut person = Person::new(
            PersonId(0),
            "Marsha Okonjo".to_owned(),
            Role::Scientist,
            Money::from_credits(1_200),
        );
        person.attributes.technical = 80;
        person.attributes.creative = 80;
        person.specialisms = vec![Specialism::Propulsion, Specialism::Computing];
        person
    }

    #[test]
    fn a_person_with_no_facilities_has_poor_tools() {
        let state = state_with(person());
        assert_eq!(tools_quality(&state), 35);
    }

    #[test]
    fn a_competent_person_outperforms_an_incompetent_one() {
        let mut good = person();
        good.attributes.technical = 90;
        let mut poor = person();
        poor.attributes.technical = 20;

        let good_result = person_performance(
            &state_with(good.clone()),
            &good,
            TaskKind::Engineering,
            4,
            true,
        );
        let poor_result = person_performance(
            &state_with(poor.clone()),
            &poor,
            TaskKind::Engineering,
            4,
            true,
        );
        assert!(good_result.effective > poor_result.effective);
    }

    #[test]
    fn the_right_specialism_matters() {
        let mut engineer = person();
        engineer.specialisms = vec![Specialism::Propulsion];
        let mut wrong = person();
        wrong.specialisms = vec![Specialism::Medicine];

        let good = person_performance(
            &state_with(engineer.clone()),
            &engineer,
            TaskKind::Engineering,
            4,
            true,
        );
        let bad = person_performance(
            &state_with(wrong.clone()),
            &wrong,
            TaskKind::Engineering,
            4,
            true,
        );
        assert!(good.effective > bad.effective);
    }

    #[test]
    fn a_team_is_held_back_by_its_weakest_member() {
        let mut tired = person();
        tired.state.fatigue = domain::scale::percent_to_units(90) as u16;
        tired.assignment = Assignment::Division(domain::ids::DivisionId(0));
        let state = state_with(tired);
        assert_eq!(
            team_binding_factor(&state, TaskKind::Engineering),
            Some(ModifierKind::Fatigue)
        );
    }

    #[test]
    fn over_capacity_makes_the_tools_worse() {
        // **Over capacity is a real constraint**, and it shows in the tools
        // factor and not somewhere else.
        let mut state = state_with(person());
        state
            .facilities
            .insert(domain::ids::FacilityId(0), a_facility());
        // **The person is working**, so the agency has a demand on its
        // capacity.
        state
            .people
            .get_mut(&PersonId(0))
            .expect("the person")
            .assignment = Assignment::Facility {
            facility: domain::ids::FacilityId(0),
        };
        let before = tools_modifier(&state).value();

        // **A second person, and a facility with one place in it.** The
        // agency is over its capacity, and the tools are worse for it.
        let mut second = person();
        second.id = PersonId(1);
        state.people.insert(PersonId(1), second);
        state
            .people
            .get_mut(&PersonId(1))
            .expect("the person")
            .assignment = Assignment::Facility {
            facility: domain::ids::FacilityId(0),
        };
        for facility in state.facilities.values_mut() {
            facility.specification.capacity = 1;
        }
        let after = tools_modifier(&state).value();
        assert!(
            after < before,
            "the tools are worse over capacity: {before} then {after}"
        );
    }

    #[test]
    fn the_capacity_is_what_the_facilities_can_hold() {
        let mut state = state_with(person());
        state
            .facilities
            .insert(domain::ids::FacilityId(0), a_facility());
        // **The person is working**, so the agency has a demand on its
        // capacity.
        state
            .people
            .get_mut(&PersonId(0))
            .expect("the person")
            .assignment = Assignment::Facility {
            facility: domain::ids::FacilityId(0),
        };
        let (capacity, demand) = industrial_capacity(&state);
        assert!(capacity > 0, "the agency has capacity");
        assert!(demand > 0, "and it has people working");
    }
}
