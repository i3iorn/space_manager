//! Phase 8 — the world.
//!
//! Rival agencies, the supplier market, and milestone primacy.
//!
//! **Deferred in the MVP.** The rival model is an open decision
//! (`docs/MANAGEABLE_PARTS.md` §17.1): whether rivals run the same full
//! simulation as the player, or on a lighter progression model, is an
//! architectural choice with a large performance impact, and the toggle
//! defaults to off.
//!
//! The phase exists, and it is in the normative order, so that adding rivals
//! later does not move anything.

use crate::error::SimError;
use crate::tick::{MonthContext, Phase};

/// The world phase.
pub struct WorldPhase;

impl Phase for WorldPhase {
    const NAME: &'static str = "world";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        // Nothing happens in the world yet. The phase is here so that the
        // resolution order is settled and adding rivals does not move anything.
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
    fn the_world_phase_does_nothing_yet() {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        let rng = Rng::new(19);
        let before = state.clone();
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        WorldPhase::run(&mut context).expect("the phase runs");
        assert_eq!(context.state.clone(), before);
    }
}
