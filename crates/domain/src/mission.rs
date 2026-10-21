//! Missions: state machines with an outcome.
//!
//! A mission advances through preparation, launch, transit, operations, return,
//! and archival. A failure is usually *partial* rather than catastrophic, and
//! the outcome type must be able to express both. See `docs/DOMAIN_MODEL.md` §11.

use serde::{Deserialize, Serialize};

use crate::ids::{MissionId, PersonId, ProgrammeId, VehicleId};
use crate::time::{Duration, Instant};

/// Where a mission is in its life.
///
/// `Failed` records the phase the mission died in, which matters for the
/// chronicle and for partial outcomes.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MissionPhase {
    /// Crew training, loadout, and final checks.
    Preparation,
    /// On the pad.
    Launch,
    /// Under way.
    Transit,
    /// Doing the work.
    Operations,
    /// Coming home.
    Return,
    /// The data is being processed.
    Archival,
    /// It failed, at the phase named.
    Failed(Box<MissionPhase>),
}

impl MissionPhase {
    /// Whether the mission is still in progress.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            Self::Preparation | Self::Launch | Self::Transit | Self::Operations | Self::Return
        )
    }

    /// The phase's name, as shown to the player.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Preparation => "Preparation",
            Self::Launch => "Launch",
            Self::Transit => "Transit",
            Self::Operations => "Operations",
            Self::Return => "Return",
            Self::Archival => "Archival",
            Self::Failed(_) => "Failed",
        }
    }

    /// How long the phase takes.
    ///
    /// **The ascent is the short one.** Everything else is weeks, and the
    /// ninety minutes of the launch are the part the player watches.
    #[must_use]
    pub fn duration(&self) -> Duration {
        match self {
            Self::Preparation => Duration::from_days(21),
            Self::Launch => Duration::from_mins(75),
            Self::Transit | Self::Return => Duration::from_days(4),
            Self::Operations => Duration::from_days(7),
            Self::Archival => Duration::from_days(14),
            Self::Failed(_) => Duration::ZERO,
        }
    }

    /// The phase that comes next.
    #[must_use]
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Preparation => Some(Self::Launch),
            Self::Launch => Some(Self::Transit),
            Self::Transit => Some(Self::Operations),
            Self::Operations => Some(Self::Return),
            Self::Return => Some(Self::Archival),
            Self::Archival | Self::Failed(_) => None,
        }
    }
}

/// Something a mission learned or lost.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct MissionResult {
    /// What it found.
    pub finding: String,
    /// How good it was, from 0 to 100.
    pub quality: u8,
}

/// What became of a mission.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Outcome {
    /// Whether it worked.
    pub success: bool,
    /// How bad it was, from 0 to 100. Zero for a success.
    pub severity: u8,
    /// Whether it partly worked.
    pub partial: bool,
    /// What it learned.
    pub results: Vec<MissionResult>,
}

impl Outcome {
    /// A clean success.
    #[must_use]
    pub fn success(results: Vec<MissionResult>) -> Self {
        Self {
            success: true,
            severity: 0,
            partial: false,
            results,
        }
    }

    /// A partial failure.
    #[must_use]
    pub fn partial(severity: u8, results: Vec<MissionResult>) -> Self {
        Self {
            success: false,
            severity,
            partial: true,
            results,
        }
    }
}

/// A mission the agency is flying.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Mission {
    /// The mission's identifier.
    pub id: MissionId,
    /// The vehicle it flies.
    pub vehicle: VehicleId,
    /// Where it is going.
    pub destination: String,
    /// Who is aboard.
    pub crew: Vec<PersonId>,
    /// Where it is in its life.
    pub phase: MissionPhase,
    /// When it was authorised.
    pub authorised: Instant,
    /// When it launches.
    pub launch_window: Instant,
    /// The chance it works, in basis points.
    pub probability: u16,
    /// What became of it, once it is over.
    pub outcome: Option<Outcome>,
    /// The programme it belongs to.
    pub programme: Option<ProgrammeId>,
}

impl Mission {
    /// A newly authorised mission.
    #[must_use]
    pub fn new(
        id: MissionId,
        vehicle: VehicleId,
        destination: String,
        crew: Vec<PersonId>,
        authorised: Instant,
        launch_window: Instant,
        programme: Option<ProgrammeId>,
    ) -> Self {
        Self {
            id,
            vehicle,
            destination,
            crew,
            phase: MissionPhase::Preparation,
            authorised,
            launch_window,
            probability: 0,
            outcome: None,
            programme,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_mission_is_in_preparation() {
        let mission = Mission::new(
            MissionId(0),
            VehicleId(0),
            "Low orbit".to_owned(),
            vec![PersonId(0)],
            Instant::ZERO,
            Instant::from_months(6),
            None,
        );
        assert_eq!(mission.phase, MissionPhase::Preparation);
        assert!(mission.phase.is_running());
    }

    #[test]
    fn a_failed_mission_is_not_running() {
        let failed = MissionPhase::Failed(Box::new(MissionPhase::Launch));
        assert!(!failed.is_running());
        assert_eq!(failed.label(), "Failed");
    }
}
