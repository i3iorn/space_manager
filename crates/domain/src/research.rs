//! Research projects, disciplines, and capabilities.
//!
//! Capabilities are thresholds on accumulated knowledge, not a purchase list,
//! and a test campaign is a separate activity from a research project: one
//! produces capability, the other produces reliability, and they compete for the
//! same staff. See `docs/DOMAIN_MODEL.md` §8.

use serde::{Deserialize, Serialize};

use crate::finance::Budget;
use crate::ids::{PersonId, ResearchProjectId, TestCampaignId};
use crate::time::Instant;

/// The identifier of a research discipline.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct DisciplineId(pub u8);

/// The identifier of a capability.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct CapabilityId(pub u8);

/// A field of research.
///
/// The catalogues are code constants rather than saved state, so they do not
/// derive serde: the campaign stores capability *identifiers*, never the
/// catalogue itself.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Discipline {
    /// The discipline's identifier.
    pub id: DisciplineId,
    /// Its name.
    pub name: &'static str,
    /// The disciplines whose breakthroughs nudge this one.
    pub adjacent: &'static [DisciplineId],
}

/// The catalogue of research disciplines.
///
/// Defined in one place, so that adding a discipline is a data change rather
/// than a code change. The catalogue is a static slice rather than a `Vec` so
/// that it can live in a constant, with no allocation and no lazy initialisation.
pub const DISCIPLINES: [Discipline; 3] = [
    Discipline {
        id: DisciplineId(0),
        name: "Propulsion",
        adjacent: &[DisciplineId(1), DisciplineId(2)],
    },
    Discipline {
        id: DisciplineId(1),
        name: "Structures & Materials",
        adjacent: &[DisciplineId(0), DisciplineId(2)],
    },
    Discipline {
        id: DisciplineId(2),
        name: "Life Support",
        adjacent: &[DisciplineId(0), DisciplineId(1)],
    },
];

/// The name of a discipline.
#[must_use]
pub fn discipline_name(id: DisciplineId) -> &'static str {
    DISCIPLINES
        .iter()
        .find(|discipline| discipline.id == id)
        .map_or("Unknown", |discipline| discipline.name)
}

/// Something a capability makes possible.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Unlock {
    /// A vehicle subsystem the agency may now specify.
    VehicleSlider {
        /// The slider's name.
        slider: &'static str,
    },
    /// A destination the agency may now reach.
    Destination {
        /// The destination's name.
        destination: &'static str,
    },
    /// A facility option the agency may now build.
    FacilityOption {
        /// The option's name.
        option: &'static str,
    },
}

/// A capability the agency can develop.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Capability {
    /// The capability's identifier.
    pub id: CapabilityId,
    /// Its name.
    pub name: &'static str,
    /// The discipline that develops it.
    pub discipline: DisciplineId,
    /// How much knowledge is needed to unlock it.
    pub threshold: u32,
    /// What it makes possible.
    pub unlocks: &'static [Unlock],
}

/// The catalogue of capabilities.
///
/// The thresholds overlap on purpose: a player who rushes to orbital
/// propulsion and skips life support can reach orbit, and then be unable to
/// keep anyone alive for a long mission. That is a legitimate strategy with a
/// specific weakness.
pub const CAPABILITIES: [Capability; 5] = [
    Capability {
        id: CapabilityId(0),
        name: "Atmospheric sounding",
        discipline: DisciplineId(0),
        threshold: 0,
        unlocks: &[],
    },
    Capability {
        id: CapabilityId(1),
        name: "Basic re-entry",
        discipline: DisciplineId(1),
        threshold: 300,
        unlocks: &[Unlock::VehicleSlider {
            slider: "heat shield",
        }],
    },
    Capability {
        id: CapabilityId(2),
        name: "Orbital propulsion",
        discipline: DisciplineId(0),
        threshold: 240,
        unlocks: &[
            Unlock::Destination {
                destination: "Orbital",
            },
            Unlock::Destination {
                destination: "Low orbit",
            },
        ],
    },
    Capability {
        id: CapabilityId(3),
        name: "Closed-loop life support",
        discipline: DisciplineId(2),
        threshold: 380,
        unlocks: &[Unlock::Destination {
            destination: "Long duration",
        }],
    },
    Capability {
        id: CapabilityId(4),
        name: "Orbital launch",
        discipline: DisciplineId(0),
        threshold: 420,
        unlocks: &[Unlock::Destination {
            destination: "Orbital",
        }],
    },
];

/// The discipline a destination belongs to.
///
/// **A destination names a capability, and a capability has a discipline**, so
/// the data a mission brings back goes to the field that asked for it.
#[must_use]
pub fn discipline_for_destination(destination: &str) -> Option<DisciplineId> {
    CAPABILITIES
        .iter()
        .find(|capability| {
            capability
                .unlocks
                .iter()
                .any(|unlock| matches!(unlock, Unlock::Destination { destination: name } if name.eq_ignore_ascii_case(destination)))
        })
        .map(|capability| capability.discipline)
}

/// The name of a capability.
#[must_use]
pub fn capability_name(id: CapabilityId) -> &'static str {
    CAPABILITIES
        .iter()
        .find(|capability| capability.id == id)
        .map_or("Unknown", |capability| capability.name)
}

/// How far along something is.
///
/// Progress is cumulative, not a percentage: a setback subtracts points rather
/// than resetting the work, so a long project that slips does not lose
/// everything it has achieved.
///
/// The points are **scaled integers** at `256` units per point, so that a rate of
/// a fraction of a point a month is not truncated away.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Progress {
    /// Points accumulated, at `256` units per point.
    pub points: u32,
    /// Points needed to finish, at `256` units per point.
    pub target: u32,
}

impl Progress {
    /// Progress that has not started.
    #[must_use]
    pub const fn empty(target: u32) -> Self {
        Self { points: 0, target }
    }

    /// The progress as a percentage.
    #[must_use]
    pub fn percent(&self) -> u32 {
        self.points
            .saturating_mul(100)
            .checked_div(self.target)
            .unwrap_or(100)
    }

    /// The progress in whole points.
    #[must_use]
    pub const fn whole_points(&self) -> u32 {
        self.points / crate::scale::PERCENT_UNITS as u32
    }
    /// Whether the work is finished.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.target > 0 && self.points >= self.target
    }
}

/// A piece of research the agency is running.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ResearchProject {
    /// The project's identifier.
    pub id: ResearchProjectId,
    /// Its title.
    pub title: String,
    /// The field it is in.
    pub discipline: DisciplineId,
    /// Who is running it.
    pub principal_investigator: PersonId,
    /// Everyone working on it.
    pub team: Vec<PersonId>,
    /// What it costs.
    pub funding: Budget,
    /// How far along it is.
    pub progress: Progress,
    /// The capability it is aiming at, if it has one.
    pub target_capability: Option<CapabilityId>,
    /// When it started.
    pub started: Instant,
    /// When it is expected to finish. An estimate, never a promise.
    pub expected_completion: Option<Instant>,
    /// The fractional remainder carried between rate applications.
    ///
    /// The progress rate is a fraction of a point a month, and without this the
    /// fraction is lost at every application.
    pub remainders: crate::scale::Remainders,
}

/// A campaign to test something the agency already knows how to build.
///
/// This is not a research project. It produces no new capability; it reduces
/// the risk of something the agency has already developed. That is the whole
/// point: capability and reliability compete for the same scarce people.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TestCampaign {
    /// The campaign's identifier.
    pub id: TestCampaignId,
    /// What is being tested.
    pub target_capability: CapabilityId,
    /// Who is running it.
    pub team: Vec<PersonId>,
    /// What it costs.
    pub funding: Budget,
    /// How far along it is.
    pub progress: Progress,
    /// How much certainty it buys, in percentage points.
    pub risk_reduction: i16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_discipline_has_a_name() {
        for discipline in &DISCIPLINES {
            assert_eq!(discipline_name(discipline.id), discipline.name);
        }
    }

    #[test]
    fn adjacent_disciplines_exist() {
        for discipline in &DISCIPLINES {
            for adjacent in discipline.adjacent {
                assert!(
                    DISCIPLINES.iter().any(|d| d.id == *adjacent),
                    "{} points at a discipline that does not exist",
                    discipline.name
                );
            }
        }
    }

    #[test]
    fn capabilities_point_at_real_disciplines() {
        for capability in &CAPABILITIES {
            assert!(
                DISCIPLINES.iter().any(|d| d.id == capability.discipline),
                "{} points at a discipline that does not exist",
                capability.name
            );
        }
    }

    #[test]
    fn capability_thresholds_are_not_ordered_by_identifier() {
        // Thresholds overlap on purpose, so the unlock order is not the
        // identifier order. This is a property of the design, not an accident:
        // re-entry is harder than reaching orbit, so a player may reach orbit
        // without ever developing it — a legitimate strategy with a specific
        // weakness.
        let thresholds: Vec<u32> = CAPABILITIES.iter().map(|c| c.threshold).collect();
        let mut sorted = thresholds.clone();
        sorted.sort_unstable();
        assert_ne!(thresholds, sorted);
    }

    #[test]
    fn a_player_can_reach_orbit_without_re_entry() {
        // Orbital propulsion opens low orbit at 240; re-entry costs 300. A
        // player who rushes the cheap path can therefore get above the
        // atmosphere without ever developing a heat shield — a legitimate
        // strategy with a specific weakness.
        let propulsion = CAPABILITIES
            .iter()
            .find(|capability| capability.name == "Orbital propulsion")
            .expect("the catalogue has an orbital propulsion capability");
        let reentry = CAPABILITIES
            .iter()
            .find(|capability| capability.name == "Basic re-entry")
            .expect("the catalogue has a re-entry capability");
        assert!(propulsion.threshold < reentry.threshold);
    }

    #[test]
    fn progress_is_cumulative_not_a_percentage() {
        let mut progress = Progress::empty(100);
        progress.points = 60;
        assert_eq!(progress.percent(), 60);
        progress.points = 40;
        assert_eq!(progress.percent(), 40);
        assert!(!progress.is_complete());
        progress.points = 100;
        assert!(progress.is_complete());
    }
}
