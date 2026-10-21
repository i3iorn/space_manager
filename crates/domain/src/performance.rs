//! The inputs to the performance model.
//!
//! The C+B hybrid: a smooth, clamped, saturating computation whose result is
//! decomposed into named modifiers and a *binding factor* for the player.
//! This module defines the modifier vocabulary and the result type; `sim`
//! computes with them. See `docs/DOMAIN_MODEL.md` §6 and
//! `docs/MANAGEABLE_PARTS.md` §5.3.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The lowest a single stage of the computation may fall to.
///
/// This is the "clamped above zero" rule. A team with one exhausted member is
/// *very bad*, not *non-existent*, and the player is never shown a hard zero
/// they cannot reason about.
pub const STAGE_FLOOR_BP: u32 = 2_500;

/// The highest a single stage of the computation may rise to.
pub const STAGE_CEILING_BP: u32 = 20_000;

/// One full scale: 10,000 basis points is 100%.
pub const FULL_SCALE_BP: u32 = 10_000;

/// Which part of a person's working life a modifier describes.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ModifierKind {
    /// The person's raw ability at the work.
    Skill,
    /// How well the work suits the person's specialisms.
    RoleFit,
    /// Morale and personal motivation.
    Morale,
    /// The team around them: cohesion, the lead, their colleagues.
    Support,
    /// The quality of the tools and facilities they have.
    Tools,
    /// Tiredness. Never positive.
    Fatigue,
    /// Originality, and the ability to find a way round a problem.
    Creativity,
    /// Physical and mental condition.
    Health,
}

impl ModifierKind {
    /// Every modifier, in the order the interface shows them.
    pub const ALL: [Self; 8] = [
        Self::Skill,
        Self::RoleFit,
        Self::Morale,
        Self::Support,
        Self::Tools,
        Self::Fatigue,
        Self::Creativity,
        Self::Health,
    ];

    /// The short name the interface uses.
    ///
    /// The phrase appears in several places — the staff list, the programme
    /// screen, the month report — and it must read identically every time so
    /// the player learns to recognise it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Skill => "SKILL",
            Self::RoleFit => "ROLE FIT",
            Self::Morale => "MORALE",
            Self::Support => "SUPPORT",
            Self::Tools => "TOOL QUALITY",
            Self::Fatigue => "FATIGUE",
            Self::Creativity => "CREATIVITY",
            Self::Health => "HEALTH",
        }
    }

    /// Whether this modifier can only ever hurt.
    #[must_use]
    pub const fn is_penalty_only(self) -> bool {
        matches!(self, Self::Fatigue)
    }
}

impl fmt::Display for ModifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// One modifier's contribution, in the range -100 to +100.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Modifier {
    kind: ModifierKind,
    value: i16,
}

impl Modifier {
    /// The largest value a modifier can take.
    pub const MAX: i16 = 100;

    /// The smallest value a modifier can take.
    pub const MIN: i16 = -100;

    /// A modifier, clamped into the permitted range.
    #[must_use]
    pub fn new(kind: ModifierKind, value: i16) -> Self {
        Self {
            kind,
            value: value.clamp(Self::MIN, Self::MAX),
        }
    }

    /// Which part of working life this modifier describes.
    #[must_use]
    pub const fn kind(self) -> ModifierKind {
        self.kind
    }

    /// The modifier's value.
    #[must_use]
    pub const fn value(self) -> i16 {
        self.value
    }

    /// The cost this modifier imposes, in basis points.
    ///
    /// A modifier that helps costs nothing. A modifier that hinders costs the
    /// distance from neutral it pushes the result. **The binding factor is the
    /// modifier with the largest cost** — the single constraint doing the most
    /// damage.
    #[must_use]
    pub const fn cost_bp(self) -> u32 {
        if self.value >= 0 {
            0
        } else {
            let penalty = (-self.value) as u32;
            penalty * (FULL_SCALE_BP - penalty) / 100
        }
    }
}

/// The eight modifiers that make up one person's working performance.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PerformanceInput {
    /// The person's raw ability at the work.
    pub skill: Modifier,
    /// How well the work suits the person's specialisms.
    pub role_fit: Modifier,
    /// Morale and personal motivation.
    pub morale: Modifier,
    /// The team around them.
    pub support: Modifier,
    /// The quality of their tools and facilities.
    pub tools: Modifier,
    /// Tiredness.
    pub fatigue: Modifier,
    /// Originality.
    pub creativity: Modifier,
    /// Physical and mental condition.
    pub health: Modifier,
}

impl PerformanceInput {
    /// A neutral set of modifiers, for a person with no context yet.
    #[must_use]
    pub fn neutral() -> Self {
        Self {
            skill: Modifier::new(ModifierKind::Skill, 0),
            role_fit: Modifier::new(ModifierKind::RoleFit, 0),
            morale: Modifier::new(ModifierKind::Morale, 0),
            support: Modifier::new(ModifierKind::Support, 0),
            tools: Modifier::new(ModifierKind::Tools, 0),
            fatigue: Modifier::new(ModifierKind::Fatigue, 0),
            creativity: Modifier::new(ModifierKind::Creativity, 0),
            health: Modifier::new(ModifierKind::Health, 0),
        }
    }

    /// Every modifier, in display order.
    #[must_use]
    pub fn modifiers(&self) -> Vec<Modifier> {
        vec![
            self.skill,
            self.role_fit,
            self.morale,
            self.support,
            self.tools,
            self.fatigue,
            self.creativity,
            self.health,
        ]
    }
}

/// The result of the performance computation.
///
/// This is what the interface renders, and it always carries the decomposition —
/// the player is shown *which* factor is holding the team back, never just a
/// number.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PerformanceResult {
    /// The effective performance, in basis points. 10,000 is 100%.
    pub effective: u32,
    /// Every modifier, in display order.
    pub modifiers: Vec<Modifier>,
    /// The single constraint costing the most, if any is costing enough to
    /// matter.
    pub binding_factor: Option<ModifierKind>,
}

impl PerformanceResult {
    /// The effective performance as a whole percentage.
    #[must_use]
    pub fn percent(&self) -> u32 {
        self.effective * 100 / FULL_SCALE_BP
    }

    /// A sentence naming the constraint, for the interface.
    #[must_use]
    pub fn constraint_sentence(&self) -> String {
        match self.binding_factor {
            Some(kind) => format!("Held back by {}", kind.label()),
            None => "No significant constraint".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_modifier_is_clamped_into_range() {
        assert_eq!(Modifier::new(ModifierKind::Skill, 500).value(), 100);
        assert_eq!(Modifier::new(ModifierKind::Skill, -500).value(), -100);
    }

    #[test]
    fn a_helpful_modifier_costs_nothing() {
        assert_eq!(Modifier::new(ModifierKind::Skill, 60).cost_bp(), 0);
    }

    #[test]
    fn a_hindering_modifier_costs_more_the_worse_it_gets() {
        let mild = Modifier::new(ModifierKind::Fatigue, -20).cost_bp();
        let heavy = Modifier::new(ModifierKind::Fatigue, -80).cost_bp();
        assert!(heavy > mild);
        assert!(mild > 0);
    }

    #[test]
    fn the_cost_curve_saturates() {
        // A modifier of -50 costs more than a modifier of -20, but not twice as
        // much: the curve flattens, which is what stops one bad factor from
        // dominating the display.
        let twenty = Modifier::new(ModifierKind::Fatigue, -20).cost_bp();
        let fifty = Modifier::new(ModifierKind::Fatigue, -50).cost_bp();
        assert!(fifty > twenty);
        assert!(fifty < twenty * 3);
    }

    #[test]
    fn every_modifier_has_a_distinct_label() {
        let mut labels: Vec<&str> = ModifierKind::ALL.iter().map(|k| k.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), ModifierKind::ALL.len());
    }

    #[test]
    fn only_fatigue_is_penalty_only() {
        for kind in ModifierKind::ALL {
            assert_eq!(kind.is_penalty_only(), kind == ModifierKind::Fatigue);
        }
    }
}
