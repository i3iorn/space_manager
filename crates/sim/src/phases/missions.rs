//! Phase 5 — missions.
//!
//! Mission phase transitions, success and partial failure, results, and losses.
//!
//! Missions are state machines ticked here. Success is resolved at the outcome
//! phase, not at authorisation, and the draw happens on the mission's own
//! stream — so adding a random draw elsewhere in the simulation cannot change
//! a launch outcome.
//!
//! **The phases have real durations, and the phase changes are scheduled
//! items.** A mission that takes eleven weeks to come home takes eleven weeks,
//! and the campaign keeps running while it does.

use domain::ids::MissionId;
use domain::mission::{MissionPhase, MissionResult, Outcome};
use domain::order::OrderKind;

use crate::error::SimError;
use crate::tick::{MonthContext, Phase};

/// The missions phase.
pub struct MissionsPhase;

impl Phase for MissionsPhase {
    const NAME: &'static str = "missions";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        authorise_launches(context);
        Ok(())
    }
}

/// Turns the player's launch authorisations into missions.
fn authorise_launches(context: &mut MonthContext<'_>) {
    let orders = std::mem::take(&mut context.validated);

    for order in orders {
        if !apply_one(context, &order) {
            context.validated.push(order);
        }
    }
}

/// Applies one mission order, and reports whether it was one.
///
/// **The handlers are per-kind [D].** An order lands at its own instant, and
/// the phase that owns it is the one that applies it.
pub fn apply_one(context: &mut MonthContext<'_>, order: &OrderKind) -> bool {
    match order {
        OrderKind::AuthorizeLaunch {
            vehicle,
            destination,
            crew,
        } => {
            let id = context.state.id_counters.next_mission();
            // **The chance is the vehicle's reliability, less what the testing
            // has taken out of the destination's capability.**
            let probability =
                crate::derived::mission_probability_for(context.state, *vehicle, destination);
            let mission = domain::mission::Mission::new(
                id,
                *vehicle,
                destination.clone(),
                crew.clone(),
                context.month,
                context
                    .month
                    .saturating_add(domain::time::Duration::from_months(1)),
                None,
            );
            let mut mission = mission;
            mission.probability = probability;
            context.state.missions.insert(id, mission);

            // **The first phase change is scheduled now**, and every phase
            // schedules the next. The mission is a chain of scheduled items and
            // not a monthly step.
            schedule_phase(
                context,
                id,
                MissionPhase::Launch,
                MissionPhase::Preparation.duration(),
            );

            context.programme_lines.push(
                domain::report::ReportLine::new(format!(
                    "A mission to {destination} is authorised, at {probability} per cent."
                ))
                .with_severity(domain::chronicle::Severity::Minor),
            );
            true
        }
        _ => false,
    }
}

/// Schedules a mission's next phase.
///
/// **The duration is the one of the phase being left**, because the mission
/// spends that long in the phase it is in before it moves on.
fn schedule_phase(
    context: &mut MonthContext<'_>,
    mission: MissionId,
    phase: MissionPhase,
    duration: domain::time::Duration,
) {
    let at = context.month.saturating_add(duration);
    context.schedule.push((
        at,
        crate::scheduler::ScheduledKind::MissionPhase { mission, phase },
    ));
}

/// Moves a mission into its next phase, and resolves it when it ends.
pub fn enter_phase(context: &mut MonthContext<'_>, mission: MissionId, phase: &MissionPhase) {
    // **The outcome is resolved when the mission comes home**, and it is
    // resolved at a scheduled instant, so the draw is not inside the
    // integration.
    let finished = *phase == MissionPhase::Archival;
    if let Some(mission) = context.state.missions.get_mut(&mission) {
        mission.phase = phase.clone();
    }
    if finished {
        resolve_outcome(context, mission);
        return;
    }
    if let Some(next) = phase.next() {
        schedule_phase(context, mission, next, phase.duration());
    }
}

/// Resolves what became of a mission.
///
/// **Partial failure is the common branch.** A mission that goes wrong usually
/// goes partly wrong: the capsule comes back, the data is partial, and the
/// agency learns something and loses something. A total loss is the rare case,
/// and it is rare on purpose — the game would be a punishment otherwise.
pub fn resolve_outcome(context: &mut MonthContext<'_>, mission: MissionId) {
    let Some(mission_state) = context.state.missions.get(&mission) else {
        return;
    };
    let probability = u32::from(mission_state.probability);
    let vehicle = mission_state.vehicle;
    let destination = mission_state.destination.clone();
    let severity = context
        .state
        .vehicles
        .get(&vehicle)
        .map_or(50, |vehicle| vehicle.derived.failure_severity);

    // **The draw is on the mission's own stream**, keyed by the mission's id
    // and the month's index, so it is the same roll every time the campaign is
    // replayed.
    let mut rng = context.rng.stream(
        crate::rng::Domain::Mission,
        context.month.months(),
        mission.0,
    );
    let roll = crate::rng::draw_basis_points(&mut rng);
    let outcome = draw_the_outcome(roll, probability, severity, &destination);

    // **The result feeds the research**, and the finding is the point of the
    // whole thing.
    let findings = outcome.results.clone();
    let success = outcome.success;
    let partial = outcome.partial;
    if let Some(mission) = context.state.missions.get_mut(&mission) {
        mission.outcome = Some(outcome);
        if !success && !partial {
            mission.phase = MissionPhase::Failed(Box::new(MissionPhase::Operations));
        }
    }

    // **A total failure writes the vehicle off.**
    //
    // A mission that does not come home does not come back in a condition
    // worth flying again, and a launch that has been lost is one of the three
    // things the Slice 3 exit test says a playtester has to be able to read
    // in the chronicle.
    if !success && !partial {
        if let Some(vehicle) = context.state.vehicles.get_mut(&vehicle) {
            vehicle.build_state = domain::vehicle::BuildState::Scrapped;
            let name = vehicle.name.clone();
            context.programme_lines.push(
                domain::report::ReportLine::new(format!("{name} was lost."))
                    .with_severity(domain::chronicle::Severity::Critical),
            );
            context.events.push(domain::chronicle::ChronicleEntry::new(
                context.state.next_chronicle_id(),
                context.month,
                domain::chronicle::Category::Technical,
                format!("{name} was lost on the mission to {destination}."),
                domain::chronicle::Severity::Critical,
            ));
        }
    }

    // **A successful launch finishes the programme that was about it.**
    //
    // The orbital programme's objective is "reach orbit", and the only thing
    // that reaches orbit is a launch that works. The programme's staff advance
    // the objective's progress — that is how the player sees the work going on
    // — but the objective is not finished until the thing it names has
    // happened, and **a partial failure is not a thing that has happened**.
    if success {
        complete_the_milestone(context, &destination);
    }

    for result in &findings {
        feed_the_research(context, &destination, result);
    }

    // **The state's approval moves with the result**, and the state is pleased
    // with a success and put out by a failure.
    let approval = if success {
        balance::APPROVAL_LAUNCH_SUCCEEDS
    } else if partial {
        balance::APPROVAL_LAUNCH_PARTIAL
    } else {
        balance::APPROVAL_LAUNCH_FAILS
    };
    context.state.political.approval.value = context
        .state
        .political
        .approval
        .value
        .saturating_add(approval);
    if success {
        context.state.political.goodwill = context
            .state
            .political
            .goodwill
            .granted(crate::balance::GOODWILL_MISSION_SUCCESS);
    }

    let message = if success {
        format!("The mission to {destination} succeeded.")
    } else if partial {
        format!("The mission to {destination} partly succeeded.")
    } else {
        format!("The mission to {destination} failed.")
    };
    context.programme_lines.push(
        domain::report::ReportLine::new(message.clone())
            .with_severity(domain::chronicle::Severity::Major),
    );
    context.events.push(domain::chronicle::ChronicleEntry::new(
        context.state.next_chronicle_id(),
        context.month,
        domain::chronicle::Category::Technical,
        message,
        domain::chronicle::Severity::Major,
    ));
}

/// Draws the outcome of a mission.
///
/// **Partial failure is the common branch.** Two failures in three are a
/// degraded mission that comes home with some of its data; one in three is a
/// total loss.
///
/// The earlier code made the partial band a flat five thousand basis points
/// wide, which meant that a reliable vehicle almost never had a partial
/// failure and a shaky one had almost nothing but total failures. **The share
/// of the failure is the thing that is fixed, and the size of the failure
/// follows from the vehicle.**
fn draw_the_outcome(roll: u32, probability: u32, severity: u8, destination: &str) -> Outcome {
    if roll < probability {
        return Outcome::success(vec![MissionResult {
            finding: format!("The data from {destination} came back complete."),
            quality: 100,
        }]);
    }
    let failures = 10_000_u32.saturating_sub(probability);
    let partial_band = failures * crate::balance::MISSION_PARTIAL_SHARE_BP / 10_000;
    let partial = roll < probability.saturating_add(partial_band);
    let quality = if partial {
        u8::try_from(100 - u32::from(severity) / 2).unwrap_or(50)
    } else {
        0
    };
    let results = if quality > 0 {
        vec![MissionResult {
            finding: format!("Some of the data from {destination} came back."),
            quality,
        }]
    } else {
        Vec::new()
    };
    if partial {
        Outcome::partial(severity, results)
    } else {
        Outcome {
            success: false,
            severity,
            partial: false,
            results,
        }
    }
}

/// Finishes the objective that a successful launch to a destination completes.
///
/// **The objective's progress is set to its target and the milestone is marked
/// as reached.** The commitment is not resolved here: it is resolved when it
/// falls due, against the work, and a deadline that is met early is still met.
fn complete_the_milestone(context: &mut MonthContext<'_>, destination: &str) {
    let programmes: Vec<_> = context
        .state
        .programmes
        .values()
        .filter(|programme| programme.category.milestone_destination() == Some(destination))
        .map(|programme| programme.id)
        .collect();

    for programme in programmes {
        let Some(programme) = context.state.programmes.get_mut(&programme) else {
            continue;
        };
        let Some(objective) = programme.objectives.iter_mut().find(|objective| {
            objective.kind == domain::programme::ObjectiveKind::Primary
                && !objective.milestone_reached
                && objective.met.is_none()
                && matches!(
                    &objective.completion,
                    domain::programme::ObjectiveCompletion::Destination(wanted)
                        if wanted == destination
                )
        }) else {
            continue;
        };
        objective.milestone_reached = true;
        objective.progress.points = objective.progress.target;
        let description = objective.description.clone();
        let title = programme.title.clone();
        context.programme_lines.push(
            domain::report::ReportLine::new(format!("{description} is complete."))
                .with_severity(domain::chronicle::Severity::Major),
        );
        context.events.push(domain::chronicle::ChronicleEntry::new(
            context.state.next_chronicle_id(),
            context.month,
            domain::chronicle::Category::Technical,
            format!("{description} is complete: the {title} programme reached {destination}."),
            domain::chronicle::Severity::Major,
        ));
    }
}

/// The findings of a mission advance the research in the matching discipline.
///
/// **A mission's destination names a capability, and a capability has a
/// discipline** — so the data comes back to the field that asked for it.
fn feed_the_research(context: &mut MonthContext<'_>, destination: &str, result: &MissionResult) {
    let Some(discipline) = domain::research::discipline_for_destination(destination) else {
        return;
    };
    let points = u32::from(result.quality) * 10;
    let project_ids: Vec<_> = context
        .state
        .projects
        .values()
        .filter(|project| project.discipline == discipline)
        .map(|project| project.id)
        .collect();
    for project in project_ids {
        let Some(project) = context.state.projects.get_mut(&project) else {
            continue;
        };
        project.progress.points = project.progress.points.saturating_add(points);
    }
}

/// The balance's approval and goodwill figures, in one place.
use crate::balance;
