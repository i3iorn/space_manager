//! Programmes: the strategic layer.
//!
//! A programme carries objectives, each with a `Commitment` holding a soft
//! *target* date and a hard *committed* date. The gap between them is the slack
//! the player manages. See `docs/DOMAIN_MODEL.md` §7.

use serde::{Deserialize, Serialize};

use crate::finance::Budget;
use crate::ids::{CommitmentId, ObjectiveId, PersonId, ProgrammeId};
use crate::research::Progress;
use crate::time::{Duration, Instant};

/// The identifier of a milestone.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct MilestoneId(pub u32);

/// What a programme is for.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ProgrammeCategory {
    /// Getting above the atmosphere and staying there.
    Orbital,
    /// Going to the Moon.
    Lunar,
    /// Going further.
    DeepSpace,
    /// Watching the homeworld.
    EarthObservation,
    /// Watching other nations.
    MilitaryReconnaissance,
    /// Selling services.
    CommercialLaunch,
    /// Staying somewhere.
    SpaceStation,
    /// Digging things up.
    ResourceExtraction,
}

impl ProgrammeCategory {
    /// Every category.
    pub const ALL: [Self; 8] = [
        Self::Orbital,
        Self::Lunar,
        Self::DeepSpace,
        Self::EarthObservation,
        Self::MilitaryReconnaissance,
        Self::CommercialLaunch,
        Self::SpaceStation,
        Self::ResourceExtraction,
    ];

    /// The category's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Orbital => "Orbital exploration",
            Self::Lunar => "Lunar landing",
            Self::DeepSpace => "Deep space probe",
            Self::EarthObservation => "Earth observation",
            Self::MilitaryReconnaissance => "Military reconnaissance",
            Self::CommercialLaunch => "Commercial launch",
            Self::SpaceStation => "Space station",
            Self::ResourceExtraction => "Resource extraction",
        }
    }
}

/// How far along a programme is.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ProgrammeStatus {
    /// Proposed, not yet funded.
    Proposed,
    /// Funded but not started.
    Funded,
    /// Running.
    Active,
    /// Stopped for now.
    Paused,
    /// Finished successfully.
    Completed,
    /// Finished unsuccessfully.
    Failed,
    /// Abandoned.
    Cancelled,
}

impl ProgrammeStatus {
    /// Every status.
    pub const ALL: [Self; 7] = [
        Self::Proposed,
        Self::Funded,
        Self::Active,
        Self::Paused,
        Self::Completed,
        Self::Failed,
        Self::Cancelled,
    ];

    /// The status's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Proposed => "Proposed",
            Self::Funded => "Funded",
            Self::Active => "Active",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }

    /// Whether the programme is still running.
    #[must_use]
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Active | Self::Paused)
    }
}

/// Who is holding a promise.
///
/// The audience determines the consequence of a missed date, which is why it
/// is stored on the commitment rather than in a policy table: the same
/// programme can owe different things to different parties.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum DeadlineAudience {
    /// The agency itself.
    Internal,
    /// A contractor or partner.
    Contractual,
    /// The state.
    Political,
    /// The public.
    Public,
    /// A regulator, or the sky.
    Mandatory,
}

impl DeadlineAudience {
    /// Every audience.
    pub const ALL: [Self; 5] = [
        Self::Internal,
        Self::Contractual,
        Self::Political,
        Self::Public,
        Self::Mandatory,
    ];

    /// The audience's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Internal => "the agency",
            Self::Contractual => "a contractor",
            Self::Political => "the state",
            Self::Public => "the public",
            Self::Mandatory => "the regulator",
        }
    }

    /// Whether missing this date is fatal.
    ///
    /// Internal dates slip; political and mandatory deadlines are not negotiable.
    #[must_use]
    pub const fn is_hard(self) -> bool {
        matches!(self, Self::Political | Self::Public | Self::Mandatory)
    }
}

/// A date the agency has committed to.
///
/// The *target* is the agency's honest internal plan and is soft. The
/// *committed* date is what was promised to whoever holds the deadline and is
/// hard. The gap between them is the slack.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Commitment {
    /// The commitment's identifier.
    ///
    /// **A commitment is a thing that falls due at an instant**, and a thing
    /// that falls due at an instant has to be addressable, because that is
    /// what the scheduler holds.
    pub id: CommitmentId,
    /// The agency's internal plan.
    pub target: Option<Instant>,
    /// What was promised.
    pub committed: Option<Instant>,
    /// Who the promise was made to.
    pub audience: DeadlineAudience,
}

impl Commitment {
    /// A commitment with only a promised date.
    #[must_use]
    pub const fn committed(id: CommitmentId, committed: Instant) -> Self {
        Self {
            id,
            target: None,
            committed: Some(committed),
            audience: DeadlineAudience::Internal,
        }
    }

    /// A commitment with a plan and a promise, and who was promised to.
    #[must_use]
    pub const fn new(
        id: CommitmentId,
        target: Option<Instant>,
        committed: Option<Instant>,
        audience: DeadlineAudience,
    ) -> Self {
        Self {
            id,
            target,
            committed,
            audience,
        }
    }

    /// The slack: the time between the plan and the promise.
    ///
    /// This is the buffer the player manages: a commitment with fourteen months
    /// of slack absorbs a nine-month delay without consequence, and one with no
    /// slack is a gamble the player chose to make.
    #[must_use]
    pub fn slack(&self) -> Option<Duration> {
        match (self.target, self.committed) {
            (Some(target), Some(committed)) => {
                if committed >= target {
                    Some(committed.duration_since(target))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// The date the agency is actually held to, if any.
    #[must_use]
    pub fn due(&self) -> Option<Instant> {
        self.committed.or(self.target)
    }

    /// Whether this commitment has come due by the given instant.
    #[must_use]
    pub fn is_due(&self, now: Instant) -> bool {
        self.due().is_some_and(|due| due <= now)
    }
}

/// Whether an objective must be met or is a bonus.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ObjectiveKind {
    /// The programme fails without it.
    Primary,
    /// A bonus.
    Secondary,
}

impl ObjectiveKind {
    /// The kind's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }
}

/// Something a programme is trying to achieve.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Objective {
    /// The objective's identifier.
    pub id: ObjectiveId,
    /// What it is, in the player's language.
    pub description: String,
    /// Whether it must be met.
    pub kind: ObjectiveKind,
    /// When it is due.
    pub commitment: Commitment,
    /// How far along it is.
    pub progress: Progress,
    /// Whether it has been met.
    pub met: Option<bool>,
    /// The fractional remainder carried between rate applications.
    ///
    /// **The objective's progress is a rate, and a rate that is truncated
    /// every hour loses its remainder.** Over a thirty-year campaign that is
    /// points that simply never arrive.
    pub remainders: crate::scale::Remainders,
}

impl Objective {
    /// A new objective.
    #[must_use]
    pub fn new(
        id: ObjectiveId,
        description: String,
        kind: ObjectiveKind,
        commitment: Commitment,
    ) -> Self {
        Self {
            id,
            description,
            kind,
            commitment,
            progress: Progress::empty(100),
            met: None,
            remainders: crate::scale::Remainders::new(),
        }
    }
}

/// A dated checkpoint in a programme's plan.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Milestone {
    /// The milestone's identifier.
    pub id: MilestoneId,
    /// What it is.
    pub description: String,
    /// When it falls due.
    pub date: Instant,
    /// Whether it has been reached.
    pub met: bool,
}

/// Something that could go wrong with a programme.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Risk {
    /// The risk's identifier.
    pub id: u32,
    /// What it is.
    pub description: String,
    /// How bad it would be, from 0 to 100.
    pub severity: u8,
    /// How much has been done about it.
    pub mitigation: u8,
}

/// A strategic programme.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Programme {
    /// The programme's identifier.
    pub id: ProgrammeId,
    /// Its title.
    pub title: String,
    /// What it is for.
    pub category: ProgrammeCategory,
    /// How far along it is.
    pub status: ProgrammeStatus,
    /// Who runs it.
    pub lead: Option<PersonId>,
    /// What it costs.
    pub budget: Budget,
    /// Everyone on it.
    pub staffing: Vec<PersonId>,
    /// What it is trying to achieve.
    pub objectives: Vec<Objective>,
    /// Its dated checkpoints.
    pub milestones: Vec<Milestone>,
    /// What could go wrong.
    pub risks: Vec<Risk>,
    /// How publicly visible it is, from 0 to 100.
    pub visibility: u8,
}

impl Programme {
    /// A new programme.
    #[must_use]
    pub fn new(id: ProgrammeId, title: String, category: ProgrammeCategory) -> Self {
        Self {
            id,
            title,
            category,
            status: ProgrammeStatus::Proposed,
            lead: None,
            budget: Budget::empty(),
            staffing: Vec::new(),
            objectives: Vec::new(),
            milestones: Vec::new(),
            risks: Vec::new(),
            visibility: 20,
        }
    }

    /// The programme's primary objective, if it has one.
    #[must_use]
    pub fn primary_objective(&self) -> Option<&Objective> {
        self.objectives
            .iter()
            .find(|objective| objective.kind == ObjectiveKind::Primary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_commitment_holds_both_dates() {
        let commitment = Commitment::new(
            CommitmentId(0),
            Some(Instant::from_months(10)),
            Some(Instant::from_months(14)),
            DeadlineAudience::Political,
        );
        assert_eq!(commitment.slack(), Some(Duration::from_months(4)));
        assert_eq!(commitment.due(), Some(Instant::from_months(14)));
    }

    #[test]
    fn a_commitment_with_only_a_promise_has_no_slack() {
        assert_eq!(
            Commitment::committed(CommitmentId(0), Instant::from_months(10)).slack(),
            None
        );
    }

    #[test]
    fn political_and_mandatory_deadlines_are_hard() {
        assert!(DeadlineAudience::Political.is_hard());
        assert!(DeadlineAudience::Mandatory.is_hard());
        assert!(!DeadlineAudience::Internal.is_hard());
        assert!(!DeadlineAudience::Contractual.is_hard());
    }

    #[test]
    fn a_programme_finds_its_primary_objective() {
        let mut programme = Programme::new(
            ProgrammeId(1),
            "Orbital".to_owned(),
            ProgrammeCategory::Orbital,
        );
        assert!(programme.primary_objective().is_none());
        programme.objectives.push(Objective::new(
            ObjectiveId(0),
            "Reach orbit".to_owned(),
            ObjectiveKind::Primary,
            Commitment::committed(CommitmentId(0), Instant::from_months(12)),
        ));
        programme.objectives.push(Objective::new(
            ObjectiveId(1),
            "Return a capsule".to_owned(),
            ObjectiveKind::Secondary,
            Commitment::committed(CommitmentId(0), Instant::from_months(18)),
        ));
        assert_eq!(
            programme
                .primary_objective()
                .map(|o| o.description.as_str()),
            Some("Reach orbit")
        );
    }
}
