//! The balance harness: reference policies, and the distribution of outcomes.
//!
//! **The harness is how the balance is read.** A constant in
//! [`crate::balance`] is invisible until the harness says what it does: a
//! change that makes the game unwinnable, trivial, or one-dimensional shows up
//! as a distribution and not as a number.
//!
//! # The policies
//!
//! Three reference ways of playing, and they are the three the MVP says must
//! all survive:
//!
//! | Policy | The shape of it |
//! |---|---|
//! | [`Policy::Cautious`] | Test everything, staff every team, and take the long way round |
//! | [`Policy::Fast`] | Spend the money, skip the testing, and go now |
//! | [`Policy::ScienceFirst`] | Research everything before building anything |
//!
//! **A policy is not a player.** It is a reference, and the point of having
//! three is that the game must not have one right answer. If two of the three
//! collapse, the trade-offs are decoration.
//!
//! # The determinism
//!
//! **A campaign under a policy is a campaign.** The policy reads the state and
//! nothing else, and it issues its orders through [`Sim::issue`], so the
//! order log records every decision and the campaign can be replayed. The
//! same seed and the same policy produce the same campaign, every time.

use std::collections::BTreeMap;

use domain::ids::{PersonId, VehicleId};
use domain::order::OrderKind;
use domain::person::{Assignment, ProgrammeRole, Role};
use domain::research::{CapabilityId, DisciplineId};
use domain::state::CampaignState;
use domain::time::Instant;
use domain::vehicle::{ArchetypeId, SliderValues};

use crate::Sim;
use crate::balance;
use crate::scenario::Scenario;

/// A reference way of playing.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Policy {
    /// Test first, staff every team, and take the long way round.
    Cautious,
    /// Spend the money, skip the testing, and go now.
    Fast,
    /// Research everything before building anything.
    ScienceFirst,
}

impl Policy {
    /// Every policy, in the order the harness reports them.
    pub const ALL: [Self; 3] = [Self::Cautious, Self::Fast, Self::ScienceFirst];

    /// The policy's name, as the player reads it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cautious => "Cautious",
            Self::Fast => "Fast",
            Self::ScienceFirst => "Science first",
        }
    }

    /// The policy's shape: how it staffs, tests, and designs.
    #[must_use]
    const fn profile(self) -> Profile {
        match self {
            Self::Cautious => Profile {
                project_team: balance::HARNESS_CAUTIOUS_PROJECT_TEAM,
                programme_team: balance::HARNESS_CAUTIOUS_PROGRAMME_TEAM,
                max_staff: balance::HARNESS_CAUTIOUS_MAX_STAFF,
                disciplines: &[DisciplineId(0), DisciplineId(1), DisciplineId(2)],
                tests: true,
                tested: &[CapabilityId(2)],
                sliders: SliderValues {
                    propulsion: 45,
                    payload: 30,
                    life_support: 40,
                    redundancy: 70,
                    power: 30,
                    structure: 70,
                    avionics: 40,
                    heat_shield: 50,
                },
                crew: true,
            },
            Self::Fast => Profile {
                project_team: balance::HARNESS_FAST_PROJECT_TEAM,
                programme_team: balance::HARNESS_FAST_PROGRAMME_TEAM,
                max_staff: balance::HARNESS_FAST_MAX_STAFF,
                disciplines: &[DisciplineId(0)],
                tests: false,
                tested: &[],
                sliders: SliderValues {
                    propulsion: 40,
                    payload: 20,
                    life_support: 0,
                    redundancy: 70,
                    power: 20,
                    structure: 70,
                    avionics: 0,
                    heat_shield: 0,
                },
                crew: false,
            },
            Self::ScienceFirst => Profile {
                project_team: balance::HARNESS_SCIENCE_PROJECT_TEAM,
                programme_team: balance::HARNESS_SCIENCE_PROGRAMME_TEAM,
                max_staff: balance::HARNESS_SCIENCE_MAX_STAFF,
                disciplines: &[DisciplineId(0), DisciplineId(1), DisciplineId(2)],
                tests: true,
                tested: &[CapabilityId(1), CapabilityId(2), CapabilityId(4)],
                sliders: SliderValues {
                    propulsion: 60,
                    payload: 40,
                    life_support: 50,
                    redundancy: 80,
                    power: 40,
                    structure: 80,
                    avionics: 60,
                    heat_shield: 60,
                },
                crew: true,
            },
        }
    }
}

/// The shift a reference policy puts an exhausted person on.
///
/// **Half a load is enough to recover on** and still keeps them on the team,
/// which is the whole point of the shift.
const LIGHT_SHIFT: u16 = (50 * domain::scale::PERCENT_UNITS as u32) as u16;

/// The shape of a policy: how it staffs, tests, and designs.
struct Profile {
    /// How many people a research project gets.
    project_team: usize,
    /// How many people the programme gets.
    programme_team: usize,
    /// How many people the agency employs.
    max_staff: usize,
    /// The fields of research the policy works in.
    disciplines: &'static [DisciplineId],
    /// Whether it tests what it has developed.
    tests: bool,
    /// The capabilities it tests.
    ///
    /// **A cautious agency tests the thing it is about to use**, and a
    /// thorough one tests everything it has. The list is data so that the
    /// difference between the two is visible in the report.
    tested: &'static [CapabilityId],
    /// The vehicle design it uses.
    sliders: SliderValues,
    /// Whether it flies a crew.
    crew: bool,
}

/// What became of a campaign under a policy.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// The policy that played it.
    pub policy: Policy,
    /// The campaign's seed.
    pub seed: u64,
    /// How many months it ran for.
    pub months: u32,
    /// Whether the agency reached orbit.
    pub reached_orbit: bool,
    /// When it reached orbit, if it did.
    pub months_to_orbit: Option<u32>,
    /// Whether the orbital deadline was met.
    pub deadline_met: bool,
    /// How many launches were authorised.
    pub launches: u32,
    /// How many of them worked.
    pub successes: u32,
    /// How many of them half-worked.
    pub partials: u32,
    /// How many of them failed outright.
    pub failures: u32,
    /// How many staff the agency had at the end.
    pub staff_at_end: usize,
    /// How many staff it lost along the way.
    pub staff_lost: u32,
    /// What was left in the treasury.
    pub treasury_at_end: i64,
    /// How much knowledge the agency had.
    pub knowledge: u32,
    /// How many capabilities it had developed.
    pub capabilities: usize,
    /// How many entries the chronicle holds.
    pub chronicle: usize,
    /// How many orders were given.
    pub orders_issued: u32,
    /// How many orders were refused.
    pub orders_refused: u32,
}

impl Outcome {
    /// A new, empty outcome.
    fn new(policy: Policy, seed: u64, months: u32) -> Self {
        Self {
            policy,
            seed,
            months,
            reached_orbit: false,
            months_to_orbit: None,
            deadline_met: false,
            launches: 0,
            successes: 0,
            partials: 0,
            failures: 0,
            staff_at_end: 0,
            staff_lost: 0,
            treasury_at_end: 0,
            knowledge: 0,
            capabilities: 0,
            chronicle: 0,
            orders_issued: 0,
            orders_refused: 0,
        }
    }

    /// Whether the policy got the agency to orbit.
    ///
    /// **This is the measure the strategy check is about.** A policy that
    /// never reaches orbit has collapsed, and a game with one surviving policy
    /// is a puzzle with a single answer.
    #[must_use]
    pub const fn is_viable(&self) -> bool {
        self.reached_orbit
    }
}

/// Runs a campaign under a policy.
///
/// **The campaign is played a month at a time**, because a month is the unit
/// the player thinks in and the unit the orders take to land. The orders are
/// given through [`Sim::issue`], so they are validated, scheduled, and
/// recorded in the order log like any other decision.
#[must_use]
pub fn run(scenario: &Scenario, policy: Policy, months: u32) -> Outcome {
    let mut sim = Sim::new(scenario).expect("the scenario starts");
    let mut outcome = Outcome::new(policy, scenario.seed, months);
    let mut counted: std::collections::BTreeSet<domain::ids::MissionId> =
        std::collections::BTreeSet::new();
    let mut seen_staff = sim.state().people.len();
    let mut deadline_resolved = false;

    for month in 0..months {
        for order in plan(sim.state(), policy) {
            match sim.issue(order) {
                Ok(_id) => outcome.orders_issued = outcome.orders_issued.saturating_add(1),
                Err(_reason) => outcome.orders_refused = outcome.orders_refused.saturating_add(1),
            }
        }

        sim.advance_to(Instant::from_months(u64::from(month) + 1))
            .expect("the campaign advances");

        // **The missions that have come home.** A mission is resolved when it
        // reaches its archival phase, and that is a scheduled instant which
        // may fall after the month boundary — so the harness looks at every
        // mission every month and counts the ones it has not counted yet.
        for mission in sim.state().missions.values() {
            // **A mission with no outcome yet is not counted.** It comes home
            // at a scheduled instant, and that instant may be after this
            // month boundary.
            let Some(result) = &mission.outcome else {
                continue;
            };
            if !counted.insert(mission.id) {
                continue;
            }
            outcome.launches = outcome.launches.saturating_add(1);
            if result.success {
                outcome.successes = outcome.successes.saturating_add(1);
                if !outcome.reached_orbit && is_orbital(&mission.destination) {
                    outcome.reached_orbit = true;
                    outcome.months_to_orbit = Some(month + 1);
                }
            } else if result.partial {
                outcome.partials = outcome.partials.saturating_add(1);
            } else {
                outcome.failures = outcome.failures.saturating_add(1);
            }
        }

        // The staff who have left.
        let staff = sim.state().people.len();
        if staff < seen_staff {
            outcome.staff_lost = outcome
                .staff_lost
                .saturating_add(u32::try_from(seen_staff - staff).unwrap_or(0));
        }
        seen_staff = staff;

        // The deadline, when it is resolved.
        if !deadline_resolved && deadline_has_been_resolved(sim.state()) {
            deadline_resolved = true;
            outcome.deadline_met = orbital_objective_met(sim.state());
        }
    }

    // **One month past the end**, so that a deadline falling due exactly on
    // the last instant of the campaign is resolved.
    //
    // **A boundary fires before the target**, which is the rule the golden
    // fixture depends on: the work for a month is dated at the month it
    // covers, and it happens as the campaign moves past that instant. A
    // campaign that stops exactly on a deadline has not yet had it resolved.
    sim.advance_to(Instant::from_months(u64::from(months) + 1))
        .expect("the campaign advances");
    if !deadline_resolved && deadline_has_been_resolved(sim.state()) {
        outcome.deadline_met = orbital_objective_met(sim.state());
    }

    let state = sim.state();
    outcome.staff_at_end = state.people.len();
    outcome.treasury_at_end = state.treasury.balance.credits();
    outcome.knowledge = state.knowledge.values().sum::<u32>();
    outcome.capabilities = state.unlocked.len();
    outcome.chronicle = state.chronicle.len();
    outcome
}

/// Runs a campaign under a policy, over a number of seeds.
///
/// **Every seed is a different campaign**, and the distribution over them is
/// the measurement. One campaign is an anecdote.
#[must_use]
pub fn run_seeds(scenario: &Scenario, policy: Policy, seeds: &[u64], months: u32) -> Vec<Outcome> {
    seeds
        .iter()
        .map(|seed| {
            let mut scenario = scenario.clone();
            scenario.seed = *seed;
            run(&scenario, policy, months)
        })
        .collect()
}

/// Runs every policy over a number of seeds.
#[must_use]
pub fn run_all(scenario: &Scenario, seeds: &[u64], months: u32) -> Vec<Outcome> {
    let mut outcomes = Vec::new();
    for policy in Policy::ALL {
        outcomes.extend(run_seeds(scenario, policy, seeds, months));
    }
    outcomes
}

/// Reports the distribution of a set of outcomes.
///
/// **The report is the deliverable of the balance pass.** It says how often
/// each policy reached orbit, what it cost, and what it left behind.
#[must_use]
pub fn report(outcomes: &[Outcome]) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "{} campaigns, {} months each.",
        outcomes.len(),
        outcomes.first().map_or(0, |outcome| outcome.months)
    ));
    lines.push(String::new());

    for policy in Policy::ALL {
        let group: Vec<&Outcome> = outcomes
            .iter()
            .filter(|outcome| outcome.policy == policy)
            .collect();
        if group.is_empty() {
            continue;
        }
        let count = group.len();
        let orbits = group.iter().filter(|outcome| outcome.reached_orbit).count();
        let deadlines = group.iter().filter(|outcome| outcome.deadline_met).count();
        let mut months: Vec<u32> = group
            .iter()
            .filter_map(|outcome| outcome.months_to_orbit)
            .collect();
        months.sort_unstable();
        let median = months.get(months.len() / 2).copied().unwrap_or_default();
        let earliest = months.first().copied().unwrap_or_default();
        let latest = months.last().copied().unwrap_or_default();
        let average_treasury = group
            .iter()
            .map(|outcome| outcome.treasury_at_end)
            .sum::<i64>()
            / i64::try_from(count).unwrap_or(i64::MAX);
        let average_staff = group
            .iter()
            .map(|outcome| outcome.staff_at_end)
            .sum::<usize>()
            / count;
        let average_knowledge =
            group.iter().map(|outcome| outcome.knowledge).sum::<u32>() / count as u32;
        let average_launches =
            group.iter().map(|outcome| outcome.launches).sum::<u32>() / count as u32;
        let total_launches: u32 = group.iter().map(|outcome| outcome.launches).sum();
        let total_successes: u32 = group.iter().map(|outcome| outcome.successes).sum();
        let total_partials: u32 = group.iter().map(|outcome| outcome.partials).sum();
        let total_failures: u32 = group.iter().map(|outcome| outcome.failures).sum();
        let chronicle = group.iter().map(|outcome| outcome.chronicle).sum::<usize>() / count;
        let staff_lost = group.iter().map(|outcome| outcome.staff_lost).sum::<u32>() / count as u32;

        lines.push(format!("{}:", policy.label()));
        lines.push(format!(
            "  reached orbit   {orbits}/{count}   (deadline met {deadlines}/{count})"
        ));
        lines.push(format!(
            "  to orbit        earliest {earliest}, median {median}, latest {latest} months"
        ));
        lines.push(format!(
            "  launches        {average_launches} a campaign, {total_launches} in all"
        ));
        lines.push(format!(
            "  outcomes        {total_successes} worked, {total_partials} partly, {total_failures} lost"
        ));
        lines.push(format!("  staff lost      {staff_lost} a campaign"));
        lines.push(format!("  chronicle       {chronicle} entries a campaign"));
        lines.push(format!("  treasury        {average_treasury} credits left"));
        lines.push(format!("  staff           {average_staff} at the end"));
        lines.push(format!("  knowledge       {average_knowledge} points"));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Whether every policy is still viable.
///
/// **This is the MVP's most important test.** If two of the three strategies
/// collapse, the trade-offs are fake, and the game is a puzzle with a single
/// answer.
#[must_use]
pub fn all_viable(outcomes: &[Outcome]) -> bool {
    Policy::ALL.iter().all(|policy| {
        let group: Vec<&Outcome> = outcomes
            .iter()
            .filter(|outcome| outcome.policy == *policy)
            .collect();
        if group.is_empty() {
            return false;
        }
        let orbits = group.iter().filter(|outcome| outcome.reached_orbit).count();
        // **At least half the campaigns must reach orbit.** A policy that
        // reaches orbit one time in five is not a strategy.
        orbits * 2 >= group.len()
    })
}

// ---- The policy ----

/// The orders a policy gives this month.
///
/// **This is the reference player's decision, and it is public** so that the
/// tests can drive it directly.
#[must_use]
pub fn plan_for_debug(state: &CampaignState, policy: Policy) -> Vec<OrderKind> {
    plan(state, policy)
}

/// The orders a policy gives this month.
fn plan(state: &CampaignState, policy: Policy) -> Vec<OrderKind> {
    let profile = policy.profile();
    let mut orders = Vec::new();
    let mut claimed: std::collections::BTreeSet<PersonId> = std::collections::BTreeSet::new();

    let free = free_people(state);
    let scientists: Vec<PersonId> = free
        .iter()
        .copied()
        .filter(|id| role_of(state, *id) == Role::Scientist)
        .collect();
    let engineers: Vec<PersonId> = free
        .iter()
        .copied()
        .filter(|id| role_of(state, *id) == Role::Engineer)
        .collect();

    plan_the_programme(state, &profile, &engineers, &mut claimed, &mut orders);
    plan_the_research(state, &profile, &scientists, &mut claimed, &mut orders);
    plan_the_testing(state, &profile, &scientists, &mut claimed, &mut orders);
    plan_the_vehicle(state, &profile, &mut orders);
    plan_the_launch(state, &profile, &mut orders);
    plan_the_shift_policy(state, &mut orders);
    plan_the_hiring(state, &profile, &scientists, &engineers, &mut orders);

    orders
}

/// The people with nothing to do.
fn free_people(state: &CampaignState) -> Vec<PersonId> {
    state
        .people
        .values()
        .filter(|person| !person.is_assigned())
        .map(|person| person.id)
        .collect()
}

/// A person's role.
fn role_of(state: &CampaignState, id: PersonId) -> Role {
    state
        .people
        .get(&id)
        .map_or(Role::Administrator, |person| person.role)
}

/// **The programme's team.** The engineers work on the orbital programme, and
/// the programme is what the deadline is attached to.
fn plan_the_programme(
    state: &CampaignState,
    profile: &Profile,
    engineers: &[PersonId],
    claimed: &mut std::collections::BTreeSet<PersonId>,
    orders: &mut Vec<OrderKind>,
) {
    let Some(programme) = orbital_programme(state) else {
        return;
    };
    let wanted = profile.programme_team;
    if programme.staffing.len() >= wanted {
        return;
    }
    for member in engineers.iter().take(wanted - programme.staffing.len()) {
        if !claimed.insert(*member) {
            continue;
        }
        orders.push(OrderKind::Assign {
            person: *member,
            to: Assignment::Programme {
                programme: programme.id,
                role: ProgrammeRole::Member,
            },
        });
    }
}

/// **The research.** A project in each field the policy cares about, and a
/// team on each of them.
fn plan_the_research(
    state: &CampaignState,
    profile: &Profile,
    scientists: &[PersonId],
    claimed: &mut std::collections::BTreeSet<PersonId>,
    orders: &mut Vec<OrderKind>,
) {
    for discipline in profile.disciplines {
        let running = state
            .projects
            .values()
            .find(|project| project.discipline == *discipline);
        match running {
            Some(project) => {
                let wanted = profile.project_team;
                if project.team.len() >= wanted {
                    continue;
                }
                let mut taken = 0_usize;
                for member in scientists {
                    if taken >= wanted - project.team.len() {
                        break;
                    }
                    if project.team.contains(member) || !claimed.insert(*member) {
                        continue;
                    }
                    orders.push(OrderKind::Assign {
                        person: *member,
                        to: Assignment::Project {
                            project: project.id,
                        },
                    });
                    taken += 1;
                }
            }
            None => {
                if let Some(lead) = scientists.iter().find(|member| !claimed.contains(member)) {
                    claimed.insert(*lead);
                    orders.push(OrderKind::StartProject {
                        discipline: *discipline,
                        principal_investigator: *lead,
                        title: format!("{} study", domain::research::discipline_name(*discipline)),
                    });
                }
            }
        }
    }
}

/// **The testing.** A campaign on every capability the policy has decided to
/// test, and somebody to run it.
fn plan_the_testing(
    state: &CampaignState,
    profile: &Profile,
    scientists: &[PersonId],
    claimed: &mut std::collections::BTreeSet<PersonId>,
    orders: &mut Vec<OrderKind>,
) {
    if !profile.tests {
        return;
    }
    for capability in profile.tested.iter().copied() {
        if !state.unlocked.contains(&capability) || state.tested.contains_key(&capability) {
            continue;
        }
        if state
            .test_campaigns
            .values()
            .any(|campaign| campaign.target_capability == capability)
        {
            continue;
        }
        orders.push(OrderKind::StartTestCampaign { capability });
    }
    if let Some(campaign) = state
        .test_campaigns
        .values()
        .find(|campaign| campaign.team.is_empty())
    {
        if let Some(member) = scientists.iter().find(|member| !claimed.contains(member)) {
            claimed.insert(*member);
            orders.push(OrderKind::Assign {
                person: *member,
                to: Assignment::TestCampaign {
                    campaign: campaign.id,
                },
            });
        }
    }
}

/// **The vehicle.** One, once the agency has the propulsion to build it.
fn plan_the_vehicle(state: &CampaignState, profile: &Profile, orders: &mut Vec<OrderKind>) {
    let has_vehicle = state
        .vehicles
        .values()
        .any(|vehicle| vehicle.build_state != domain::vehicle::BuildState::Scrapped);
    if has_vehicle || !state.unlocked.contains(&CapabilityId(2)) {
        return;
    }
    let unlocked: Vec<_> = state.unlocked.iter().copied().collect();
    let sliders = profile.sliders.bounded(ArchetypeId(1), &unlocked);
    orders.push(OrderKind::DesignVehicle {
        archetype: ArchetypeId(1),
        name: next_vehicle_name(state),
        sliders,
    });
}

/// **The launch.** As soon as there is something to launch with, and nothing
/// already in flight — and not at all once the programme's objective is
/// finished, because the campaign's business is done.
fn plan_the_launch(state: &CampaignState, profile: &Profile, orders: &mut Vec<OrderKind>) {
    let mission_running = state
        .missions
        .values()
        .any(|mission| mission.phase.is_running());
    let objective_done = orbital_programme(state)
        .and_then(|programme| programme.primary_objective())
        .is_some_and(|objective| objective.met == Some(true) || objective.milestone_reached);
    if mission_running || objective_done {
        return;
    }
    let Some(vehicle) = state
        .vehicles
        .values()
        .find(|vehicle| vehicle.build_state == domain::vehicle::BuildState::Complete)
    else {
        return;
    };
    let destination = "Low orbit";
    if !domain::research::is_destination_unlocked(destination, &state.unlocked) {
        return;
    }
    // **The fast policy flies unmanned**, because a cheap rocket has no life
    // support to carry anybody in.
    let crew = if profile.crew {
        state
            .people
            .values()
            .filter(|person| person.role == Role::Astronaut)
            .map(|person| person.id)
            .take(1)
            .collect()
    } else {
        Vec::new()
    };
    orders.push(OrderKind::AuthorizeLaunch {
        vehicle: vehicle.id,
        destination: destination.to_owned(),
        crew,
    });
}

/// **The shift.** An exhausted person works at a fraction of their best, and
/// the reference policies put them on light duties rather than losing them.
/// **The shift policy.** A reference player at this size manages the whole
/// agency with one standing instruction rather than thirty-odd individual
/// changes, and that is what the policy order is for.
///
/// The policy is a **thermostat**: the people who are fresher than the
/// threshold keep their own shift, and the people who are tireder are eased
/// off. The reference agency therefore never works anybody to pieces, which is
/// what the cautious strategy is for.
fn plan_the_shift_policy(state: &CampaignState, orders: &mut Vec<OrderKind>) {
    let tired = state
        .people
        .values()
        .any(|person| person.state.fatigue_percent() >= u32::from(balance::HARNESS_TIRED_FATIGUE));
    let wanted = domain::person::Policy::Shift {
        shift: LIGHT_SHIFT,
        only_over: Some(balance::HARNESS_TIRED_FATIGUE),
    };
    if tired && state.shift_policy != Some(wanted) {
        orders.push(OrderKind::SetPolicy { policy: wanted });
    }
}
fn plan_the_hiring(
    state: &CampaignState,
    profile: &Profile,
    scientists: &[PersonId],
    engineers: &[PersonId],
    orders: &mut Vec<OrderKind>,
) {
    if state.people.len() >= profile.max_staff || !can_afford_another(state) {
        return;
    }
    let role = if scientists.is_empty() {
        Role::Scientist
    } else if engineers.is_empty() {
        Role::Engineer
    } else {
        return;
    };
    orders.push(OrderKind::Hire {
        role,
        salary: balance::starting_salary(role),
    });
}

/// Whether the agency can carry another member of staff.
///
/// **A reference player does not hire into a hole.** The agency is funded in
/// proportion to its headcount, so a hire is a bet that the bigger agency
/// will be worth more than it costs — and a policy that hires regardless is
/// not a reference for anything.
fn can_afford_another(state: &CampaignState) -> bool {
    let outgoings = crate::derived::monthly_outgoings(state);
    let runway = state.treasury.runway_months();
    runway >= balance::HARNESS_MINIMUM_RUNWAY_MONTHS
        && state.treasury.balance >= outgoings.percent(50)
}

/// The orbital programme, if the agency has one.
fn orbital_programme(state: &CampaignState) -> Option<&domain::programme::Programme> {
    state
        .programmes
        .values()
        .find(|programme| programme.category == domain::programme::ProgrammeCategory::Orbital)
}

/// The name the next vehicle gets.
fn next_vehicle_name(state: &CampaignState) -> String {
    let number = state.vehicles.len() + 1;
    format!("Orbital Launcher {number}")
}

/// Whether a destination is an orbit.
fn is_orbital(destination: &str) -> bool {
    destination.eq_ignore_ascii_case("Low orbit") || destination.eq_ignore_ascii_case("Orbital")
}

/// Whether the orbital deadline has been resolved.
fn deadline_has_been_resolved(state: &CampaignState) -> bool {
    orbital_programme(state)
        .and_then(|programme| programme.primary_objective())
        .and_then(|objective| objective.met)
        .is_some()
}

/// Whether the orbital objective was met.
fn orbital_objective_met(state: &CampaignState) -> bool {
    orbital_programme(state)
        .and_then(|programme| programme.primary_objective())
        .and_then(|objective| objective.met)
        .unwrap_or(false)
}

/// The seeds the harness runs by default.
///
/// **They are a fixed list and not a range**, so that a balance run is
/// comparable with the last one: the same campaigns, every time.
#[must_use]
pub fn default_seeds(count: usize) -> Vec<u64> {
    (0..count)
        .map(|index| {
            // A simple, stable spread of seeds.
            1_000_003_u64
                .wrapping_mul(index as u64 + 1)
                .wrapping_add(12_345_678_901)
        })
        .collect()
}

/// The outcome of every campaign, keyed by seed.
///
/// **This is what a balance report is made of**, and it is a map so that two
/// runs can be compared seed by seed.
#[must_use]
pub fn by_seed(outcomes: &[Outcome]) -> BTreeMap<u64, Outcome> {
    outcomes
        .iter()
        .map(|outcome| (outcome.seed, outcome.clone()))
        .collect()
}

/// The vehicles the agency owns, by id.
#[must_use]
pub fn vehicles(state: &CampaignState) -> BTreeMap<VehicleId, String> {
    state
        .vehicles
        .values()
        .map(|vehicle| (vehicle.id, vehicle.name.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_policy_has_a_name() {
        for policy in Policy::ALL {
            assert!(!policy.label().is_empty());
        }
    }

    #[test]
    fn a_campaign_under_a_policy_runs_and_ends() {
        let scenario = Scenario::first_orbit();
        let outcome = run(&scenario, Policy::Fast, 12);
        assert_eq!(outcome.months, 12);
        assert!(outcome.chronicle > 0, "and the campaign left a chronicle");
    }

    #[test]
    fn the_same_seed_and_the_same_policy_give_the_same_campaign() {
        // **A policy is deterministic.** The harness is only a measurement if
        // the thing it measures is the same every time.
        let scenario = Scenario::first_orbit();
        let first = run(&scenario, Policy::Cautious, 12);
        let second = run(&scenario, Policy::Cautious, 12);
        assert_eq!(first.treasury_at_end, second.treasury_at_end);
        assert_eq!(first.chronicle, second.chronicle);
        assert_eq!(first.successes, second.successes);
    }

    #[test]
    fn the_report_says_something_about_every_policy() {
        let scenario = Scenario::first_orbit();
        let outcomes = run_all(&scenario, &default_seeds(2), 6);
        let report = report(&outcomes);
        for policy in Policy::ALL {
            assert!(
                report.contains(policy.label()),
                "the report names {policy:?}"
            );
        }
    }
}
