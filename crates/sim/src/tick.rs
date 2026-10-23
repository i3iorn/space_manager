//! The Instant tick: the ten-phase driver.
//!
//! The phase order is normative. It is hashed into `sim_version`, so changing
//! it invalidates old replays — deliberately. See `docs/SIMULATION.md` §2.2.

use domain::chronicle::ChronicleEntry;
use domain::ids::EventId;
use domain::order::OrderKind;
use domain::report::ReportLine;
use domain::state::CampaignState;
use domain::time::Instant;

use crate::error::{OrderError, SimError};
use crate::phases;
use crate::rng::Rng;

/// Everything a phase is given to work with.
///
/// The state is a working copy: the Instant's changes are staged here and only
/// committed once every phase has run, so a Instant either happens entirely or
/// not at all.
pub struct MonthContext<'a> {
    /// The campaign, being resolved.
    pub state: &'a mut CampaignState,
    /// The campaign's random source.
    pub rng: &'a Rng,
    /// The instant being resolved.
    pub month: Instant,
    /// The orders the player issued, as given.
    pub orders: Vec<OrderKind>,
    /// The orders that survived validation. Phase 1 fills this; later
    /// phases consume it.
    pub validated: Vec<OrderKind>,
    /// The orders that were refused, and why.
    pub rejections: Vec<OrderError>,
    /// What happened to the people this month.
    pub personnel_lines: Vec<ReportLine>,
    /// What happened to the programmes this month.
    pub programme_lines: Vec<ReportLine>,
    /// Things that will become a problem if ignored.
    pub warnings: Vec<String>,
    /// Things the player could act on.
    pub opportunities: Vec<String>,
    /// The month's chronicle entries.
    pub events: Vec<ChronicleEntry>,
    /// The events that fired and want the player to answer.
    ///
    /// **An event that carries a decision does not resolve its question** — it
    /// raises it. The simulation turns these into pending decisions, and the
    /// host shows them and stops the clock.
    pub decisions: Vec<EventId>,
    /// The items the phases want on the scheduler.
    ///
    /// **A phase cannot reach the scheduler**, because the scheduler belongs
    /// to the host and not to the state. A phase that makes something fall
    /// due puts it here, and the simulation schedules it when the phase is
    /// done.
    pub schedule: Vec<(Instant, crate::scheduler::ScheduledKind)>,
}

impl<'a> MonthContext<'a> {
    /// The instant being resolved.
    ///
    /// The same value as month, under the name the continuous model will use.
    #[must_use]
    pub const fn instant(&self) -> Instant {
        self.month
    }

    /// A context for resolving one month.
    #[must_use]
    pub fn new(
        state: &'a mut CampaignState,
        rng: &'a Rng,
        month: Instant,
        orders: Vec<OrderKind>,
    ) -> Self {
        Self {
            state,
            rng,
            month,
            orders,
            validated: Vec::new(),
            rejections: Vec::new(),
            personnel_lines: Vec::new(),
            programme_lines: Vec::new(),
            warnings: Vec::new(),
            opportunities: Vec::new(),
            events: Vec::new(),
            decisions: Vec::new(),
            schedule: Vec::new(),
        }
    }
}

/// One step of the Instant.
///
/// Each phase lives in its own module, so the tick file never changes when a
/// phase's internals do. That is the extension-point property
/// `docs/SIMULATION.md` §12 demands.
pub trait Phase {
    /// The phase's name, for the log and for error messages.
    const NAME: &'static str;

    /// Runs the phase.
    ///
    /// # Errors
    ///
    /// Returns a [`SimError`] if the phase cannot complete. The state's changes
    /// are discarded, because the commit is atomic.
    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError>;
}

/// Runs a phase.
fn run_phase<P: Phase>(context: &mut MonthContext<'_>) -> Result<(), SimError> {
    P::run(context)
}

/// The names of the monthly phases, in the order they run.
///
/// The events phase is **not** here: it is a scheduled item now, and it resolves
/// last within an instant so that it reacts to the work that has just happened.
pub const PHASE_ORDER: [&str; 12] = [
    "calendar",
    "intake",
    "personnel",
    "research",
    "construction",
    "missions",
    "vehicles",
    "programmes",
    "finance",
    "politics",
    "world",
    "commit",
];

/// Resolves one month, in the normative order.
///
/// The order is:
///
/// | Phase | What it does |
/// |---|---|
/// | 0 calendar | Fires the periodic triggers |
/// | 1 intake | Validates the player's orders. Writes nothing |
/// | 2 personnel | Fatigue, morale, health, attrition, and the assignments |
/// | 3 research | Project progress, breakthroughs, capability unlocks |
/// | 4 construction | Builds, and the output of finished facilities |
/// | 5 missions | Mission phases and their outcomes |
/// | 6 programmes | The programme orders, and the commitments falling due |
/// | 7 finance | Income, outgoings, the treasury, the crisis ladder |
/// | 8 politics | Approval, promises falling due, public opinion |
/// | 9 world | Rivals, the market, milestone primacy |
/// | 10 commit | The report, and the atomic commit |
///
/// **The events phase is not here.** It is a *scheduled item* now
/// (`ScheduledKind::EventCheck`), and it resolves last within an instant so that
/// it reacts to the work that has just happened. See `docs/SIMULATION.md` §5.2.
pub fn resolve_month(context: &mut MonthContext<'_>) -> Result<(), SimError> {
    run_phase::<phases::calendar::CalendarPhase>(context)?;
    run_phase::<phases::intake::IntakePhase>(context)?;
    run_phase::<phases::personnel::PersonnelPhase>(context)?;
    run_phase::<phases::research::ResearchPhase>(context)?;
    run_phase::<phases::construction::ConstructionPhase>(context)?;
    run_phase::<phases::missions::MissionsPhase>(context)?;
    run_phase::<phases::vehicles::VehiclesPhase>(context)?;
    run_phase::<phases::programmes::ProgrammePhase>(context)?;
    run_phase::<phases::finance::FinancePhase>(context)?;
    run_phase::<phases::politics::PoliticsPhase>(context)?;
    run_phase::<phases::world::WorldPhase>(context)?;
    run_phase::<phases::events::EventsPhase>(context)?;
    run_phase::<phases::commit::CommitPhase>(context)?;
    Ok(())
}

/// The month's random events.
///
/// This is a scheduled item rather than a phase, so that events can happen at
/// their own instants rather than only at a month boundary.
pub fn resolve_events(context: &mut MonthContext<'_>) -> Result<(), SimError> {
    run_phase::<phases::events::EventsPhase>(context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_phase_order_is_the_normative_one() {
        assert_eq!(PHASE_ORDER[0], "calendar");
        assert_eq!(PHASE_ORDER[1], "intake");
        assert_eq!(PHASE_ORDER[2], "personnel");
        assert_eq!(PHASE_ORDER[6], "vehicles");
        assert_eq!(PHASE_ORDER[7], "programmes");
        assert_eq!(PHASE_ORDER[10], "world");
        assert_eq!(PHASE_ORDER[11], "commit");
        assert_eq!(PHASE_ORDER.len(), 12);
    }

    #[test]
    fn events_are_not_a_monthly_phase() {
        assert!(!PHASE_ORDER.contains(&"events"));
    }

    #[test]
    fn events_resolve_after_everything_else_in_an_instant() {
        // The events are a scheduled item, and their priority is the highest, so
        // they land after the month's work at the same instant.
        use crate::scheduler::ScheduledKind;
        assert!(ScheduledKind::EventCheck.priority() > ScheduledKind::MonthBoundary.priority());
        assert!(
            ScheduledKind::EventCheck.priority()
                > ScheduledKind::OrderLands {
                    order: domain::ids::OrderId(0)
                }
                .priority()
        );
        assert!(
            ScheduledKind::EventCheck.priority()
                > ScheduledKind::AttritionCheck {
                    person: domain::ids::PersonId(0)
                }
                .priority()
        );
    }
}
