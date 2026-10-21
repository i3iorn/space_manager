//! The Instant report: the structured account of what a Instant did.
//!
//! Every Instant produces one, even a Instant that changed nothing. The report is
//! what the player reads each turn, and it is structured rather than prose so
//! the UI can render it in panels. See `docs/SIMULATION.md` §2.4.

use serde::{Deserialize, Serialize};

use crate::chronicle::ChronicleEntry;
use crate::money::Money;
use crate::time::Duration;
use crate::time::Instant;

/// One line of the report.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ReportLine {
    /// The line's text, in the player's language.
    pub text: String,
    /// How much it matters.
    pub severity: crate::chronicle::Severity,
    /// What it is about, if anything.
    pub subject: Option<crate::chronicle::SubjectRef>,
}

impl ReportLine {
    /// A line of ordinary text.
    #[must_use]
    pub fn new(text: String) -> Self {
        Self {
            text,
            severity: crate::chronicle::Severity::Info,
            subject: None,
        }
    }

    /// A line with a severity.
    #[must_use]
    pub fn with_severity(mut self, severity: crate::chronicle::Severity) -> Self {
        self.severity = severity;
        self
    }
}

/// A commitment that came due this Instant.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CommitmentReport {
    /// What was promised.
    pub description: String,
    /// When it was due.
    pub due: Instant,
    /// Whether it was met.
    pub met: Option<bool>,
    /// How much slack there was, as a duration.
    pub slack: Option<Duration>,
    /// Who it was promised to.
    pub audience: crate::programme::DeadlineAudience,
}

/// What the money did.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct FinancialReport {
    /// What came in.
    pub income: Money,
    /// What went out.
    pub outgoings: Money,
    /// The difference.
    pub net: Money,
    /// The balance.
    pub balance: Money,
    /// How many months the treasury lasts at this burn rate.
    pub runway_months: u32,
    /// How far down the crisis ladder the agency is.
    pub crisis_stage: crate::finance::CrisisStage,
}

/// The account of one month.
///
/// Every section is always present. A month that changed nothing still produces
/// a report with empty sections, because silence is information.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MonthReport {
    /// The instant the report covers.
    pub month: Instant,
    /// The instant's readable date.
    pub date: String,
    /// The one thing that needs deciding.
    pub headline: String,
    /// Commitments that came due.
    pub commitments_due: Vec<CommitmentReport>,
    /// What happened to the people.
    pub personnel: Vec<ReportLine>,
    /// What happened to the money.
    pub financial: FinancialReport,
    /// What happened to the programmes.
    pub programmes: Vec<ReportLine>,
    /// What happened.
    pub events: Vec<ChronicleEntry>,
    /// Things that will become a problem if ignored.
    pub warnings: Vec<String>,
    /// Things the player could act on.
    pub opportunities: Vec<String>,
    /// Orders that were refused, and why.
    pub rejections: Vec<String>,
}

impl MonthReport {
    /// An empty report for an instant.
    #[must_use]
    pub fn empty(month: Instant, date: String) -> Self {
        Self {
            month,
            date,
            headline: "Nothing needed your attention.".to_owned(),
            commitments_due: Vec::new(),
            personnel: Vec::new(),
            financial: FinancialReport {
                income: Money::ZERO,
                outgoings: Money::ZERO,
                net: Money::ZERO,
                balance: Money::ZERO,
                runway_months: u32::MAX,
                crisis_stage: crate::finance::CrisisStage::None,
            },
            programmes: Vec::new(),
            events: Vec::new(),
            warnings: Vec::new(),
            opportunities: Vec::new(),
            rejections: Vec::new(),
        }
    }

    /// Whether the report was completely uneventful.
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.commitments_due.is_empty()
            && self.personnel.is_empty()
            && self.programmes.is_empty()
            && self.events.is_empty()
            && self.warnings.is_empty()
            && self.opportunities.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quiet_month_is_still_a_report() {
        let report = MonthReport::empty(Instant::ZERO, "2039-01".to_owned());
        assert!(report.is_quiet());
        assert!(!report.headline.is_empty());
    }

    #[test]
    fn a_report_with_warnings_is_not_quiet() {
        let mut report = MonthReport::empty(Instant::ZERO, "2039-01".to_owned());
        report.warnings.push("The treasury is low.".to_owned());
        assert!(!report.is_quiet());
    }
}
