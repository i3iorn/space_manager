//! Phase 0 — calendar advance.
//!
//! Fires the periodic triggers: the payroll period, contract anniversaries, the
//! fiscal year boundary, promotion rounds, and political windows.
//!
//! **The clock itself belongs to the simulation [D], not to the state.** The
//! instant is projected onto a civil calendar here, and the simulation advances
//! it. See `docs/SIMULATION.md` §2.2 and §3.2.

use domain::time::{Duration, Instant, MILLIS_PER_MONTH};

use crate::error::SimError;
use crate::tick::{MonthContext, Phase};

/// The calendar phase.
pub struct CalendarPhase;

impl Phase for CalendarPhase {
    const NAME: &'static str = "calendar";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        // The instant being resolved is the one the simulation is at.
        debug_assert_eq!(context.month, context.instant());

        let months = context.month.millis() / MILLIS_PER_MONTH;

        // The fiscal year turns over on the anniversary of the campaign's
        // beginning, so a campaign that starts in March is renewed in March.
        if months > 0 && months % 12 == 0 {
            context.opportunities.push(format!(
                "The fiscal year {} has begun. The state will review the agency's \
                 appropriation.",
                context.month.months() / 12
            ));
        }

        if months % 3 == 0 {
            context
                .opportunities
                .push("Promotion round: senior staff are expecting to be considered.".to_owned());
        }

        Ok(())
    }
}

/// The instant a month after the one given.
#[must_use]
pub fn following_instant(month: Instant) -> Instant {
    month.saturating_add(Duration::from_months(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_following_instant_is_a_month_later() {
        assert_eq!(following_instant(Instant::ZERO), Instant::from_months(1));
    }
}
