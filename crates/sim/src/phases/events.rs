//! Phase 9 — events.
//!
//! Drawn **last**, so that events react to the state the player just created. A
//! launch that failed in phase 5 is a *different* event than a launch that
//! failed last Instant — the political response, the press reaction, and the
//! investigation all depend on the state as it stands at the end of the Instant.
//!
//! **Deferred in the MVP.** The event catalogue arrives with Slice 3
//! (`ROADMAP.md` S3-13 to S3-18). The mechanism is designed and documented in
//! `docs/SIMULATION.md` §5; this phase is the slot it will run in, and it is
//! already in the normative order.

use crate::error::SimError;
use crate::tick::{MonthContext, Phase};

/// The events phase.
pub struct EventsPhase;

impl Phase for EventsPhase {
    const NAME: &'static str = "events";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        // **The bucket for an event draw is the check's index**, which is
        // `context.state.event_checks` — a stable identifier, and the count of
        // the checks drawn so far. It is *not* the instant: the check's
        // interval tightens under stress, so the instants move and the index
        // does not, and the n-th check of a campaign always draws the same
        // numbers.
        //
        // No events are drawn yet. The phase is in the normative order so that
        // adding the catalogue does not move it.
        let _ = context;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::AgencyId;
    use domain::state::{Agency, CampaignState};
    use domain::time::Instant;

    use crate::rng::Rng;

    #[test]
    fn the_events_phase_does_nothing_yet() {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        let rng = Rng::new(23);
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        EventsPhase::run(&mut context).expect("the phase runs");
        assert!(context.events.is_empty());
    }
}
