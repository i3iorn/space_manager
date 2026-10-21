//! Facilities and their construction.
//!
//! Every facility is built, never abstract capacity. A facility's specification
//! is computed once, at completion, from the construction choices, and then
//! fixed — the player chose a method forty years ago and still lives with it.
//! See `docs/DOMAIN_MODEL.md` §9.

use serde::{Deserialize, Serialize};

use crate::ids::{FacilityId, PersonId};
use crate::money::Money;
use crate::research::Progress;
use crate::time::Instant;

/// What a facility is for.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum FacilityType {
    /// Where research happens.
    Laboratory,
    /// Where existing technology is tested.
    TestStand,
    /// Where hardware is built.
    FabricationPlant,
    /// Where vehicles leave the ground.
    LaunchComplex,
}

impl FacilityType {
    /// Every facility type.
    pub const ALL: [Self; 4] = [
        Self::Laboratory,
        Self::TestStand,
        Self::FabricationPlant,
        Self::LaunchComplex,
    ];

    /// The type's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Laboratory => "Laboratory",
            Self::TestStand => "Test stand",
            Self::FabricationPlant => "Fabrication plant",
            Self::LaunchComplex => "Launch complex",
        }
    }
}

/// How a facility was built.
///
/// **Robust** builds are slow and expensive but do not fail catastrophically.
/// **Fast and cheap** builds are 30% cheaper and 35% quicker, and cost 15
/// reliability. Over a forty-year campaign that penalty compounds into the
/// difference between a facility that still works and one that has been rebuilt
/// twice.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum BuildMethod {
    /// Quick and cheap, and less reliable.
    FastCheap,
    /// The ordinary way.
    Standard,
    /// Slow, expensive, and dependable.
    Robust,
    /// Cheap and quick, and may unlock capability nobody else has.
    Experimental,
}

impl BuildMethod {
    /// Every method.
    pub const ALL: [Self; 4] = [
        Self::FastCheap,
        Self::Standard,
        Self::Robust,
        Self::Experimental,
    ];

    /// The method's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FastCheap => "Fast & cheap",
            Self::Standard => "Standard",
            Self::Robust => "Robust",
            Self::Experimental => "Experimental",
        }
    }

    /// The multiplier on build cost.
    #[must_use]
    pub const fn cost_multiplier_bp(self) -> u32 {
        match self {
            Self::FastCheap => 7_000,
            Self::Standard => 10_000,
            Self::Robust => 14_500,
            Self::Experimental => 8_500,
        }
    }

    /// The multiplier on build time.
    #[must_use]
    pub const fn time_multiplier_bp(self) -> u32 {
        match self {
            Self::FastCheap => 6_500,
            Self::Standard => 10_000,
            Self::Robust => 13_500,
            Self::Experimental => 8_500,
        }
    }

    /// The multiplier on how much work the facility can take on.
    ///
    /// The method barely changes the capacity; it changes how dependable the
    /// facility is and how much it costs to run.
    #[must_use]
    pub const fn capacity_multiplier_bp(self) -> u32 {
        match self {
            Self::FastCheap => 9_000,
            Self::Standard | Self::Experimental => 10_000,
            Self::Robust => 11_000,
        }
    }

    /// The multiplier on running costs.
    #[must_use]
    pub const fn upkeep_multiplier_bp(self) -> u32 {
        match self {
            Self::FastCheap => 7_500,
            Self::Standard => 10_000,
            Self::Robust => 13_000,
            Self::Experimental => 11_500,
        }
    }

    /// The change in reliability, in percentage points.
    #[must_use]
    pub const fn reliability_delta(self) -> i16 {
        match self {
            Self::FastCheap => -15,
            Self::Standard => 0,
            Self::Robust => 12,
            Self::Experimental => -5,
        }
    }
}

/// How big a facility is.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum BuildScale {
    /// A corner of the agency.
    Small,
    /// An ordinary facility.
    Standard,
    /// A large installation.
    Large,
    /// Something the whole agency is built around.
    Oversized,
}

impl BuildScale {
    /// Every scale.
    pub const ALL: [Self; 4] = [Self::Small, Self::Standard, Self::Large, Self::Oversized];

    /// The scale's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Standard => "Standard",
            Self::Large => "Large",
            Self::Oversized => "Oversized",
        }
    }

    /// The multiplier on build cost.
    #[must_use]
    pub const fn cost_multiplier_bp(self) -> u32 {
        match self {
            Self::Small => 7_000,
            Self::Standard => 10_000,
            Self::Large => 19_000,
            Self::Oversized => 28_000,
        }
    }

    /// The multiplier on running costs.
    #[must_use]
    pub const fn upkeep_multiplier_bp(self) -> u32 {
        match self {
            Self::Small => 6_000,
            Self::Standard => 10_000,
            Self::Large => 22_000,
            Self::Oversized => 36_000,
        }
    }

    /// The multiplier on how much work the facility can take on.
    #[must_use]
    pub const fn capacity_multiplier_bp(self) -> u32 {
        match self {
            Self::Small => 5_000,
            Self::Standard => 10_000,
            Self::Large => 20_000,
            Self::Oversized => 30_000,
        }
    }
}

/// How a facility was paid for.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Financing {
    /// The agency's own budget. No strings attached.
    StateBudget,
    /// Someone else's money, with conditions.
    PrivateInvestment,
    /// A partner with its own claims on the output.
    CommercialPartner,
}

/// Where a facility stands.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Site {
    /// The site's name.
    pub name: String,
    /// Whether the site is visible to the public.
    pub publicly_visible: bool,
}

/// What a finished facility can do and how well.
///
/// Computed once, at completion, and then fixed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Specification {
    /// How much work it can take on at once.
    pub capacity: i64,
    /// How dependable it is, from 0 to 100.
    pub reliability: u8,
    /// How automated it is, from 0 to 100.
    pub automation: u8,
    /// How many people it needs.
    pub staff_needed: i64,
    /// The quality of what it produces, from 0 to 100.
    pub quality: u8,
}

impl Specification {
    /// A specification of nothing, for a facility still being built.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            capacity: 0,
            reliability: 0,
            automation: 0,
            staff_needed: 0,
            quality: 0,
        }
    }
}

/// What a facility was built to be.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ConstructionPlan {
    /// How it was built.
    pub method: BuildMethod,
    /// How big it is.
    pub scale: BuildScale,
    /// Where it stands.
    pub site: Site,
    /// How it was paid for.
    pub financing: Financing,
    /// What it cost.
    pub total_cost: Money,
    /// How long it will take.
    pub build_months: u32,
}

/// A facility under construction.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Construction {
    /// What it is being built to be.
    pub plan: ConstructionPlan,
    /// When building started.
    pub started: Instant,
    /// How far along it is.
    pub progress: Progress,
    /// When it is expected to finish. An estimate, never a promise.
    pub expected_completion: Option<Instant>,
    /// The fractional remainder carried between rate applications.
    pub remainders: crate::scale::Remainders,
}

/// A facility the agency owns.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Facility {
    /// The facility's identifier.
    pub id: FacilityId,
    /// Its name.
    pub name: String,
    /// What it is for.
    pub kind: FacilityType,
    /// Where it stands.
    pub site: Site,
    /// What it can do.
    pub specification: Specification,
    /// How well it is maintained, from `0` to `25_600`.
    ///
    /// **Scaled, like every other percentage in the domain.** A condition
    /// that falls by a fraction of a percent a day has to be able to hold that
    /// fraction, or the wear is lost to a rounding error.
    pub condition: u16,
    /// Who works there.
    pub staff: Vec<PersonId>,
    /// The build, if it is still under construction.
    pub construction: Option<Construction>,
    /// What it costs to run each month.
    pub upkeep: Money,
    /// What it produces each month.
    pub monthly_output: i64,
    /// The fractional remainders carried between rate applications.
    pub remainders: crate::scale::Remainders,
}

impl Facility {
    /// Whether the facility is finished and in service.
    #[must_use]
    pub const fn is_operational(&self) -> bool {
        self.construction.is_none()
    }

    /// Whether the facility is staffed well enough to run at specification.
    #[must_use]
    pub fn is_fully_staffed(&self) -> bool {
        i64::try_from(self.staff.len()).unwrap_or(i64::MAX) >= self.specification.staff_needed
    }

    /// The condition as a whole percentage.
    #[must_use]
    pub fn condition_percent(&self) -> u32 {
        crate::scale::units_to_percent(u32::from(self.condition))
    }

    /// What the facility actually produces, at its current condition.
    ///
    /// **A neglected facility produces less.** The specification is what it
    /// was built to be; the condition is what it is worth today.
    #[must_use]
    pub fn effective_output(&self) -> i64 {
        self.monthly_output * i64::from(self.condition) / i64::from(crate::scale::FULL_PERCENT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_method_has_a_cost_and_time_multiplier() {
        for method in BuildMethod::ALL {
            assert!(method.cost_multiplier_bp() > 0);
            assert!(method.time_multiplier_bp() > 0);
            assert!(method.upkeep_multiplier_bp() > 0);
        }
    }

    #[test]
    fn fast_and_cheap_is_cheaper_and_quicker_than_robust() {
        assert!(
            BuildMethod::FastCheap.cost_multiplier_bp() < BuildMethod::Robust.cost_multiplier_bp()
        );
        assert!(
            BuildMethod::FastCheap.time_multiplier_bp() < BuildMethod::Robust.time_multiplier_bp()
        );
    }

    #[test]
    fn only_robust_improves_reliability() {
        assert!(BuildMethod::Robust.reliability_delta() > 0);
        for method in [
            BuildMethod::FastCheap,
            BuildMethod::Standard,
            BuildMethod::Experimental,
        ] {
            assert!(method.reliability_delta() <= 0);
        }
    }

    #[test]
    fn bigger_costs_more_to_build_and_to_run() {
        let costs: Vec<u32> = BuildScale::ALL
            .iter()
            .map(|s| s.cost_multiplier_bp())
            .collect();
        let mut sorted = costs.clone();
        sorted.sort_unstable();
        assert_eq!(costs, sorted);
    }
}
