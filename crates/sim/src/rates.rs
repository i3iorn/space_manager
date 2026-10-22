//! The continuous rates, and the integration over elapsed time.
//!
//! Everything that used to be a monthly delta is a **rate** now, and the rates
//! are integrated over the elapsed time rather than applied once a month. A
//! person gets tired over the month rather than at the end of it, and the
//! treasury drains as the days pass rather than on the first of the month.
//!
//! # The integration grid
//!
//! The rates are applied in **quanta of one hour**, on a grid anchored at the
//! start of the campaign. The grid is what makes dilation independence hold.
//!
//! A rate is often a function of the state — a project's progress depends on
//! how tired its team is — so the result of an integration depends on how
//! finely it was done. If the host's frame rate decided that, a campaign
//! played at 1x and the same campaign played at 10,000x would integrate
//! differently and diverge in the third month, with nothing on screen to
//! explain it. **The grid is fixed, so the two campaigns integrate identically.**
//!
//! # The remainder
//!
//! Every rate application carries a **remainder** — the fraction of the
//! increment that did not fit in a whole scaled unit. Without it the
//! truncation accumulates: a rate of 0.008% an hour loses 0.133 units an hour,
//! which is 136 percentage points of fatigue over a thirty-year campaign, and
//! no test would fail. See `domain::scale::accrue`.
//!
//! # No random source
//!
//! **This module takes no random source, and that is the point.** A draw
//! inside an integration sub-step would make the campaign depend on the number
//! of sub-steps, and the number of sub-steps depends on the dilation. There is
//! no `Rng` in this module to draw from, so the type system makes the mistake
//! impossible rather than merely unlikely.
//!
//! See `docs/SIMULATION.md` §3.4.

use domain::ids::{FacilityId, PersonId, ResearchProjectId};
use domain::money::Money;
use domain::person::Assignment;
use domain::political::Goodwill;
use domain::scale::{PERCENT_UNITS, accrue};
use domain::state::CampaignState;
use domain::time::Duration;

use crate::balance;
use crate::derived::{self, TaskKind};

/// The integration quantum: the finest step the rates are applied over.
///
/// **One hour.** The grid is anchored at the start of the campaign, and every
/// integration lands on it, so the sub-steps a campaign takes are the same
/// whatever dilation it is played at.
pub const QUANTUM: Duration = Duration::from_hours(1);

/// The denominator of every rate that is expressed in scaled units a month.
///
/// It is a constant, and it must stay one: the carried remainder is only
/// meaningful against the denominator it was carried with.
const PER_MONTH: i64 = domain::time::MILLIS_PER_MONTH_I64;

/// The denominator of a project's progress rate.
///
/// A project's rate is a number of basis points, and a basis point is a
/// hundredth of a point, and a point is `PERCENT_UNITS` scaled units.
const PROGRESS_PER_BASIS_POINT: i64 = 100 * PER_MONTH;

/// Applies one quantum of every continuous rate.
///
/// Returns the research projects that finished during this quantum. The
/// completion itself is **not** done here: it draws, and a draw may only
/// happen at a scheduled instant. The caller turns each one into a scheduled
/// item.
pub fn apply_quantum(state: &mut CampaignState) -> Finished {
    // **The tools factor is the same for everybody**, and it is the only part of
    // the performance model that reads the whole state. **It is computed once
    // here and passed down**, because computing it per person per project made
    // the integration quadratic in the size of the agency: at three hundred
    // people a five-year campaign took ten seconds, and the budget is one.
    let tools = derived::tools_modifier(state);
    apply_people(state);
    let mut completed = apply_projects(state, tools);
    apply_programmes(state, tools);
    apply_facilities(state);
    let vehicles = apply_vehicles(state);
    apply_construction(state);
    apply_treasury(state);
    apply_politics(state);
    completed.shrink_to_fit();
    Finished {
        projects: completed,
        vehicles,
    }
}
/// Integrates the continuous quantities over a whole number of quanta.
///
/// The elapsed must be a whole number of quanta: the grid is what makes the
/// result independent of the host's frame rate, and a partial quantum would
/// break it.
pub fn integrate(state: &mut CampaignState, elapsed: Duration) -> Finished {
    let quanta = elapsed.millis() / QUANTUM.millis();
    let mut finished = Finished::default();
    for _ in 0..quanta {
        let step = apply_quantum(state);
        finished.projects.extend(step.projects);
        finished.vehicles.extend(step.vehicles);
    }
    finished
}

/// Moves the people's fatigue, morale, and health.
fn apply_people(state: &mut CampaignState) {
    let ids: Vec<PersonId> = state.people.keys().copied().collect();
    let goodwill = state.political.goodwill;

    for id in ids {
        // The satisfaction is read first, because it needs the whole state and
        // the person is about to be borrowed mutably.
        let satisfaction = {
            let person = &state.people[&id];
            derived::assignment_satisfaction(state, person)
        };

        // **And so is the shift**, because the policy decides it and the
        // policy lives in the state.
        let shift = {
            let person = &state.people[&id];
            derived::effective_shift(state, person)
        };
        // **And so is the policy**, because the thermostat is a step and the
        // step happens here, where the fatigue moves.
        let policy = state.shift_policy;

        let person = state.people.get_mut(&id).expect("the person exists");

        // **Fatigue.** An unfit person is on light duty, whatever their
        // assignment says, and an unassigned person rests.
        //
        // **The shift is the throttle.** A person on a full load tires at the
        // ordinary rate; a person on light duties recovers while they are
        // still on the team; and a person worked past a full load tires faster
        // than anybody should.
        let fatigue_rate = if person.state.health < balance::HEALTH_UNFIT {
            -i64::from(balance::FATIGUE_RECOVERY)
        } else {
            match person.assignment {
                Assignment::Unassigned => {
                    -i64::from(balance::FATIGUE_RECOVERY)
                        + i64::from(balance::FATIGUE_UNASSIGNED) / 2
                }
                _ => crate::shift::fatigue_rate(shift),
            }
        };
        let delta = accrue(
            fatigue_rate,
            PER_MONTH,
            QUANTUM,
            &mut person.remainders.fatigue,
        );
        person.state.fatigue = clamp_units(
            i64::from(person.state.fatigue) + delta,
            0,
            i64::from(25_600),
        ) as u16;

        // **The policy's thermostat.** The agency eases a person off when their
        // fatigue reaches the limit and keeps them eased until they have
        // recovered by a margin — so the shift does not flip on and off every
        // hour around the limit.
        crate::shift::update_easing(person, policy);

        // **Morale.** It drifts towards what the assignment makes of it, and
        // the state's goodwill is part of what the assignment makes of it: a
        // staff member who works somewhere good still cares how the country
        // treats the agency.
        //
        // **And working past a full load is miserable.** The shift is not
        // only a throttle on the body.
        let target = (i32::from(morale_target(satisfaction >= 0, &goodwill))
            - crate::shift::overtime_morale_penalty(shift) as i32)
            .clamp(-25_600, 25_600) as i16;
        let morale_rate = match person.state.morale.cmp(&target) {
            std::cmp::Ordering::Less => i64::from(balance::MORALE_RECOVERY),
            std::cmp::Ordering::Greater => -i64::from(balance::MORALE_DECAY),
            std::cmp::Ordering::Equal => 0,
        };
        let delta = accrue(
            morale_rate,
            PER_MONTH,
            QUANTUM,
            &mut person.remainders.morale,
        );
        person.state.morale =
            clamp_units(i64::from(person.state.morale) + delta, -25_600, 25_600) as i16;

        // **Health.** It recovers, slowly, and never past whole.
        let health_rate = if person.state.health < (100 * PERCENT_UNITS) as u16 {
            i64::from(balance::HEALTH_RECOVERY)
        } else {
            0
        };
        let delta = accrue(
            health_rate,
            PER_MONTH,
            QUANTUM,
            &mut person.remainders.health,
        );
        person.state.health =
            clamp_units(i64::from(person.state.health) + delta, 0, i64::from(25_600)) as u16;
    }
}

/// The morale an assignment holds, given how the country treats the agency.
///
/// **A low goodwill means the staff disengage**, and it shows in their morale
/// before it shows anywhere else.
#[must_use]
pub fn morale_target(satisfied: bool, goodwill: &Goodwill) -> i16 {
    let base = if satisfied {
        balance::MORALE_TARGET_SATISFIED
    } else {
        balance::MORALE_TARGET_UNSATISFIED
    };
    let shift =
        i32::from(goodwill.percent()) * i32::from(balance::GOODWILL_MORALE_EFFECT_PERCENT) / 100;
    let target = i32::from(base) + shift;
    i16::try_from(target).unwrap_or(base)
}

/// Moves the research projects forward.
///
/// Each team member contributes their own performance as points, and the
/// points are **summed**. A project is finished by the team, so adding people
/// to it makes it finish sooner — the target never moves.
fn apply_projects(
    state: &mut CampaignState,
    tools: domain::performance::Modifier,
) -> Vec<ResearchProjectId> {
    let mut completed = Vec::new();
    let ids: Vec<ResearchProjectId> = state.projects.keys().copied().collect();

    for id in ids {
        let team_size = derived::project_team_size(state, id);
        if team_size == 0 {
            continue;
        }

        let total_basis_points: u64 = {
            let project = &state.projects[&id];
            // **The lead is present if the lead is on the team.** A player who
            // moves the principal investigator somewhere else leaves the team
            // without its lead, and the support modifier says so - which is
            // what the binding-factor panel is for.
            let lead_present = project.team.contains(&project.principal_investigator);
            project
                .team
                .iter()
                .filter_map(|member| state.people.get(member))
                .map(|person| {
                    // **The shift scales what the person contributes.** A person
                    // on light duties contributes less; a person worked past a
                    // full load contributes more, but less than the extra
                    // effort is worth.
                    let factor =
                        crate::shift::output_factor(derived::effective_shift(state, person));
                    let effective = derived::person_performance_with_tools(
                        tools,
                        person,
                        TaskKind::Research,
                        team_size,
                        lead_present,
                    )
                    .effective;
                    u64::from(effective).saturating_mul(u64::try_from(factor).unwrap_or(0))
                        / u64::from(domain::scale::FULL_PERCENT as u32)
                })
                .sum()
        };
        if total_basis_points == 0 {
            continue;
        }

        let project = state.projects.get_mut(&id).expect("the project exists");
        if project.progress.is_complete() {
            // It is already finished; the completion is waiting for its
            // scheduled instant, and the points do not move in the meantime.
            continue;
        }

        // The effective performance is a ratio in basis points; the points are
        // a scaled quantity. Ten thousand basis points is 100% of a point.
        let numerator =
            i64::try_from(total_basis_points * u64::from(PERCENT_UNITS as u32)).unwrap_or(i64::MAX);
        let delta = accrue(
            numerator,
            PROGRESS_PER_BASIS_POINT,
            QUANTUM,
            &mut project.remainders.progress,
        );
        let gained = u32::try_from(delta.max(0)).unwrap_or(0);
        project.progress.points = project.progress.points.saturating_add(gained);

        if project.progress.is_complete() {
            completed.push(id);
        }
    }

    completed
}

/// Moves the programmes forward.
///
/// **An objective is advanced by the programme's staff**, at the rate they can
/// sustain, and the points are summed the way a project's are. The target
/// never moves: an objective needs six hundred points, whatever the programme
/// is and whoever is on it.
fn apply_programmes(state: &mut CampaignState, tools: domain::performance::Modifier) {
    let ids: Vec<domain::ids::ProgrammeId> = state.programmes.keys().copied().collect();

    for id in ids {
        let programme = &state.programmes[&id];
        if programme.objectives.is_empty() {
            continue;
        }
        let team_size = programme.staffing.len().max(1);
        let lead_present = programme.lead.is_some();

        // **The rate is the staff's performance, summed**, and it does not
        // depend on how many objectives the programme has: a programme with
        // three objectives is not a third as fast at each of them.
        let total: i64 = programme
            .staffing
            .iter()
            .filter_map(|member| state.people.get(member))
            .map(|person| {
                i64::from(
                    derived::person_performance_with_tools(
                        tools,
                        person,
                        TaskKind::Management,
                        team_size,
                        lead_present,
                    )
                    .effective,
                )
            })
            .sum();
        if total <= 0 {
            continue;
        }

        // **Whether any objective is still open.** A programme whose objectives
        // are all met or all complete has nothing to advance, and the check
        // does not need a list to make it.
        let has_open_objective = programme
            .objectives
            .iter()
            .any(|objective| objective.met.is_none() && !objective.progress.is_complete());
        if !has_open_objective {
            continue;
        }

        let programme = state.programmes.get_mut(&id).expect("the programme exists");
        for objective in &mut programme.objectives {
            if objective.met.is_some() || objective.progress.is_complete() {
                continue;
            }
            // **The points are a rate, and the remainder is carried**, so the
            // objective advances at the same rate however finely the campaign is
            // integrated.
            // The rate is the *same* rate a research project runs at: the
            // team's summed performance, in points a month. The earlier code
            // scaled the rate by the target as well, which made every
            // objective complete in about a month whatever it was worth - the
            // target cancelled itself out of the rate.
            let numerator = total.saturating_mul(i64::from(domain::scale::PERCENT_UNITS));
            let delta = accrue(
                numerator,
                PROGRESS_PER_BASIS_POINT,
                QUANTUM,
                &mut objective.remainders.progress,
            );
            let gained = u32::try_from(delta.max(0)).unwrap_or(0);
            objective.progress.points = objective.progress.points.saturating_add(gained);
        }
    }
}
/// Moves the facilities' condition.
///
/// **A facility nobody is looking after falls apart.** A fully staffed facility
/// holds its condition, and one that is short of staff loses four percent of it
/// a month. The wear is the point: the upkeep is the money and the staff are
/// the maintenance, and the agency can pay for one and not the other.
fn apply_facilities(state: &mut CampaignState) {
    let ids: Vec<domain::ids::FacilityId> = state.facilities.keys().copied().collect();
    for id in ids {
        let fully_staffed = state
            .facilities
            .get(&id)
            .is_some_and(domain::facility::Facility::is_fully_staffed);
        let operational = state
            .facilities
            .get(&id)
            .is_some_and(domain::facility::Facility::is_operational);
        if !operational {
            continue;
        }
        let rate = if fully_staffed {
            0
        } else {
            -i64::from(balance::FACILITY_DECAY_PER_MONTH)
        };
        let facility = state.facilities.get_mut(&id).expect("the facility exists");
        let delta = accrue(rate, PER_MONTH, QUANTUM, &mut facility.remainders.condition);
        facility.condition =
            clamp_units(i64::from(facility.condition) + delta, 0, i64::from(25_600)) as u16;
    }
}
/// What a run of integration finished.
///
/// **The progress crosses the target inside the integration**, where there is no
/// random source to draw from, so the completion is scheduled.
#[derive(Clone, Debug, Default)]
pub struct Finished {
    /// The projects that reached their target.
    pub projects: Vec<ResearchProjectId>,
    /// The vehicles that finished being built.
    pub vehicles: Vec<domain::ids::VehicleId>,
}

/// Moves the vehicle builds forward.
///
/// **A vehicle takes real time, and the time is its cost.** The build is a
/// rate, and the industrial capacity slows it: an agency building more than it
/// has room for builds everything more slowly.
fn apply_vehicles(state: &mut CampaignState) -> Vec<domain::ids::VehicleId> {
    let ids: Vec<domain::ids::VehicleId> = state.vehicles.keys().copied().collect();
    let mut finished = Vec::new();
    // **The industrial capacity is read once**, not once per vehicle: it is the
    // same for all of them, and the agency has a hundred vehicles one day.
    //
    // **It is read here, after the facilities have been through their decay**,
    // which is where it was read before.
    let (capacity, demand) = crate::derived::industrial_capacity(state);
    for id in ids {
        let Some(vehicle) = state.vehicles.get(&id) else {
            continue;
        };
        if vehicle.build_state != domain::vehicle::BuildState::Building {
            continue;
        }
        // **The build months are the cost**, and the cost is what the player
        // paid to have it.
        let cost = vehicle.derived.cost_estimate.credits().max(0);
        let months = ((cost * balance::VEHICLE_BUILD_MONTHS_PER_MILLION) / 1_000_000).max(1);

        // **The industrial capacity slows the build**, the same way it slows
        // the work: an over-stretched agency builds everything more slowly.
        let penalty = if capacity > 0 && demand > capacity {
            ((demand - capacity) * 5_000 / capacity).min(9_000)
        } else {
            0
        };
        let rate = months.max(1) * (10_000 - penalty) / 10_000;

        let vehicle = state.vehicles.get_mut(&id).expect("the vehicle exists");
        let target = 100 * i64::from(domain::scale::PERCENT_UNITS);
        let delta = accrue(
            target,
            rate * PER_MONTH,
            QUANTUM,
            &mut vehicle.remainders.condition,
        );
        let gained = u32::try_from(delta.max(0)).unwrap_or(0);
        vehicle.build_progress = vehicle.build_progress.saturating_add(gained);
        if vehicle.build_progress >= target as u32 {
            finished.push(id);
        }
    }
    finished
}
/// Moves the constructions forward.
fn apply_construction(state: &mut CampaignState) {
    let ids: Vec<FacilityId> = state.facilities.keys().copied().collect();

    for id in ids {
        let Some(construction) = state.facilities[&id].construction.as_ref() else {
            continue;
        };
        if construction.progress.is_complete() {
            continue;
        }
        let months = i64::from(construction.plan.build_months.max(1));
        let target = i64::from(construction.progress.target);

        let facility = state.facilities.get_mut(&id).expect("the facility exists");
        let Some(construction) = facility.construction.as_mut() else {
            continue;
        };
        // The build takes exactly as long as the plan says, and the fraction of
        // it that is left is carried so the last month is not a rounding
        // accident.
        let delta = accrue(
            target,
            months * PER_MONTH,
            QUANTUM,
            &mut construction.remainders.construction,
        );
        let gained = u32::try_from(delta.max(0)).unwrap_or(0);
        construction.progress.points = construction.progress.points.saturating_add(gained);
    }
}

/// The goodwill comes back, slowly.
///
/// **Goodwill regenerates at three percent a month, up to a cap of a hundred.**
/// A director who spends it can earn it back, but not quickly — and the cap
/// means a state that is fond of the agency does not become fond of it more.
fn apply_politics(state: &mut CampaignState) {
    let cap = i64::from(balance::GOODWILL_CAP) * i64::from(PERCENT_UNITS);
    if i64::from(state.political.goodwill.value) >= cap {
        return;
    }
    let delta = accrue(
        balance::GOODWILL_REGENERATION_PER_MONTH,
        PER_MONTH,
        QUANTUM,
        &mut state.political.remainders.goodwill,
    );
    let value = i64::from(state.political.goodwill.value)
        .saturating_add(delta)
        .min(cap);
    state.political.goodwill.value = i16::try_from(value).unwrap_or(0);
}

/// Drains the treasury at the rate the agency spends.
///
/// The income is a monthly allocation and arrives in the finance phase; only
/// the outgoings are continuous, because they are spent all the time.
fn apply_treasury(state: &mut CampaignState) {
    let outgoings = derived::monthly_outgoings(state);
    if outgoings.is_zero() {
        return;
    }

    // **The treasury's outgoings describe the burn rate right now.** The
    // interface shows them - the budget panel breaks the month down line by
    // line - and a figure that is a month out of date is a figure the player
    // cannot act on. **The detail is only rebuilt when the total changes**,
    // because the rates run every hour for the whole campaign.
    if outgoings != state.treasury.total_outgoings() {
        state.treasury.outgoings = derived::monthly_outgoings_detail(state);
    }
    // And the income, which arrives monthly but is the same every month, so
    // the panel has something to show before the first one lands.
    if state.treasury.income.is_empty() {
        state.treasury.income = derived::monthly_income(state);
    }

    let numerator = -outgoings.minor();
    let delta = accrue(
        numerator,
        PER_MONTH,
        QUANTUM,
        &mut state.treasury.remainders.money,
    );
    let balance = state.treasury.balance.minor().saturating_add(delta);
    state.treasury.balance = Money::from_minor(balance);
}
/// Clamps a signed value into a range.
fn clamp_units(value: i64, low: i64, high: i64) -> i64 {
    value.clamp(low, high)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::AgencyId;
    use domain::money::Money;
    use domain::person::{Person, Role};
    use domain::state::Agency;

    fn state_with_one_person() -> CampaignState {
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
        state
    }

    #[test]
    fn a_working_person_gets_tired_at_the_right_rate() {
        let mut state = state_with_one_person();
        state
            .people
            .get_mut(&PersonId(0))
            .expect("the person")
            .assignment = Assignment::Division(domain::ids::DivisionId(0));
        integrate(&mut state, Duration::from_months(1));
        let fatigue = state.people[&PersonId(0)].state.fatigue;
        assert_eq!(fatigue, balance::FATIGUE_ON_ASSIGNMENT);
    }

    #[test]
    fn the_rate_is_the_same_however_it_is_integrated() {
        // The same month, integrated in one go and in thirty days, must give
        // the same answer. This is the property the grid exists to protect.
        let mut whole = state_with_one_person();
        whole
            .people
            .get_mut(&PersonId(0))
            .expect("the person")
            .assignment = Assignment::Division(domain::ids::DivisionId(0));
        integrate(&mut whole, Duration::from_months(1));

        let mut piecemeal = state_with_one_person();
        piecemeal
            .people
            .get_mut(&PersonId(0))
            .expect("the person")
            .assignment = Assignment::Division(domain::ids::DivisionId(0));
        for _ in 0..30 {
            integrate(&mut piecemeal, Duration::from_days(1));
        }

        assert_eq!(
            whole.people[&PersonId(0)].state.fatigue,
            piecemeal.people[&PersonId(0)].state.fatigue
        );
    }

    #[test]
    fn the_remainder_survives_a_long_campaign() {
        // Three months of hourly integration must give exactly three months of
        // the rate, and not a rounding accident. A month of 6% is 1,536
        // units, and an hour of it is 2.133 — so the fraction has to be
        // carried for 2,160 applications or the campaign loses 287 units.
        let mut state = state_with_one_person();
        state
            .people
            .get_mut(&PersonId(0))
            .expect("the person")
            .assignment = Assignment::Division(domain::ids::DivisionId(0));
        for _ in 0..3 {
            integrate(&mut state, Duration::from_months(1));
        }
        let expected = i64::from(balance::FATIGUE_ON_ASSIGNMENT) * 3;
        assert_eq!(
            i64::from(state.people[&PersonId(0)].state.fatigue),
            expected
        );
    }

    #[test]
    fn a_resting_person_recovers() {
        let mut state = state_with_one_person();
        state
            .people
            .get_mut(&PersonId(0))
            .expect("the person")
            .state
            .fatigue = (50 * PERCENT_UNITS) as u16;
        integrate(&mut state, Duration::from_months(1));
        let fatigue = state.people[&PersonId(0)].state.fatigue;
        assert!(
            fatigue < (50 * PERCENT_UNITS) as u16,
            "an unassigned person should recover"
        );
    }

    #[test]
    fn the_treasury_drains_continuously() {
        let mut state = state_with_one_person();
        let before = state.treasury.balance.minor();
        integrate(&mut state, Duration::from_days(1));
        let after = state.treasury.balance.minor();
        assert!(after < before, "the agency spends money all the time");
    }

    #[test]
    fn the_goodwill_comes_back_at_the_right_rate() {
        // **Three percent a month**, and the fraction is carried — the same
        // remainder discipline as everything else.
        let mut state = state_with_one_person();
        integrate(&mut state, Duration::from_months(1));
        let expected = balance::GOODWILL_REGENERATION_PER_MONTH;
        assert_eq!(i64::from(state.political.goodwill.value), expected);
    }

    #[test]
    fn the_goodwill_stops_at_the_cap() {
        let mut state = state_with_one_person();
        state.political.goodwill.value = (balance::GOODWILL_CAP * PERCENT_UNITS as i16) - 1;
        integrate(&mut state, Duration::from_months(12));
        assert_eq!(
            state.political.goodwill.value,
            balance::GOODWILL_CAP * PERCENT_UNITS as i16,
            "the goodwill does not go past the cap"
        );
    }

    #[test]
    fn a_hostile_state_holds_the_morale_down() {
        // **A low goodwill means the staff disengage**, and it shows in their
        // morale before it shows anywhere else.
        let hostile = Goodwill {
            value: domain::scale::signed_percent_to_units(-100),
        };
        let friendly = Goodwill {
            value: domain::scale::signed_percent_to_units(100),
        };
        assert!(
            morale_target(true, &hostile) < morale_target(true, &friendly),
            "the staff are less content when the state is hostile"
        );
    }

    /// A state with one person and one laboratory.
    fn state_with_a_facility() -> CampaignState {
        let mut state = state_with_one_person();
        state.facilities.insert(
            domain::ids::FacilityId(0),
            domain::facility::Facility {
                id: domain::ids::FacilityId(0),
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
        state
    }

    /// The facility's condition, as a whole percentage.
    fn condition(state: &CampaignState) -> u32 {
        state.facilities[&domain::ids::FacilityId(0)].condition_percent()
    }

    #[test]
    fn a_neglected_facility_degrades() {
        // **A facility nobody is looking after falls apart.** The scenario's
        // laboratory has nobody in it, so it is wearing out.
        let mut state = state_with_a_facility();
        let before = condition(&state);
        integrate(&mut state, Duration::from_months(12));
        assert!(
            condition(&state) < before,
            "the facility wore: {before} then {}",
            condition(&state)
        );
    }

    #[test]
    fn a_staffed_facility_holds_its_condition() {
        // **Staffing a facility is what maintains it**, and the upkeep is the
        // money. A facility with both holds its condition.
        let mut state = state_with_a_facility();
        {
            let facility = state
                .facilities
                .get_mut(&domain::ids::FacilityId(0))
                .expect("the facility");
            facility.staff = vec![PersonId(0)];
            // **The state has one person**, so the laboratory needs one.
            facility.specification.staff_needed = 1;
            let person = state.people.get_mut(&PersonId(0)).expect("the person");
            person.assignment = Assignment::Facility {
                facility: domain::ids::FacilityId(0),
            };
        }
        let before = condition(&state);
        integrate(&mut state, Duration::from_months(12));
        assert_eq!(
            condition(&state),
            before,
            "a staffed facility holds its condition"
        );
    }

    #[test]
    fn a_worn_facility_produces_less() {
        // **The specification is what it was built to be; the condition is what
        // it is worth today.**
        let mut state = state_with_a_facility();
        let output = state.facilities[&domain::ids::FacilityId(0)].effective_output();
        integrate(&mut state, Duration::from_months(24));
        let after = state.facilities[&domain::ids::FacilityId(0)].effective_output();
        assert!(
            after < output,
            "a worn facility produces less: {output} then {after}"
        );
    }

    #[test]
    fn the_decay_is_the_same_however_it_is_integrated() {
        // **A facility that wears at four percent a month wears at the same
        // rate however finely the campaign is integrated.**
        let mut whole = state_with_a_facility();
        integrate(&mut whole, Duration::from_months(12));
        let mut piecemeal = state_with_a_facility();
        for _ in 0..12 {
            integrate(&mut piecemeal, Duration::from_months(1));
        }
        assert_eq!(
            condition(&piecemeal),
            condition(&whole),
            "the wear does not depend on how finely the campaign was cut up"
        );
    }

    #[test]
    fn the_decay_is_four_percent_a_month() {
        let mut state = state_with_a_facility();
        integrate(&mut state, Duration::from_months(1));
        let worn = 100 - u32::from(balance::FACILITY_DECAY_PER_MONTH / 256);
        assert_eq!(
            condition(&state),
            worn,
            "a neglected facility loses four percent a month"
        );
    }
}
