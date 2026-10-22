//! Every derived value: performance, morale, approval, risk, slack, burn rate.
//!
//! Derived values are computed, never stored. If recomputation is cheap, do it;
//! if it is not, cache it with an explicit dirty flag. **Never cache by time** —
//! a time-based cache is a non-determinism source. See `docs/DOMAIN_MODEL.md` §19.

use domain::money::Money;
use domain::performance::{Modifier, ModifierKind, PerformanceInput, PerformanceResult};
use domain::person::{Assignment, Person, ProgrammeRole, Specialism};
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
    let tools = tools_modifier(state);
    person_performance_with_tools(tools, person, task, team_size, lead_present)
}

/// The performance of a person, with the tools factor already computed.
///
/// **The tools factor is the only part of the model that reads the whole
/// state**, and it is the same for everybody: it is the agency's facilities and
/// how many people are working in them. **Computing it once and passing it in
/// is the difference between a linear campaign and a quadratic one** — at three
/// hundred people it was ten seconds a five-year campaign, and now it is a
/// fraction of that.
#[must_use]
pub fn person_performance_with_tools(
    tools: Modifier,
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

    let mut input = PerformanceInput {
        skill: balance::skill_modifier(attribute),
        role_fit: balance::role_fit_modifier(matches, has_any),
        morale: balance::morale_modifier(person.state.morale),
        support: balance::support_modifier(team_size, lead_present),
        tools,
        fatigue: balance::fatigue_modifier(person.state.fatigue),
        creativity: balance::creativity_modifier(person.attributes.creative),
        health: balance::health_modifier(person.state.health),
    };
    // **And the person's traits.**
    //
    // A trait is a small permanent difference in how somebody works, and it
    // lands on the factor it describes. **This is the whole of the trait
    // system**: the traits are data, the data is applied here, and everything
    // else in the performance model is untouched.
    apply_traits(&mut input, &person.traits);

    performance::compute(&input)
}

/// The person's traits, added to their modifiers.
fn apply_traits(input: &mut PerformanceInput, traits: &[domain::person::TraitId]) {
    for id in traits {
        let Some(definition) = domain::person::trait_def(*id) else {
            continue;
        };
        for modifier in &definition.modifiers {
            input.add(*modifier);
        }
    }
}

/// The shift a person actually works.
///
/// **The policy decides, and the individual decides better.** A person who has
/// been moved off the agency policy works their own shift; a person the policy
/// covers works the policy's shift; and a person the policy does not cover
/// works a full load.
///
/// **This is the whole of the policy, and it is a pure function of the state**,
/// so the campaign is still deterministic and still replayable.
#[must_use]
pub fn effective_shift(state: &CampaignState, person: &domain::person::Person) -> u16 {
    if let Some(shift) = person.state.shift_override() {
        return shift;
    }
    let Some(domain::person::Policy::Shift { shift, only_over }) = state.shift_policy else {
        return FULL_SHIFT;
    };
    match only_over {
        // **The thermostat.** The people who are fresher than the threshold keep
        // their own shift, and the people who are tireder than it are eased
        // off.
        Some(threshold) if person.state.fatigue_percent() < u32::from(threshold) => FULL_SHIFT,
        _ => shift,
    }
}

/// The shift a person actually works, as a whole percentage.
#[must_use]
pub fn effective_shift_percent(state: &CampaignState, person: &domain::person::Person) -> u32 {
    domain::scale::units_to_percent(u32::from(effective_shift(state, person)))
}

/// A full load, as a scaled percentage.
const FULL_SHIFT: u16 = 100 * domain::scale::PERCENT_UNITS as u16;

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
    // **The points a month, at the rate the simulation applies.** The same
    // arithmetic the rate applies, so the projection and the progress cannot
    // disagree.
    total * i64::from(domain::scale::PERCENT_UNITS) / 100
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
/// What the state allocates, by source.
///
/// **The allocation scales with the number of people the agency employs, with
/// a floor for a young agency, and is then multiplied by the state's political
/// standing.**
///
/// It is here and not in the finance phase because the interface shows it: the
/// player wants to know where the money comes from, and the answer has to be
/// the same answer the simulation gives.
#[must_use]
pub fn monthly_income(
    state: &CampaignState,
) -> Vec<(domain::finance::IncomeSource, domain::money::Money)> {
    let headcount = i64::try_from(state.people.len()).unwrap_or(i64::MAX);
    let base = (balance::APPROPRIATION_PER_HEAD * headcount).max(balance::APPROPRIATION_FLOOR);

    // **The approval is a scaled integer, and the multiplier wants whole
    // points.** The scale is 256 units to the percent, so a hundred points of
    // approval is 25,600 units — and multiplying the units by the political
    // scale made the allocation twenty-six times what the design says it
    // should be. **A scaled quantity compared against an unscaled threshold is
    // always wrong**, and this one was wrong in the agency's favour.
    let approval =
        i64::from(state.political.approval.value) / i64::from(domain::scale::PERCENT_UNITS);
    let multiplier = 10_000 + approval * balance::APPROPRIATION_POLITICAL_SCALE_BP;
    let amount =
        domain::money::Money::from_credits(base).multiply_fraction(multiplier.max(0), 10_000);

    // **The austerity cut.** The state has imposed a cut, and it is a cut on
    // the state's own money — so it is applied here, where the allocation is
    // computed, rather than somewhere downstream where two things could
    // disagree about how much the agency was given.
    let amount = if state.treasury.crisis_stage.in_austerity() {
        let kept = 100 - balance::AUSTERITY_CUT_PERCENT.min(100);
        amount.multiply_fraction(i64::from(kept), 100)
    } else {
        amount
    };

    vec![(domain::finance::IncomeSource::CivilisationBase, amount)]
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
///
/// **This is the same figure as the sum of the detail**, and it is computed
/// without building the detail: the detail is a list of named lines, and
/// building it allocates, and the burn rate is read on every hour of the
/// campaign.
#[must_use]
pub fn monthly_outgoings(state: &CampaignState) -> Money {
    let payroll: Money = state
        .people
        .values()
        .map(|person| person.contract.salary)
        .fold(Money::ZERO, |total, salary| {
            total.checked_add(salary).unwrap_or(total)
        });
    let research: Money = state
        .projects
        .values()
        .map(|project| balance::research_cost_per_month(project.team.len()))
        .fold(Money::ZERO, |total, cost| {
            total.checked_add(cost).unwrap_or(total)
        });
    let testing: Money = state
        .test_campaigns
        .values()
        .map(|campaign| balance::test_cost_per_month(campaign.team.len()))
        .fold(Money::ZERO, |total, cost| {
            total.checked_add(cost).unwrap_or(total)
        });
    let upkeep: Money = state
        .facilities
        .values()
        .filter(|facility| facility.is_operational())
        .map(|facility| facility.upkeep)
        .fold(Money::ZERO, |total, cost| {
            total.checked_add(cost).unwrap_or(total)
        });
    let total = Money::ZERO;
    [payroll, research, testing, upkeep]
        .into_iter()
        .fold(total, |sum, amount| sum.checked_add(amount).unwrap_or(sum))
}

/// The modifier for a person working on a team of a given size.
#[must_use]
pub fn team_support_modifier(team_size: usize, lead_present: bool) -> Modifier {
    balance::support_modifier(team_size, lead_present)
}

/// Where a person is, in words, with the name in it.
///
/// **The player must never be shown an identifier.** The domain knows what
/// *kind* of thing a person is assigned to but not what it is called, so it
/// can only say "works on a programme"; this function has the state and can
/// say "works on the Orbital Survey programme".
///
/// Every message that names an assignment goes through here. The fallback for
/// a thing that has closed is a name that says so, because a message reading
/// "programme 3" is an implementation detail leaking into the interface.
#[must_use]
pub fn assignment_description(state: &CampaignState, assignment: &Assignment) -> String {
    match assignment {
        Assignment::Unassigned => "Unassigned".to_owned(),
        Assignment::Division(_) => "a division".to_owned(),
        Assignment::Programme { programme, role } => {
            let title = state.programmes.get(programme).map_or_else(
                || "a programme that has closed".to_owned(),
                |programme| programme.title.clone(),
            );
            match role {
                ProgrammeRole::Lead => format!("leads the {title} programme"),
                ProgrammeRole::Member => format!("works on the {title} programme"),
                ProgrammeRole::Adviser => format!("advises on the {title} programme"),
            }
        }
        Assignment::Project { project } => {
            let title = state.projects.get(project).map_or_else(
                || "a project that has closed".to_owned(),
                |project| project.title.clone(),
            );
            format!("the {title} project")
        }
        Assignment::Facility { facility } => state.facilities.get(facility).map_or_else(
            || "a facility that has closed".to_owned(),
            |facility| facility.name.clone(),
        ),
        Assignment::TestCampaign { campaign } => state.test_campaigns.get(campaign).map_or_else(
            || "a test campaign that has closed".to_owned(),
            |campaign| {
                format!(
                    "the {} test campaign",
                    domain::research::capability_name(campaign.target_capability)
                )
            },
        ),
    }
}

/// Where a person is, as a noun phrase.
///
/// **This is the sibling of `assignment_description`, and it is the one a
/// sentence needs when the person is a thing rather than an actor**: "already on
/// the Orbital Launch programme", "off the Combustion stability project".
#[must_use]
pub fn assignment_place(state: &CampaignState, assignment: &Assignment) -> String {
    match assignment {
        Assignment::Unassigned => "not on anything".to_owned(),
        Assignment::Division(_) => "a division".to_owned(),
        Assignment::Programme { programme, .. } => state.programmes.get(programme).map_or_else(
            || "a programme that has closed".to_owned(),
            |programme| format!("the {} programme", programme.title),
        ),
        Assignment::Project { project } => state.projects.get(project).map_or_else(
            || "a project that has closed".to_owned(),
            |project| format!("the {} project", project.title),
        ),
        Assignment::Facility { facility } => state.facilities.get(facility).map_or_else(
            || "a facility that has been demolished".to_owned(),
            |facility| format!("the {}", facility.name),
        ),
        Assignment::TestCampaign { campaign } => state.test_campaigns.get(campaign).map_or_else(
            || "a test campaign that has closed".to_owned(),
            |campaign| {
                format!(
                    "the {} test campaign",
                    domain::research::capability_name(campaign.target_capability)
                )
            },
        ),
    }
}

/// The single most binding constraint across a team, if any.
///
/// The team is held back by its *weakest* member, which is what makes a single
/// exhausted engineer a management problem rather than a rounding error.
///
/// **The tools factor is computed once and passed in.** The panel is drawn on
/// every frame, and computing the factor per person made it quadratic in the
/// size of the agency — at three hundred people that is ninety thousand
/// operations a frame, sixty times a second.
#[must_use]
pub fn team_binding_factor(state: &CampaignState, task: TaskKind) -> Option<ModifierKind> {
    let tools = tools_modifier(state);
    let mut worst: Option<(ModifierKind, u32)> = None;
    for person in state.people.values() {
        if !person.is_assigned() {
            continue;
        }
        let result = person_performance_with_tools(tools, person, task, 1, false);
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
