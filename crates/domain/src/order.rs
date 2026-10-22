//! Orders: the only way the player changes anything.
//!
//! An order is a request, not a mutation. It is validated without side
//! effects, and an order that cannot be honoured is rejected with a reason
//! that reaches the player. It is never silently ignored or partially applied.
//!
//! **An order is a decision that must be implemented [D].** It carries when it
//! was made, when it lands, and what it costs. Every order has a lead time,
//! and the player sees the lead time, the price, and the goodwill charge
//! before they commit.
//!
//! See `docs/SIMULATION.md` §4.

use serde::{Deserialize, Serialize};

use crate::facility::{ConstructionPlan, FacilityType};
use crate::ids::{FacilityId, MissionId, ObjectiveId, OrderId, PersonId, ProgrammeId, VehicleId};
use crate::money::Money;
use crate::person::{Assignment, Role};
use crate::programme::{Commitment, ObjectiveKind, ProgrammeCategory};
use crate::research::{CapabilityId, DisciplineId};
use crate::time::Instant;

/// An order the player has issued, and the agency has not yet carried out.
///
/// The order lives in the campaign state until it lands, and it is **part of
/// the save**: a campaign saved with a hire in flight must restore that hire.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Order {
    /// The order's identifier.
    pub id: OrderId,
    /// When the player gave it.
    pub issued_at: Instant,
    /// When it lands: the issue instant plus the lead time.
    pub effective_at: Instant,
    /// What it is.
    pub kind: OrderKind,
    /// What it costs, charged when it lands.
    pub cost: Money,
    /// What it costs in goodwill, charged when it lands.
    pub goodwill: i16,
}

impl Order {
    /// How long the order takes to implement.
    #[must_use]
    pub fn lead_time(&self) -> crate::time::Duration {
        crate::time::Duration::from_millis(
            self.effective_at
                .millis()
                .saturating_sub(self.issued_at.millis()),
        )
    }
}

/// What happened to an order.
///
/// The record the player reads: an order that landed, or an order that was
/// refused, and why.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct OrderReport {
    /// The order's kind, as a stable name.
    pub kind: String,
    /// When it was issued.
    pub issued_at: Instant,
    /// When it was due to land.
    pub effective_at: Instant,
    /// Whether it landed.
    pub landed: bool,
    /// Why it was refused, if it was.
    pub reason: Option<String>,
}

/// How a piece of knowledge is released.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum KnowledgeMode {
    /// Publish openly. Reputation, and the world gets faster too.
    Open,
    /// Patent and license. Income and exclusivity.
    Patent,
    /// License to a rival. Immediate cash, and the rival advances.
    License,
}

impl KnowledgeMode {
    /// Every mode.
    pub const ALL: [Self; 3] = [Self::Open, Self::Patent, Self::License];

    /// The mode's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "Publish openly",
            Self::Patent => "Patent and license",
            Self::License => "License to a rival",
        }
    }
}

/// Something the player wants the agency to do.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum OrderKind {
    /// Hire someone.
    Hire {
        /// What they will do.
        role: Role,
        /// What they will be paid.
        salary: Money,
    },

    /// Let someone go.
    Fire {
        /// Who.
        person: PersonId,
    },

    /// Put someone on something.
    Assign {
        /// Who.
        person: PersonId,
        /// What they will do.
        to: Assignment,
    },

    /// Take someone off something.
    Unassign {
        /// Who.
        person: PersonId,
    },

    /// Set how hard somebody works.
    ///
    /// **The shift is the player's throttle on a person.** It is a scaled
    /// percentage of a normal load: a hundred is a full load, and the range
    /// runs to two hundred.
    ///
    /// **A shift given to one person is an override of the agency policy**,
    /// and `None` puts them back on it — which is the other half of the
    /// override, and the way a player says "she works flat out until she is
    /// tired, and then she eases off".
    SetShift {
        /// Who.
        person: PersonId,
        /// How hard, in scaled percent. Zero to two hundred, or `None` to
        /// follow the agency's policy.
        shift: Option<u16>,
    },

    /// Give someone a step up.
    Promote {
        /// Who.
        person: PersonId,
    },

    /// Put someone through a course.
    Train {
        /// Who.
        person: PersonId,
        /// What they will learn.
        course: String,
    },

    /// Start a programme.
    CreateProgramme {
        /// What it is called.
        title: String,
        /// What it is for.
        category: ProgrammeCategory,
    },

    /// Add an objective to a programme.
    SetObjective {
        /// Which programme.
        programme: ProgrammeId,
        /// What the objective is.
        description: String,
        /// When it is due, and who it is due to.
        commitment: Commitment,
        /// Whether it must be met.
        kind: ObjectiveKind,
    },

    /// Push a commitment back.
    ///
    /// **Renegotiating is not free.** The cost depends on who holds the
    /// deadline: the state does not forgive a broken promise for nothing.
    PostponeDeadline {
        /// Which programme.
        programme: ProgrammeId,
        /// Which objective's commitment.
        objective: ObjectiveId,
        /// The new committed instant.
        to: Instant,
    },

    /// Set a programme's budget.
    SetBudget {
        /// Which programme.
        programme: ProgrammeId,
        /// How much it gets.
        allocation: Money,
    },

    /// Start a research project.
    StartProject {
        /// The field.
        discipline: DisciplineId,
        /// Who will run it.
        principal_investigator: PersonId,
        /// What it is called.
        title: String,
    },

    /// Start a test campaign.
    StartTestCampaign {
        /// What is being tested.
        capability: CapabilityId,
    },

    /// Begin building a facility.
    BuildFacility {
        /// What kind.
        kind: FacilityType,
        /// How it should be built.
        plan: ConstructionPlan,
    },

    /// Authorise a launch.
    AuthorizeLaunch {
        /// Which vehicle.
        vehicle: VehicleId,
        /// Where it is going.
        destination: String,
        /// Who is aboard.
        crew: Vec<PersonId>,
    },

    /// Design a vehicle.
    ///
    /// **The design is the player's decision**, and the derived statistics
    /// follow from it. The sliders are bounded by the archetype and by what
    /// the agency has developed, and the order carries the bounded values so
    /// that the design that is applied is the design that was legal.
    DesignVehicle {
        /// What kind of vehicle it is.
        archetype: crate::vehicle::ArchetypeId,
        /// What it is called.
        name: String,
        /// The sliders, as chosen.
        sliders: crate::vehicle::SliderValues,
    },

    /// Change one of a vehicle's subsystems.
    SetSubsystem {
        /// Which vehicle.
        vehicle: VehicleId,
        /// Which subsystem.
        slider: crate::vehicle::Slider,
        /// How far up, from 0 to 100.
        value: u8,
    },

    /// Change a standing instruction.
    ///
    /// **A policy is a standing instruction and not a one-off.** The
    /// per-person shift is the surgical tool; this is the tool for an agency of
    /// three hundred people, and it is the answer to "my team is burning out".
    SetPolicy {
        /// What the instruction says.
        policy: crate::person::Policy,
    },

    /// Release a piece of knowledge.
    PublishResult {
        /// Which result.
        result: String,
        /// How it is released.
        mode: KnowledgeMode,
    },

    /// Attach a facility to a programme.
    AssignFacility {
        /// Which facility.
        facility: FacilityId,
        /// Which programme.
        programme: ProgrammeId,
    },

    /// Cancel a mission.
    CancelMission {
        /// Which mission.
        mission: MissionId,
    },
}

impl OrderKind {
    /// A short name for the order, used in the log and in messages.
    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::Hire { .. } => "Hire",
            Self::Fire { .. } => "Fire",
            Self::Assign { .. } => "Assign",
            Self::Unassign { .. } => "Unassign",
            Self::SetShift { .. } => "SetShift",
            Self::Promote { .. } => "Promote",
            Self::Train { .. } => "Train",
            Self::CreateProgramme { .. } => "CreateProgramme",
            Self::SetObjective { .. } => "SetObjective",
            Self::SetBudget { .. } => "SetBudget",
            Self::DesignVehicle { .. } => "DesignVehicle",
            Self::SetSubsystem { .. } => "SetSubsystem",
            Self::PostponeDeadline { .. } => "PostponeDeadline",
            Self::StartProject { .. } => "StartProject",
            Self::StartTestCampaign { .. } => "StartTestCampaign",
            Self::BuildFacility { .. } => "BuildFacility",
            Self::AuthorizeLaunch { .. } => "AuthorizeLaunch",
            Self::SetPolicy { .. } => "SetPolicy",
            Self::PublishResult { .. } => "PublishResult",
            Self::AssignFacility { .. } => "AssignFacility",
            Self::CancelMission { .. } => "CancelMission",
        }
    }
}

/// The names of every order kind, in a stable order.
///
/// **These are part of the engine's version.** A new kind, or a renamed one,
/// changes what a campaign does, and a save made before the change cannot be
/// replayed after it.
#[must_use]
pub fn order_kind_names() -> Vec<&'static str> {
    vec![
        "Hire",
        "Fire",
        "Assign",
        "Unassign",
        "SetShift",
        "Promote",
        "Train",
        "CreateProgramme",
        "SetObjective",
        "SetBudget",
        "DesignVehicle",
        "SetSubsystem",
        "PostponeDeadline",
        "StartProject",
        "StartTestCampaign",
        "BuildFacility",
        "AuthorizeLaunch",
        "SetPolicy",
        "PublishResult",
        "AssignFacility",
        "CancelMission",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_order_kind_has_a_stable_name() {
        let hire = OrderKind::Hire {
            role: Role::Scientist,
            salary: Money::from_credits(1_200),
        };
        assert_eq!(hire.kind_name(), "Hire");
    }

    #[test]
    fn an_order_kind_round_trips_through_serde() {
        let kind = OrderKind::Assign {
            person: PersonId(3),
            to: Assignment::Unassigned,
        };
        let json = serde_json::to_string(&kind).expect("order kinds serialise");
        let restored: OrderKind = serde_json::from_str(&json).expect("order kinds deserialise");
        assert_eq!(restored, kind);
    }
}
