//! Event definitions.
//!
//! Events are *data*, not code: preconditions, weight modifiers, and effects
//! expressed as deltas. An event can therefore never corrupt an invariant and
//! can never loop. See `docs/SIMULATION.md` §5.2.

use serde::{Deserialize, Serialize};

use crate::chronicle::{Category, Severity};
use crate::ids::{EventId, FacilityId, PersonId, ProgrammeId, ResearchProjectId};
use crate::money::Money;
use crate::time::Instant;

/// A condition on the state, checked before an event may fire.
///
/// Conditions are pure predicates. They never change anything.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Condition {
    /// The campaign has run for at least this long.
    MonthAtLeast(Instant),
    /// The treasury is below this.
    TreasuryBelow(Money),
    /// The treasury is above this.
    TreasuryAbove(Money),
    /// The state's approval is below this.
    ApprovalBelow(i16),
    /// The state's approval is above this.
    ApprovalAbove(i16),
    /// A programme exists.
    HasProgramme(ProgrammeId),
    /// No programme exists with this title.
    NoProgrammeNamed(String),
    /// A person exists and is tired.
    PersonFatigueAbove(PersonId, u8),
    /// A facility exists and is poorly maintained.
    FacilityConditionBelow(FacilityId, u8),
    /// A research project exists.
    HasProject(ResearchProjectId),
    /// A capability has been unlocked.
    HasCapability(u8),
    /// A capability has not been unlocked.
    LacksCapability(u8),
    /// The agency's average fatigue is above this.
    ///
    /// **The average, not the worst.** A tired agency is a tired agency, and the
    /// condition is the one the player caused by working everybody too hard.
    AverageFatigueAbove(u8),
    /// The treasury has less than this many months of runway.
    RunwayBelowMonths(u32),
    /// The public's opinion is below this.
    PublicOpinionBelow(i16),
    /// A programme has no slack left.
    ProgrammeWithoutSlack,
    /// A milestone falls due within this many months.
    MilestoneDueWithinMonths(u32),
    /// A mission's risk of failure is above this percent.
    LaunchRiskAbove(u8),
    /// A facility has nobody working in it.
    FacilityUnstaffed,
}

/// A modifier on an event's weight.
///
/// The design rule: **bad conditions raise the odds of bad events, and good
/// conditions raise the odds of good ones.** This is what makes the simulation
/// feel fair rather than random.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct WeightMod {
    /// The condition that must hold for the modifier to apply.
    pub when: Condition,
    /// How much to multiply the base weight by, in percent.
    pub percent: u32,
}

/// A change an event makes to the world.
///
/// Effects are deltas, never closures. An event cannot corrupt an invariant
/// because it cannot express an arbitrary mutation.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Effect {
    /// A person's morale moves.
    AdjustMorale {
        /// The person.
        person: PersonId,
        /// By how much.
        delta: i16,
    },
    /// A person's loyalty moves.
    AdjustLoyalty {
        /// The person.
        person: PersonId,
        /// By how much.
        delta: i16,
    },
    /// A person's health moves.
    AdjustHealth {
        /// The person.
        person: PersonId,
        /// By how much.
        delta: i16,
    },
    /// The treasury moves.
    AdjustTreasury {
        /// By how much.
        delta: Money,
    },
    /// The state's approval moves.
    AdjustApproval {
        /// By how much.
        delta: i16,
    },
    /// The public's opinion moves.
    AdjustPublicOpinion {
        /// By how much.
        delta: i16,
    },
    /// The agency's prestige moves.
    AdjustPrestige {
        /// By how much.
        delta: i16,
    },
    /// A research project advances.
    AdvanceProject {
        /// The project.
        project: ResearchProjectId,
        /// By how many points.
        points: u32,
    },
    /// A programme's objective advances.
    AdvanceObjective {
        /// The programme.
        programme: ProgrammeId,
        /// By how many points.
        points: u32,
    },
    /// A facility's condition moves.
    AdjustFacilityCondition {
        /// The facility.
        facility: FacilityId,
        /// By how much.
        delta: i16,
    },
    /// Everybody's morale moves.
    ///
    /// **The catalogue cannot name a person**, because the people in the
    /// catalogue do not exist yet: a condition is checked at the moment the
    /// event fires, and an effect that lands on everybody is the honest
    /// version of "the whole agency felt it".
    AdjustAllMorale {
        /// By how much.
        delta: i16,
    },
    /// Everybody's loyalty moves.
    AdjustAllLoyalty {
        /// By how much.
        delta: i16,
    },
    /// Everybody's fatigue moves.
    AdjustAllFatigue {
        /// By how much.
        delta: i16,
    },
    /// Everybody's health moves.
    AdjustAllHealth {
        /// By how much.
        delta: i16,
    },
    /// Every facility's condition moves.
    AdjustAllCondition {
        /// By how much.
        delta: i16,
    },
    /// A person leaves the agency.
    PersonLeaves {
        /// The person.
        person: PersonId,
        /// Why, in the player's language.
        reason: String,
    },
}

/// The words an event uses.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct NarrativeSpec {
    /// One line.
    pub headline: String,
    /// Optional detail.
    pub body: Option<String>,
    /// How much it matters.
    pub severity: Severity,
    /// The chronicle category it is filed under.
    pub category: Category,
}

/// One of the choices an event puts to the player.
///
/// **A decision is data, never a closure**, for the same reason an effect is:
/// the catalogue is static and the state it names may not exist yet, so an
/// option can only change the world through the same deltas an event uses.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct DecisionOption {
    /// The option's identifier, stable and comparable.
    pub id: String,
    /// What the player reads, in the player's language.
    pub label: String,
    /// What choosing it does.
    pub effects: Vec<Effect>,
}

/// A choice an event puts to the player, with a window to answer in.
///
/// **An event that asks nothing is a sentence; an event that asks something is
/// a decision.** The decision carries the window — how long the player has to
/// answer — and what happens when nobody does. It is raised by the event that
/// carries it, held by the simulation until the player answers or the window
/// closes, and whichever happens first decides which effects are applied.
///
/// The decision's own effects are **not** the event's effects. The event's
/// effects are the baseline — what already happened — and the decision's
/// options and expiry are what the player's answer, or silence, adds.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Decision {
    /// The headline the player reads, in the player's language.
    pub headline: String,
    /// The detail, if there is one.
    pub body: Option<String>,
    /// How many days the player has to answer.
    pub days_to_decide: u32,
    /// What happens when the window closes and nobody has answered.
    pub expiry_effects: Vec<Effect>,
    /// The choices, in the order they are shown.
    pub options: Vec<DecisionOption>,
}

/// Something an event causes later.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct FollowUp {
    /// The event to schedule.
    pub event: EventId,
    /// How many months later.
    pub in_months: u32,
    /// A condition that must hold when it comes due.
    pub condition: Option<Condition>,
}

/// An event, defined entirely as data.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct EventDef {
    /// The event's identifier.
    pub id: EventId,
    /// What kind of event it is.
    pub category: Category,
    /// How likely it is to be drawn, before context is taken into account.
    pub base_weight: u32,
    /// What must be true for it to be eligible at all.
    pub preconditions: Vec<Condition>,
    /// How the context changes its weight.
    pub weight_modifiers: Vec<WeightMod>,
    /// What it changes.
    pub effects: Vec<Effect>,
    /// What it says.
    pub narrative: NarrativeSpec,
    /// What it causes later.
    pub follow_ups: Vec<FollowUp>,
}

impl EventDef {
    /// A new event definition.
    #[must_use]
    pub fn new(
        id: EventId,
        category: Category,
        base_weight: u32,
        narrative: NarrativeSpec,
    ) -> Self {
        Self {
            id,
            category,
            base_weight,
            preconditions: Vec::new(),
            weight_modifiers: Vec::new(),
            effects: Vec::new(),
            narrative,
            follow_ups: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_event_carries_its_narrative_and_weight() {
        let event = EventDef::new(
            EventId(1),
            Category::Technical,
            100,
            NarrativeSpec {
                headline: "A stage failed".to_owned(),
                body: None,
                severity: Severity::Major,
                category: Category::Technical,
            },
        );
        assert_eq!(event.base_weight, 100);
        assert!(event.effects.is_empty());
    }

    #[test]
    fn effects_are_deltas_and_not_closures() {
        // The point of the check is that `Effect` is a plain data enum, so it
        // can be serialised and applied atomically. A variant that carried a
        // closure would break that.
        let effect = Effect::AdjustApproval { delta: -5 };
        let json = serde_json::to_string(&effect).expect("effects serialise");
        assert!(json.contains("AdjustApproval"));
    }
}
