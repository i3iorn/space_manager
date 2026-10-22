//! Phase 9 — events.
//!
//! Drawn **last**, so that events react to the state the player just created. A
//! launch that failed in phase 5 is a *different* event than a launch that
//! failed last month — the political response, the press reaction, and the
//! investigation all depend on the state as it stands at the end of the month.
//!
//! **The event is drawn from the context-weighted pool**, and the weight of an
//! event is its base weight multiplied by the modifiers that apply. The design
//! rule is that **bad conditions raise the odds of bad events, and good
//! conditions raise the odds of good ones**: the player should be able to look
//! back at a disaster and say *my fault — I was running everyone at eighty
//! percent fatigue and skipped testing*.
//!
//! **A quiet stretch is a legitimate outcome.** The weights are in basis points
//! and they add up to the chance that the check produces anything at all, so a
//! roll above the total is a week in which nothing happened — and most weeks
//! are like that.

use domain::chronicle::{Category, Severity};
use domain::event::{Condition, Effect, EventDef, FollowUp};
use domain::ids::EventId;
use domain::state::CampaignState;

use crate::error::SimError;
use crate::tick::{MonthContext, Phase};

/// The events phase.
pub struct EventsPhase;

impl Phase for EventsPhase {
    const NAME: &'static str = "events";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        let _ = context;
        Ok(())
    }
}

/// Draws an event at an event check, and applies it.
///
/// **The draw is on the check's own stream**, keyed by the check's index, so
/// the n-th check of a campaign always draws the same numbers. The interval
/// tightens under stress, so the *instants* move and the *index* does not.
///
/// Returns the event that fired, if one did.
pub fn draw_and_apply(context: &mut MonthContext<'_>, check: u64) -> Option<EventDef> {
    let pool = weighted_pool(context.state, context.month);
    if pool.is_empty() {
        return None;
    }

    // **The draw is on the check's index**, which is a stable identifier.
    let mut rng = context.rng.stream(crate::rng::Domain::Events, check, 0);
    // **The pool is scaled so that its total is the era's pressure.** The
    // weights decide *which* event the pressure lands on, and not how much of it
    // there is.
    let pool = scaled_pool(&pool, context.month);
    let total = total_weight(&pool);
    if total == 0 {
        return None;
    }
    let mut roll = crate::rng::draw_basis_points(&mut rng);
    // **A quiet stretch is a legitimate outcome.** A roll above the total is a
    // week in which nothing happened, and most weeks are like that.
    if roll >= total {
        return None;
    }
    let chosen = pool
        .iter()
        .find(|(_, weight)| {
            if roll < *weight {
                true
            } else {
                roll -= *weight;
                false
            }
        })
        .map(|(event, _)| event)?;

    apply_event(context, chosen);
    Some((*chosen).clone())
}

/// The pool, scaled so that its total is the era's pressure.
///
/// **The weights are relative and the total is the pressure.** The design
/// fixes the expected number of events a check — a quarter in the early years,
/// more later — so the pool is scaled to the era's target and the modifiers
/// decide *which* event the pressure lands on.
///
/// The earlier code multiplied the raw total by the target instead of dividing
/// it, so the world was quieter than the design said by a factor of four, and
/// the individual weights were compared against a roll that had been scaled
/// differently from them. **The total and the weights have to be in the same
/// scale**, or the roll is a roll against two different rulers.
fn scaled_pool(
    pool: &[(&'static EventDef, u32)],
    now: domain::time::Instant,
) -> Vec<(&'static EventDef, u32)> {
    let raw: u64 = pool.iter().map(|(_, weight)| u64::from(*weight)).sum();
    if raw == 0 {
        return Vec::new();
    }
    let target = u64::from(era_target(now));
    pool.iter()
        .map(|(event, weight)| {
            let scaled = (u64::from(*weight) * target / raw).max(1);
            (*event, u32::try_from(scaled).unwrap_or(u32::MAX))
        })
        .collect()
}

/// The events that could fire now, and their weights.
///
/// **This is the whole of the context weighting**, and it is public so that
/// the rule it obeys can be tested.
#[must_use]
pub fn weighted_pool(
    state: &CampaignState,
    now: domain::time::Instant,
) -> Vec<(&'static EventDef, u32)> {
    crate::event_catalogue::catalogue()
        .iter()
        .filter(|event| {
            event
                .preconditions
                .iter()
                .all(|condition| holds(state, condition, now))
        })
        .map(|event| {
            let weight = event
                .weight_modifiers
                .iter()
                .filter(|modifier| holds(state, &modifier.when, now))
                .fold(event.base_weight, |weight, modifier| {
                    (u64::from(weight) * u64::from(modifier.percent) / 100) as u32
                });
            (event, weight)
        })
        .collect()
}

/// The pool's total weight.
///
/// **The total is the pressure, and it is fixed by the era.** The pool has
/// already been scaled to it, so this is the sum of the scaled weights and it
/// is the number a roll is measured against.
fn total_weight(pool: &[(&'static EventDef, u32)]) -> u32 {
    pool.iter()
        .fold(0_u32, |total, (_, weight)| total.saturating_add(*weight))
}

/// The target total weight for the era the campaign is in.
///
/// **The early years are quiet and the late years are not**, and the target is
/// in basis points of the weekly check: a quarter of a month for the early
/// era, and so on.
fn era_target(now: domain::time::Instant) -> u32 {
    let years = era_years(now);
    match years {
        0..=4 => 2_500,
        5..=19 => 3_750,
        _ => 5_000,
    }
}

/// How many years into the campaign the world is.
///
/// **The world gets busier as the agency gets bigger**, and the year is the
/// simplest measure of that.
fn era_years(now: domain::time::Instant) -> u64 {
    // The months since the campaign began.
    now.months() / 12
}
/// Whether a condition holds.
fn holds(state: &CampaignState, condition: &Condition, now: domain::time::Instant) -> bool {
    match condition {
        Condition::MonthAtLeast(instant) => now >= *instant,
        Condition::TreasuryBelow(money) => state.treasury.balance < *money,
        Condition::TreasuryAbove(money) => state.treasury.balance > *money,
        // **The approval is scaled and the threshold is not**, so the
        // comparison scales the threshold. A condition that compared a scaled
        // approval against a whole number was never true.
        Condition::ApprovalBelow(value) => {
            state.political.approval.value < *value * domain::scale::PERCENT_UNITS as i16
        }
        Condition::ApprovalAbove(value) => {
            state.political.approval.value > *value * domain::scale::PERCENT_UNITS as i16
        }
        Condition::HasProgramme(programme) => state.programmes.contains_key(programme),
        Condition::NoProgrammeNamed(name) => !state
            .programmes
            .values()
            .any(|programme| programme.title == *name),
        // **A condition about a person is about the worst one**, because the
        // catalogue cannot name a person: the people in the catalogue do not
        // exist yet. The wildcard id means *anybody*, so the condition is
        // "somebody in the agency is exhausted".
        Condition::PersonFatigueAbove(person, threshold) => {
            state
                .people
                .values()
                .any(|person| person.state.fatigue_percent() > u32::from(*threshold))
                || matches!(state.people.get(person), Some(person) if person.state.fatigue_percent() > u32::from(*threshold))
        }
        // **The threshold is a whole percentage and the condition is scaled**,
        // so the comparison scales the threshold. A threshold that was
        // compared against the raw value would never be true, and the rule
        // that a worn facility raises the odds of its bad events would be
        // dead code.
        Condition::FacilityConditionBelow(facility, threshold) => {
            let limit = u16::from(*threshold) * domain::scale::PERCENT_UNITS as u16;
            state
                .facilities
                .values()
                .any(|facility| facility.condition < limit)
                || matches!(state.facilities.get(facility), Some(facility) if facility.condition < limit)
        }
        // **The eight weighting contexts.** Every one of them corresponds to
        // something the player did, and the fairness rule is that a bad event
        // fires because of one of them.
        Condition::AverageFatigueAbove(threshold) => {
            let people = state
                .people
                .values()
                .filter(|person| person.state.fatigue > 0)
                .count();
            if people == 0 {
                return false;
            }
            let total: u64 = state
                .people
                .values()
                .map(|person| u64::from(person.state.fatigue_percent()))
                .sum();
            (total / people as u64) > u64::from(*threshold)
        }
        Condition::RunwayBelowMonths(months) => state.treasury.runway_months() < *months,
        Condition::PublicOpinionBelow(value) => state.public.opinion.value < *value,
        Condition::ProgrammeWithoutSlack => state.programmes.values().any(|programme| {
            programme.objectives.iter().any(|objective| {
                objective
                    .commitment
                    .slack()
                    .is_some_and(domain::time::Duration::is_zero)
            })
        }),
        Condition::MilestoneDueWithinMonths(months) => {
            let horizon =
                now.saturating_add(domain::time::Duration::from_months(u64::from(*months)));
            state.programmes.values().any(|programme| {
                programme
                    .milestones
                    .iter()
                    .any(|milestone| milestone.date <= horizon && !milestone.met)
            }) || state.programmes.values().any(|programme| {
                programme.objectives.iter().any(|objective| {
                    objective
                        .commitment
                        .committed
                        .is_some_and(|due| due <= horizon)
                })
            })
        }
        Condition::LaunchRiskAbove(threshold) => state
            .missions
            .values()
            .any(|mission| mission.probability / 100 < u16::from(*threshold)),
        Condition::FacilityUnstaffed => state
            .facilities
            .values()
            .any(|facility| facility.staff.is_empty()),
        Condition::HasProject(project) => state.projects.contains_key(project),
        Condition::HasCapability(capability) => state
            .unlocked
            .contains(&domain::research::CapabilityId(*capability)),
        Condition::LacksCapability(capability) => !state
            .unlocked
            .contains(&domain::research::CapabilityId(*capability)),
    }
}

/// Applies an event's effects, and writes it into the chronicle.
pub fn apply_event(context: &mut MonthContext<'_>, event: &EventDef) {
    for effect in &event.effects {
        apply_effect(context, effect);
    }

    let headline = event.narrative.headline.clone();
    let body = event.narrative.body.clone();
    let severity = event.narrative.severity;
    let category = event.narrative.category;

    // **The event is a chronicle entry and nothing else.**
    //
    // It used to be a report line as well, and that put the same headline in
    // the month report twice and in the live feed twice — once from the
    // chronicle and once from the report lines. **The chronicle is the record
    // of what happened**, and the report and the feed both read it.
    context.events.push(domain::chronicle::ChronicleEntry::new(
        context.state.next_chronicle_id(),
        context.month,
        category,
        headline,
        severity,
    ));
    if let Some(body) = body {
        context.opportunities.push(body);
    }

    // **The follow-ups are scheduled, not applied.** A follow-up fires in a
    // later month, and it appears in the forecast.
    for follow_up in &event.follow_ups {
        schedule_follow_up(context, follow_up);
    }
}

/// Adds a signed delta to a signed scaled value, clamped to the scale.
fn add_signed(value: i16, delta: i16) -> i16 {
    i32::from(value)
        .saturating_add(i32::from(delta))
        .clamp(-25_600, 25_600)
        .try_into()
        .unwrap_or(0)
}

/// Adds a signed delta to an unsigned scaled value, clamped to the scale.
fn add_unsigned(value: u16, delta: i16) -> u16 {
    i32::from(value)
        .saturating_add(i32::from(delta))
        .clamp(0, 25_600)
        .try_into()
        .unwrap_or(0)
}

/// Everybody's morale moves.
fn adjust_all_morale(context: &mut MonthContext<'_>, delta: i16) {
    for person in context.state.people.values_mut() {
        person.state.morale = add_signed(person.state.morale, delta);
    }
}

/// Everybody's loyalty moves.
fn adjust_all_loyalty(context: &mut MonthContext<'_>, delta: i16) {
    for person in context.state.people.values_mut() {
        person.state.loyalty = add_signed(person.state.loyalty, delta);
    }
}

/// Everybody's fatigue moves.
fn adjust_all_fatigue(context: &mut MonthContext<'_>, delta: i16) {
    for person in context.state.people.values_mut() {
        person.state.fatigue = add_unsigned(person.state.fatigue, delta);
    }
}

/// Everybody's health moves.
fn adjust_all_health(context: &mut MonthContext<'_>, delta: i16) {
    for person in context.state.people.values_mut() {
        person.state.health = add_unsigned(person.state.health, delta);
    }
}

/// Every facility's condition moves.
fn adjust_all_condition(context: &mut MonthContext<'_>, delta: i16) {
    for facility in context.state.facilities.values_mut() {
        facility.condition = add_unsigned(facility.condition, delta);
    }
}

/// Applies one effect.
/// Scales a delta in percentage points into the scaled units the state keeps.
///
/// **The catalogue speaks in points and the state speaks in hundredths of a
/// point.** An effect that was meant to move morale by four was moving it by
/// four hundredths of a point, and the events were doing nothing at all
/// without anybody noticing.
fn scaled(delta: i16) -> i16 {
    i16::try_from(i32::from(delta) * domain::scale::PERCENT_UNITS).unwrap_or(0)
}
fn apply_effect(context: &mut MonthContext<'_>, effect: &Effect) {
    match effect {
        Effect::AdjustMorale { person, delta } => {
            if let Some(person) = context.state.people.get_mut(person) {
                person.state.morale = add_signed(person.state.morale, scaled(*delta));
            }
        }
        Effect::AdjustLoyalty { person, delta } => {
            if let Some(person) = context.state.people.get_mut(person) {
                person.state.loyalty = add_signed(person.state.loyalty, scaled(*delta));
            }
        }
        Effect::AdjustHealth { person, delta } => {
            if let Some(person) = context.state.people.get_mut(person) {
                person.state.health = add_unsigned(person.state.health, scaled(*delta));
            }
        }
        Effect::AdjustAllMorale { delta } => adjust_all_morale(context, scaled(*delta)),
        Effect::AdjustAllLoyalty { delta } => adjust_all_loyalty(context, scaled(*delta)),
        Effect::AdjustAllFatigue { delta } => adjust_all_fatigue(context, scaled(*delta)),
        Effect::AdjustAllHealth { delta } => adjust_all_health(context, scaled(*delta)),
        Effect::AdjustAllCondition { delta } => adjust_all_condition(context, scaled(*delta)),
        Effect::AdjustTreasury { delta } => {
            context.state.treasury.balance = context
                .state
                .treasury
                .balance
                .checked_add(*delta)
                .unwrap_or(context.state.treasury.balance);
        }
        Effect::AdjustApproval { delta } => {
            context.state.political.approval.value =
                add_signed(context.state.political.approval.value, scaled(*delta));
        }
        Effect::AdjustPublicOpinion { delta } => {
            context.state.public.opinion.value =
                add_signed(context.state.public.opinion.value, scaled(*delta));
        }
        Effect::AdjustPrestige { delta } => {
            context.state.public.prestige.value =
                add_signed(context.state.public.prestige.value, scaled(*delta));
        }
        Effect::AdvanceProject { project, points } => {
            if let Some(project) = context.state.projects.get_mut(project) {
                project.progress.points = project.progress.points.saturating_add(*points);
            }
        }
        Effect::AdvanceObjective { programme, points } => {
            if let Some(programme) = context.state.programmes.get_mut(programme) {
                for objective in &mut programme.objectives {
                    objective.progress.points = objective.progress.points.saturating_add(*points);
                }
            }
        }
        Effect::AdjustFacilityCondition { facility, delta } => {
            if let Some(facility) = context.state.facilities.get_mut(facility) {
                facility.condition = add_unsigned(facility.condition, scaled(*delta));
            }
        }
        Effect::PersonLeaves { person, reason } => {
            let name = context
                .state
                .people
                .get(person)
                .map(|person| person.name.clone())
                .unwrap_or_default();
            context.state.people.remove(person);
            context
                .warnings
                .push(format!("{name} has left the agency: {reason}"));
        }
    }
}

/// Schedules a follow-up event for a later month.
fn schedule_follow_up(context: &mut MonthContext<'_>, follow_up: &FollowUp) {
    let at = context
        .month
        .saturating_add(domain::time::Duration::from_months(u64::from(
            follow_up.in_months,
        )));
    context.schedule.push((
        at,
        crate::scheduler::ScheduledKind::EventFollowUp {
            event: follow_up.event,
        },
    ));
}

/// The severity of an event, for the report.
#[must_use]
pub fn severity_of(event: &EventDef) -> Severity {
    event.narrative.severity
}

/// The category an event is filed under.
#[must_use]
pub fn category_of(event: &EventDef) -> Category {
    event.narrative.category
}

/// The identifier of an event, for the tests.
#[must_use]
pub fn id_of(event: &EventDef) -> EventId {
    event.id
}
