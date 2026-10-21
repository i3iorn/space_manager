//! Building the Instant report.
//!
//! The report is a structured document, not prose, so the UI can render it in
//! panels. It leads with what happened and ends with what to do.
//! See `docs/SIMULATION.md` §2.4 and `docs/UI.md` §5.

use domain::chronicle::Severity;
use domain::report::{FinancialReport, MonthReport};

use crate::tick::MonthContext;

/// An empty report, for a month that produced nothing.
#[must_use]
pub fn empty(month: domain::time::Instant, date: String) -> MonthReport {
    domain::report::MonthReport::empty(month, date)
}

/// Builds the report for the month that has just resolved.
#[must_use]
pub fn build(context: &MonthContext<'_>) -> MonthReport {
    let state = &*context.state;
    let date = state.calendar().date(context.month).to_string();

    let financial = FinancialReport {
        income: state.treasury.total_income(),
        outgoings: state.treasury.total_outgoings(),
        net: state.treasury.net(),
        balance: state.treasury.balance,
        runway_months: state.treasury.runway_months(),
        crisis_stage: state.treasury.crisis_stage,
    };

    let mut report = MonthReport {
        month: context.month,
        date,
        headline: String::new(),
        commitments_due: Vec::new(),
        personnel: context.personnel_lines.clone(),
        financial,
        programmes: context.programme_lines.clone(),
        events: context.events.clone(),
        warnings: context.warnings.clone(),
        opportunities: context.opportunities.clone(),
        rejections: context.rejections.iter().map(ToString::to_string).collect(),
    };

    collect_commitments(context, &mut report);
    report.headline = headline(context, &report);
    report
}

/// Finds the commitments that came due this Instant.
fn collect_commitments(context: &MonthContext<'_>, report: &mut MonthReport) {
    for programme in context.state.programmes.values() {
        for objective in &programme.objectives {
            let Some(due) = objective.commitment.due() else {
                continue;
            };
            if due != context.month {
                continue;
            }
            report
                .commitments_due
                .push(domain::report::CommitmentReport {
                    description: objective.description.clone(),
                    due,
                    met: objective.met,
                    slack: objective.commitment.slack(),
                    audience: objective.commitment.audience,
                });
        }
    }

    for promise in &context.state.political.promises {
        let Some(due) = promise.commitment.due() else {
            continue;
        };
        if due != context.month {
            continue;
        }
        report
            .commitments_due
            .push(domain::report::CommitmentReport {
                description: promise.description.clone(),
                due,
                met: promise.kept,
                slack: promise.commitment.slack(),
                audience: promise.audience,
            });
    }
}

/// The one sentence the player reads first.
///
/// It is *the thing that needs deciding*, not a summary of the Instant.
fn headline(context: &MonthContext<'_>, report: &MonthReport) -> String {
    if let Some(worst) = report
        .events
        .iter()
        .max_by_key(|entry| severity_rank(entry.severity))
    {
        return worst.headline.clone();
    }

    if !report.commitments_due.is_empty() {
        let count = report.commitments_due.len();
        let noun = if count == 1 {
            "commitment"
        } else {
            "commitments"
        };
        return format!("{count} {noun} came due this Instant.");
    }

    if let Some(warning) = report.warnings.first() {
        return warning.clone();
    }

    if let Some(opportunity) = report.opportunities.first() {
        return opportunity.clone();
    }

    if !context.rejections.is_empty() {
        return format!(
            "{} of your orders could not be carried out.",
            context.rejections.len()
        );
    }

    "Nothing needed your attention this Instant.".to_owned()
}

/// The rank of a severity, for finding the most important entry.
const fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Info => 0,
        Severity::Minor => 1,
        Severity::Major => 2,
        Severity::Critical => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick::Phase;
    use domain::ids::AgencyId;
    use domain::state::{Agency, CampaignState};
    use domain::time::Instant;

    use crate::rng::Rng;

    #[test]
    fn a_quiet_month_has_a_quiet_headline() {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        let rng = Rng::new(29);
        let context = MonthContext::new(&mut state, &rng, Instant::ZERO, Vec::new());
        let report = build(&context);
        assert_eq!(
            report.headline,
            "Nothing needed your attention this Instant."
        );
    }

    #[test]
    fn a_refused_order_becomes_the_headline() {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        let rng = Rng::new(29);
        let mut context = MonthContext::new(
            &mut state,
            &rng,
            Instant::ZERO,
            vec![domain::order::OrderKind::Promote {
                person: domain::ids::PersonId(99),
            }],
        );
        crate::phases::intake::IntakePhase::run(&mut context).expect("intake runs");
        let report = build(&context);
        assert!(report.headline.contains("could not be carried out"));
        assert_eq!(report.rejections.len(), 1);
    }
}
