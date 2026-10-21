//! Phase 10 — commit and report.
//!
//! The Instant's staged deltas are applied as a single atomic unit and the Instant
//! report is produced. A Instant either happened or it did not.
//!
//! The state was already a working copy when the Instant began, so committing is
//! a matter of writing the chronicle entries and advancing the calendar.

use domain::chronicle::ChronicleEntry;

use crate::error::SimError;
use crate::report::build;
use crate::tick::{MonthContext, Phase};

/// The commit phase.
pub struct CommitPhase;

impl Phase for CommitPhase {
    const NAME: &'static str = "commit";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        // The chronicle is append-only, and the entries the Instant generated are
        // written in the order they happened.
        let entries: Vec<ChronicleEntry> = std::mem::take(&mut context.events);
        for entry in entries {
            context.state.chronicle(entry);
        }

        // The clock itself is the simulation's, and it advances after the commit.

        Ok(())
    }
}

/// Builds the report for a resolved Instant.
///
/// The report is built before the commit, so it describes the Instant that has
/// just happened rather than the Instant the campaign has moved into.
#[must_use]
pub fn report_for(context: &MonthContext<'_>) -> domain::report::MonthReport {
    build(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::AgencyId;
    use domain::state::{Agency, CampaignState};
    use domain::time::Instant;

    use crate::rng::Rng;

    #[test]
    fn committing_writes_the_chronicle() {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        let rng = Rng::new(31);
        let mut context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        context.events.push(domain::chronicle::ChronicleEntry::new(
            domain::ids::ChronicleId(0),
            Instant::ZERO,
            domain::chronicle::Category::Discovery,
            "Something happened".to_owned(),
            domain::chronicle::Severity::Info,
        ));
        CommitPhase::run(&mut context).expect("the phase runs");
        assert_eq!(context.state.chronicle.len(), 1);
    }
}
