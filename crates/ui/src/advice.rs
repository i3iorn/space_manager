//! What to do next.
//!
//! **The advice is a situation, not a script.** A tutorial says "now do this",
//! and the player who came here to do something else has to ignore it. This
//! module says nothing of the sort: it looks at the campaign as it stands and
//! finds the thing that is *wrong* or *missing* right now, and it says what to
//! do about it in one sentence.
//!
//! **The advice is read from the campaign, and it is never stored.** The banner
//! is a view of the campaign, so a player who pauses the clock and looks again
//! sees the same advice, and a player whose campaign has changed sees the
//! advice that matches the new campaign.
//!
//! **The advice never names an identifier.** It names people, vehicles and
//! places, because a player can act on a name and cannot act on a number.
//!
//! See `docs/UI.md` §4.3.

use sim::Sim;

/// One piece of advice, and the situation that makes it true.
///
/// **The struct is not `PartialEq`**, because comparing two function pointers
/// compares their addresses and says nothing about whether the situations are
/// the same one.
#[derive(Clone, Copy, Debug)]
pub struct Advice {
    /// How urgent this is.
    ///
    /// **The most urgent advice is the one that is shown.** A player has one
    /// attention, and the advice that competes for it is the advice about the
    /// thing that is actually costing them the campaign.
    pub priority: u8,
    /// The situation: what has to be true, and what to say about it.
    ///
    /// **`None` means the situation does not hold**, and the advice is not
    /// offered at all.
    pub situation: fn(&Sim) -> Option<String>,
}

/// Every situation the game knows how to talk about.
///
/// **The order is the order of urgency, and the order of the catalogue is the
/// tie-break.** A situation that has never been true does not take up a slot.
pub const ALL: [Advice; 17] = [
    Advice {
        priority: 112,
        situation: the_state_has_cut_the_funding,
    },
    Advice {
        priority: 110,
        situation: orbit_needs_the_research,
    },
    Advice {
        priority: 105,
        situation: a_programme_has_nobody_on_it,
    },
    Advice {
        priority: 100,
        situation: the_orbital_programme_needs_a_rocket,
    },
    Advice {
        priority: 95,
        situation: the_agency_is_going_broke,
    },
    Advice {
        priority: 90,
        situation: a_deadline_is_close,
    },
    Advice {
        priority: 85,
        situation: the_rocket_is_ready_but_nobody_has_launched,
    },
    Advice {
        priority: 80,
        situation: a_vehicle_is_being_built,
    },
    Advice {
        priority: 75,
        situation: somebody_is_exhausted,
    },
    Advice {
        priority: 70,
        situation: a_rocket_was_lost,
    },
    Advice {
        priority: 65,
        situation: a_project_is_understaffed,
    },
    Advice {
        priority: 60,
        situation: somebody_has_nothing_to_do,
    },
    Advice {
        priority: 55,
        situation: a_launch_is_being_prepared,
    },
    Advice {
        priority: 50,
        situation: a_project_is_almost_finished,
    },
    Advice {
        priority: 40,
        situation: the_clock_is_running,
    },
    Advice {
        priority: 30,
        situation: nobody_has_ordered_anything,
    },
    Advice {
        priority: 20,
        situation: the_staff_screen_explains_the_binding_factor,
    },
];

/// The advice that applies to the campaign, most urgent first.
///
/// **The sort is stable**, so two situations of the same urgency keep the
/// order of the catalogue, and the advice does not shuffle itself every frame.
#[must_use]
pub fn applicable(sim: &Sim) -> Vec<Advice> {
    let mut advice: Vec<Advice> = ALL
        .iter()
        .copied()
        .filter(|advice| (advice.situation)(sim).is_some())
        .collect();
    advice.sort_by_key(|advice| std::cmp::Reverse(advice.priority));
    advice
}

/// The advice to show, at a position in the cycle.
///
/// **The cycle wraps**, so a player who keeps asking for another tip gets
/// every tip there is and then comes back to the first one.
#[must_use]
pub fn at(sim: &Sim, index: usize) -> Option<Advice> {
    let advice = applicable(sim);
    if advice.is_empty() {
        return None;
    }
    Some(advice[index % advice.len()])
}

/// What the advice says, at a position in the cycle.
#[must_use]
pub fn text_at(sim: &Sim, index: usize) -> Option<String> {
    let advice = at(sim, index)?;
    (advice.situation)(sim)
}

/// How many pieces of advice there are for the campaign as it stands.
#[must_use]
pub fn count(sim: &Sim) -> usize {
    applicable(sim).len()
}

// --- The situations -------------------------------------------------------

/// The state has cut the agency's funding.
///
/// **A cut is not a mood: it is a fifth of the money gone every month, and it
/// makes the hole worse.** The advice says how much and what would lift it,
/// because the player's only move is to find the money somewhere else, and a
/// player who does not know the size of the hole cannot find it.
fn the_state_has_cut_the_funding(sim: &Sim) -> Option<String> {
    if !sim.state().treasury.crisis_stage.in_austerity() {
        return None;
    }
    Some(format!(
        "The state has cut the agency's funding by {}%. Find a month of runway again and the cut lifts.",
        sim::balance::AUSTERITY_CUT_PERCENT
    ))
}

/// The agency cannot reach orbit yet, because it has not done the research.
///
/// **This is the real blocker and nothing on the screen says so.** The launch
/// screen's list of destinations simply does not have "Low orbit" in it, and
/// the vehicle's propulsion slider is pinned at zero — so a player who has
/// never played the game cannot tell that the answer is *research*, and not a
/// rocket.
///
/// **The design says the agency has to research its way to orbit**, and the
/// interface has to say so, or the first hour of the game is a guessing game.
fn orbit_needs_the_research(sim: &Sim) -> Option<String> {
    let state = sim.state();
    let objective = orbital_objective(state)?;
    let domain::programme::ObjectiveCompletion::Destination(destination) = &objective.completion
    else {
        return None;
    };
    let capability = capability_for_destination(destination)?;
    if state.unlocked.contains(&capability.id) {
        return None;
    }
    let points = state
        .knowledge
        .get(&capability.discipline)
        .copied()
        .unwrap_or(0);
    let short = capability.threshold.saturating_sub(points);
    // **The advice names the work, because "put more people on the propulsion
    // project" is only useful if the agency has one.**
    let action = state
        .projects
        .values()
        .find(|project| project.discipline == capability.discipline)
        .map_or_else(
            || "start a project in that field".to_owned(),
            |project| format!("put more people on \"{}\"", project.title),
        );
    Some(format!(
        "Reaching orbit needs {}, and the agency is {short} points of research short of it. \
         Press 4, and {action}.",
        capability.name
    ))
}

/// A programme has nobody on it.
///
/// **An unstaffed programme is a promise nobody is working on**, and the player
/// has to know it before the commitment comes due.
fn a_programme_has_nobody_on_it(sim: &Sim) -> Option<String> {
    let programme = sim
        .state()
        .programmes
        .values()
        .find(|programme| programme.staffing.is_empty())?;
    Some(format!(
        "Nobody is on the {} programme. Press 3, then o, to put somebody on it.",
        programme.title
    ))
}

/// The orbital programme cannot be met without a rocket, and there is none.
///
/// **This is the first thing a new player has to work out**, and the design's
/// own note says so: reaching orbit takes research, a vehicle and a launch.
fn the_orbital_programme_needs_a_rocket(sim: &Sim) -> Option<String> {
    let state = sim.state();
    if !state.vehicles.is_empty() {
        return None;
    }
    let objective = orbital_objective(state)?;
    let domain::programme::ObjectiveCompletion::Destination(destination) = &objective.completion
    else {
        return None;
    };
    // **The agency cannot design a launch rocket until it has the capability**,
    // and the research advice is the one that says so.
    if capability_for_destination(destination)
        .is_some_and(|capability| !state.unlocked.contains(&capability.id))
    {
        return None;
    }
    Some(
        "Reach orbit needs a rocket, and the agency has not designed one. \
         Press 5 for the vehicles screen, then o to design one."
            .to_owned(),
    )
}

/// The agency is spending more than it has.
///
/// **A player who does not know this now finds out in six months**, and by
/// then the hole is a crater.
fn the_agency_is_going_broke(sim: &Sim) -> Option<String> {
    let runway = sim.state().treasury.runway_months();
    if runway > 6 {
        return None;
    }
    Some(format!(
        "The treasury has {runway} months left. Raise more money, or cut something."
    ))
}

/// A promise is close and the work is not done.
///
/// **The committed instant is the one that matters**, and the player has to
/// know how long they have.
fn a_deadline_is_close(sim: &Sim) -> Option<String> {
    let now = sim.effective_now();
    let mut closest: Option<(u64, String)> = None;
    for programme in sim.state().programmes.values() {
        for objective in &programme.objectives {
            if objective.met.is_some() {
                continue;
            }
            let Some(due) = objective.commitment.due() else {
                continue;
            };
            let months = due.duration_since(now).whole_months();
            if months > 9 {
                continue;
            }
            let description = objective.description.clone();
            let closer = closest.as_ref().is_none_or(|(best, _)| months < *best);
            if closer {
                closest = Some((months, description));
            }
        }
    }
    let (months, description) = closest?;
    Some(format!(
        "\"{description}\" is committed in {months} months, and it is not done yet."
    ))
}

/// A rocket is ready and nobody has launched it.
///
/// **A finished rocket sitting on the pad is money that is not doing
/// anything.**
fn the_rocket_is_ready_but_nobody_has_launched(sim: &Sim) -> Option<String> {
    let state = sim.state();
    let ready = state
        .vehicles
        .values()
        .find(|vehicle| vehicle.build_state == domain::vehicle::BuildState::Complete)?;
    if state
        .missions
        .values()
        .any(|mission| mission.vehicle == ready.id)
    {
        return None;
    }
    Some(format!(
        "{} is ready to fly. Press o and choose Authorise a launch.",
        ready.name
    ))
}

/// A vehicle is being built.
///
/// **The build takes months, and the player has to know that**, because the
/// answer to "why is nothing happening" is usually "because you ordered it
/// last month".
fn a_vehicle_is_being_built(sim: &Sim) -> Option<String> {
    let building = sim
        .state()
        .vehicles
        .values()
        .find(|vehicle| vehicle.build_state == domain::vehicle::BuildState::Building)?;
    let percent = domain::scale::units_to_percent(building.build_progress);
    Some(format!(
        "{} is being built - {percent}% done. Nothing can launch until it is ready.",
        building.name
    ))
}

/// Somebody is worn out.
///
/// **Fatigue is the most expensive factor in the performance model**, and the
/// cheapest way to fix it is the shift.
fn somebody_is_exhausted(sim: &Sim) -> Option<String> {
    let person = sim
        .state()
        .people
        .values()
        .filter(|person| person.is_assigned())
        .max_by_key(|person| person.state.fatigue_percent())?;
    let fatigue = person.state.fatigue_percent();
    if fatigue < 70 {
        return None;
    }
    Some(format!(
        "{} is at {fatigue}% fatigue. Press 2, then the left arrow, to put them on light duties.",
        person.name
    ))
}

/// A rocket was lost.
///
/// **A lost rocket is a setback and not the end**, and the player has to know
/// that the answer is another one.
fn a_rocket_was_lost(sim: &Sim) -> Option<String> {
    let lost = sim.state().missions.values().any(|mission| {
        mission
            .outcome
            .as_ref()
            .is_some_and(|outcome| !outcome.success && !outcome.partial)
    });
    if !lost {
        return None;
    }
    Some(
        "A launch was lost. Reach orbit takes another rocket, and another launch order.".to_owned(),
    )
}

/// A research project has too few people on it.
///
/// **Points are summed, so a bigger team finishes sooner and the target never
/// moves.** A project with one person takes twice as long as one with two.
///
/// **The advice names somebody**, because "put somebody else on it" is a
/// sentence the player has to act out, and a name is a sentence they can.
fn a_project_is_understaffed(sim: &Sim) -> Option<String> {
    let state = sim.state();
    let project = state
        .projects
        .values()
        .find(|project| project.team.len() == 1)?;
    // **Somebody who is free**, if there is one — the assignment moves a
    // person, so anybody will do, and a free one costs the agency nothing.
    let free = state
        .people
        .values()
        .find(|person| !person.is_assigned())
        .map(|person| person.name.clone());
    let instruction = match free {
        Some(name) => format!("put {name} on it"),
        None => "put somebody else on it".to_owned(),
    };
    Some(format!(
        "\"{}\" has one person on it. Press 4, then o, and {instruction}.",
        project.title
    ))
}

/// Somebody has nothing to do.
///
/// **An unassigned person is a cost and not a neutral state**, and the agency
/// is paying their salary.
fn somebody_has_nothing_to_do(sim: &Sim) -> Option<String> {
    let person = sim
        .state()
        .people
        .values()
        .find(|person| !person.is_assigned())?;
    Some(format!(
        "{} has nothing to do. Press 2, then o, to put them on the work.",
        person.name
    ))
}

/// A launch is being prepared.
///
/// **The player cannot hurry a launch**, and the answer to "why is nothing
/// happening" is usually a rocket on the pad.
fn a_launch_is_being_prepared(sim: &Sim) -> Option<String> {
    let mission = sim
        .state()
        .missions
        .values()
        .find(|mission| mission.phase.is_running())?;
    Some(format!(
        "The launch to {} is under way. It cannot be hurried.",
        mission.destination
    ))
}

/// A project is nearly finished.
///
/// **The player wants to know whether the work they are paying for is
/// working**, and this is the answer.
fn a_project_is_almost_finished(sim: &Sim) -> Option<String> {
    let project = sim
        .state()
        .projects
        .values()
        .find(|project| project.progress.percent() >= 80)?;
    Some(format!(
        "\"{}\" is {}% done. Its result is what the orbital programme is waiting on.",
        project.title,
        project.progress.percent()
    ))
}

/// The clock is running and the player may not know it.
///
/// **Time passes whether or not the player is ready**, and a player who has
/// just opened the game does not necessarily know that.
fn the_clock_is_running(sim: &Sim) -> Option<String> {
    if sim.state().chronicle.len() > 3 {
        return None;
    }
    Some(
        "The clock is running: a minute of game time passes every second. \
         Press Space to stop and think."
            .to_owned(),
    )
}

/// Nothing has been ordered yet.
///
/// **The order palette is how the player acts**, and a player who has not
/// found it is a player who is not playing.
fn nobody_has_ordered_anything(sim: &Sim) -> Option<String> {
    let state = sim.state();
    if !state.pending_orders.is_empty() {
        return None;
    }
    if !state.chronicle.is_empty() {
        return None;
    }
    Some("Press o to give an order. You will see what it costs before you commit to it.".to_owned())
}

/// The staff screen has something the player needs to understand.
///
/// **The binding factor is the one panel the player has to understand to
/// understand the game**, and it is on the staff screen.
fn the_staff_screen_explains_the_binding_factor(sim: &Sim) -> Option<String> {
    if sim.state().people.is_empty() {
        return None;
    }
    Some(
        "Press 2 for the staff screen. The panel under the person names the thing holding \
         them back, and it is not always the same thing."
            .to_owned(),
    )
}

// --- The helpers ----------------------------------------------------------

/// The primary objective of the orbital programme, if it has one and it is
/// not yet met.
///
/// **This is the objective the whole first act of the game is about**, and it
/// is the one whose completion condition names a launch.
fn orbital_objective(
    state: &domain::state::CampaignState,
) -> Option<&domain::programme::Objective> {
    state.programmes.values().find_map(|programme| {
        programme.objectives.iter().find(|objective| {
            objective.kind == domain::programme::ObjectiveKind::Primary
                && objective.met.is_none()
                && matches!(
                    objective.completion,
                    domain::programme::ObjectiveCompletion::Destination(_)
                )
        })
    })
}

/// The capability that unlocks a destination.
///
/// **A destination is not a place, it is a capability the agency has
/// developed.** "Low orbit" is orbital propulsion, and an agency without
/// orbital propulsion cannot fly there however good its rocket is.
fn capability_for_destination(destination: &str) -> Option<&domain::research::Capability> {
    domain::research::CAPABILITIES.iter().find(|capability| {
        capability.unlocks.iter().any(|unlock| {
            matches!(unlock, domain::research::Unlock::Destination { destination: name } if *name == destination)
        })
    })
}
