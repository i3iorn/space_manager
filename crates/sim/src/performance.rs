//! The C+B performance model.
//!
//! Diminishing-returns arithmetic with every factor clamped above zero and
//! saturating toward its maximum, and a *binding factor* naming the constraint
//! that costs the most. See `docs/MANAGEABLE_PARTS.md` §5.3.
//!
//! # The shape of the computation
//!
//! ```text
//! skill_fit = skill × role_fit
//! drive     = skill_fit × morale
//! condition = drive × health × fatigue
//! effective = condition × support × tools × creativity
//! ```
//!
//! Each stage is clamped to `[STAGE_FLOOR_BP, STAGE_CEILING_BP]`, so no single
//! stage can collapse the team and no single stage can make it unbounded. All
//! arithmetic is integer, in basis points. **No floating point takes part in a
//! decision.**

use domain::performance::{
    FULL_SCALE_BP, Modifier, ModifierKind, PerformanceInput, PerformanceResult, STAGE_CEILING_BP,
    STAGE_FLOOR_BP,
};

use crate::balance;

/// The cost at which a modifier is worth naming as the constraint.
///
/// A cost below this is not worth the player's attention, and naming it would
/// be noise.
pub const BINDING_FACTOR_THRESHOLD_BP: u32 = 1_000;

/// Computes a person's effective performance.
///
/// The result always carries the decomposition, because the player is shown
/// *which* factor is holding the team back, never just a number.
#[must_use]
pub fn compute(input: &PerformanceInput) -> PerformanceResult {
    let modifiers = input.modifiers();
    let binding_factor = binding_factor(&modifiers);

    // The composite is a **product**, so everything bad compounds — one
    // exhausted person really does slow the whole team. The product is taken
    // in u128 so that no stage has to be clamped, because clamping an
    // intermediate stage would saturate it and destroy the very differences
    // the player is being asked to read.
    //
    // The product is then divided by the reference composite, so that a team of
    // competent, rested people reads as 100%.
    let effective = effective_bp(&composite_of(input));

    PerformanceResult {
        effective,
        modifiers,
        binding_factor,
    }
}

/// The product of the eight factors, in u128.
///
/// The product is left unscaled: it is the raw product of eight basis-point
/// factors, so it lives in the tens of billions of billions.
fn composite_of(input: &PerformanceInput) -> u128 {
    let mut product: u128 = 1;
    for modifier in [
        input.skill,
        input.role_fit,
        input.morale,
        input.support,
        input.tools,
        input.fatigue,
        input.creativity,
        input.health,
    ] {
        product *= u128::from(balance::modifier_factor_bp(modifier.value()));
    }
    product
}

/// The composite as a percentage of the reference composite.
fn effective_bp(composite: &u128) -> u32 {
    let effective = composite * u128::from(FULL_SCALE_BP) / balance::REFERENCE_COMPOSITE;
    effective.clamp(u128::from(STAGE_FLOOR_BP), u128::from(STAGE_CEILING_BP)) as u32
}

/// The modifier costing the most, if any is costing enough to matter.
fn binding_factor(modifiers: &[Modifier]) -> Option<ModifierKind> {
    let mut worst: Option<(ModifierKind, u32)> = None;
    for modifier in modifiers {
        let cost = modifier.cost_bp();
        if cost < BINDING_FACTOR_THRESHOLD_BP {
            continue;
        }
        match worst {
            Some((_, worst_cost)) if worst_cost >= cost => {}
            _ => worst = Some((modifier.kind(), cost)),
        }
    }
    worst.map(|(kind, _)| kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> PerformanceInput {
        PerformanceInput::neutral()
    }

    #[test]
    fn a_good_team_performs_at_about_one_hundred_percent() {
        // The reference composite is exactly this team's factors, so a good
        // team reads as 100%.
        let mut good = input();
        good.skill = Modifier::new(ModifierKind::Skill, 13);
        good.role_fit = Modifier::new(ModifierKind::RoleFit, 10);
        good.morale = Modifier::new(ModifierKind::Morale, 10);
        good.support = Modifier::new(ModifierKind::Support, 10);
        good.tools = Modifier::new(ModifierKind::Tools, 5);
        good.fatigue = Modifier::new(ModifierKind::Fatigue, 0);
        good.creativity = Modifier::new(ModifierKind::Creativity, 13);
        good.health = Modifier::new(ModifierKind::Health, 15);

        let result = compute(&good);
        assert!(
            (i64::from(result.effective) - 10_000).abs() < 100,
            "a good team should read as 100%, but read as {}",
            result.percent()
        );
    }

    #[test]
    fn a_person_with_no_advantages_is_well_below_a_good_team() {
        // Every modifier at zero means nobody is better or worse than the
        // baseline in any dimension — which is a mediocre team, not a good one.
        let result = compute(&input());
        assert!(
            result.effective < 10_000,
            "a team with no advantages should read below 100%, but read as {}",
            result.percent()
        );
    }

    #[test]
    fn no_single_factor_can_collapse_a_team() {
        let mut worst = input();
        worst.fatigue = Modifier::new(ModifierKind::Fatigue, -100);
        let result = compute(&worst);
        assert!(result.effective >= STAGE_FLOOR_BP);
        assert_ne!(result.effective, 0);
    }

    #[test]
    fn fatigue_is_the_binding_factor_when_a_person_is_exhausted() {
        let mut exhausted = input();
        exhausted.fatigue = Modifier::new(ModifierKind::Fatigue, -80);
        exhausted.tools = Modifier::new(ModifierKind::Tools, -10);
        let result = compute(&exhausted);
        assert_eq!(result.binding_factor, Some(ModifierKind::Fatigue));
        assert!(result.constraint_sentence().contains("FATIGUE"));
    }

    #[test]
    fn a_healthy_team_has_no_constraint() {
        let mut content = input();
        content.morale = Modifier::new(ModifierKind::Morale, 40);
        content.tools = Modifier::new(ModifierKind::Tools, 20);
        let result = compute(&content);
        assert_eq!(result.binding_factor, None);
        assert_eq!(result.constraint_sentence(), "No significant constraint");
    }

    #[test]
    fn better_people_perform_better() {
        let mut good = input();
        good.skill = Modifier::new(ModifierKind::Skill, 40);
        let mut poor = input();
        poor.skill = Modifier::new(ModifierKind::Skill, -40);
        assert!(compute(&good).effective > compute(&poor).effective);
    }

    #[test]
    fn the_result_always_carries_the_decomposition() {
        let result = compute(&input());
        assert_eq!(result.modifiers.len(), 8);
    }

    #[test]
    fn the_computation_is_deterministic() {
        let first = compute(&input());
        let second = compute(&input());
        assert_eq!(first, second);
    }
}
