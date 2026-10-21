//! Vehicles: archetypes and subsystem sliders.
//!
//! All nine sliders exist in the type from the start, so exposing more of them
//! later is a UI change rather than a data-model change. Derived statistics are
//! computed from the sliders and the available capabilities; they are never
//! stored as an authority. See `docs/DOMAIN_MODEL.md` §10.

use serde::{Deserialize, Serialize};

use crate::ids::{MissionId, VehicleId};
use crate::money::Money;
use crate::research::CapabilityId;

/// The identifier of a vehicle archetype.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ArchetypeId(pub u8);

/// One of the vehicle's subsystem sliders.
///
/// **The slider is the player's only real decision about a vehicle**, and it is
/// a decision about where the money goes: every one of them can be turned up
/// and every one of them costs.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Slider {
    /// Thrust against efficiency.
    Propulsion,
    /// What the vehicle can carry.
    Payload,
    /// How long the crew can survive.
    LifeSupport,
    /// How much can fail without sinking the vehicle.
    Redundancy,
    /// How the vehicle makes and stores power.
    Power,
    /// How strong it is against what it hits.
    Structure,
    /// How capable its guidance and control systems are.
    Avionics,
    /// How well it survives re-entry.
    HeatShield,
}

impl Slider {
    /// Every slider, in display order.
    pub const ALL: [Self; 8] = [
        Self::Propulsion,
        Self::Payload,
        Self::LifeSupport,
        Self::Redundancy,
        Self::Power,
        Self::Structure,
        Self::Avionics,
        Self::HeatShield,
    ];

    /// The slider's name, as the player reads it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Propulsion => "Propulsion",
            Self::Payload => "Payload",
            Self::LifeSupport => "Life support",
            Self::Redundancy => "Redundancy",
            Self::Power => "Power",
            Self::Structure => "Structure",
            Self::Avionics => "Avionics",
            Self::HeatShield => "Heat shield",
        }
    }

    /// The slider's value.
    #[must_use]
    pub const fn value(self, sliders: SliderValues) -> u8 {
        match self {
            Self::Propulsion => sliders.propulsion,
            Self::Payload => sliders.payload,
            Self::LifeSupport => sliders.life_support,
            Self::Redundancy => sliders.redundancy,
            Self::Power => sliders.power,
            Self::Structure => sliders.structure,
            Self::Avionics => sliders.avionics,
            Self::HeatShield => sliders.heat_shield,
        }
    }

    /// Sets the slider's value.
    pub const fn set(self, sliders: &mut SliderValues, value: u8) {
        match self {
            Self::Propulsion => sliders.propulsion = value,
            Self::Payload => sliders.payload = value,
            Self::LifeSupport => sliders.life_support = value,
            Self::Redundancy => sliders.redundancy = value,
            Self::Power => sliders.power = value,
            Self::Structure => sliders.structure = value,
            Self::Avionics => sliders.avionics = value,
            Self::HeatShield => sliders.heat_shield = value,
        }
    }

    /// The slider an unlock names.
    ///
    /// **The capability list is data and the sliders are a type**, so the
    /// mapping between them lives here. An unlock that names no slider unlocks
    /// nothing, which is a data error rather than a crash.
    #[must_use]
    pub fn from_unlock(name: &str) -> Option<Self> {
        match name.trim().to_lowercase().as_str() {
            "propulsion" | "orbital launch" => Some(Self::Propulsion),
            "payload" => Some(Self::Payload),
            "life support" | "life_support" => Some(Self::LifeSupport),
            "redundancy" => Some(Self::Redundancy),
            "power" => Some(Self::Power),
            "structure" => Some(Self::Structure),
            "avionics" => Some(Self::Avionics),
            "heat shield" | "heat_shield" => Some(Self::HeatShield),
            _ => None,
        }
    }
}

/// The player's choices for a vehicle's subsystems.
///
/// Each slider is 0-100. A slider is bounded by the capabilities the agency has
/// developed, so a player cannot buy a capability they have not researched.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SliderValues {
    /// Thrust against efficiency.
    pub propulsion: u8,
    /// What the vehicle can carry.
    pub payload: u8,
    /// How long the crew can survive.
    pub life_support: u8,
    /// How much can fail without sinking the vehicle.
    pub redundancy: u8,
    /// How the vehicle makes and stores power.
    pub power: u8,
    /// How strong it is against what it hits.
    pub structure: u8,
    /// How capable its guidance and control systems are.
    pub avionics: u8,
    /// How well it survives re-entry.
    pub heat_shield: u8,
}

impl SliderValues {
    /// Every slider at zero.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            propulsion: 0,
            payload: 0,
            life_support: 0,
            redundancy: 0,
            power: 0,
            structure: 0,
            avionics: 0,
            heat_shield: 0,
        }
    }

    /// Every slider, in display order.
    #[must_use]
    pub const fn as_list(self) -> [(&'static str, u8); 8] {
        [
            ("Propulsion", self.propulsion),
            ("Payload", self.payload),
            ("Life support", self.life_support),
            ("Redundancy", self.redundancy),
            ("Power", self.power),
            ("Structure", self.structure),
            ("Avionics", self.avionics),
            ("Heat shield", self.heat_shield),
        ]
    }

    /// The values, in slider order.
    #[must_use]
    pub const fn in_order(self) -> [u8; 8] {
        [
            self.propulsion,
            self.payload,
            self.life_support,
            self.redundancy,
            self.power,
            self.structure,
            self.avionics,
            self.heat_shield,
        ]
    }

    /// Builds the values from an array in slider order.
    #[must_use]
    pub const fn from_order(values: [u8; 8]) -> Self {
        Self {
            propulsion: values[0],
            payload: values[1],
            life_support: values[2],
            redundancy: values[3],
            power: values[4],
            structure: values[5],
            avionics: values[6],
            heat_shield: values[7],
        }
    }

    /// The values, clamped to the archetype's bands and to what the agency has
    /// developed.
    ///
    /// **A slider the agency has not researched is pinned at zero**, and a
    /// slider is never outside the archetype's band. The player chooses within
    /// what the technology allows; the technology decides the edges.
    #[must_use]
    pub fn bounded(self, archetype: ArchetypeId, unlocked: &[CapabilityId]) -> Self {
        let mut values = self;
        for slider in Slider::ALL {
            let (floor, ceiling) = archetype_band(archetype, slider);
            let available = slider_unlocked(slider, unlocked);
            let mut value = slider.value(values).min(ceiling);
            if available {
                value = value.max(floor);
            } else {
                // **The pin wins over the floor.** A capability the agency has
                // not developed is not available at all, whatever the
                // archetype would otherwise insist on.
                value = 0;
            }
            slider.set(&mut values, value);
        }
        values
    }
}

/// Whether a slider may be set at all, given the agency's capabilities.
///
/// **A player cannot buy a capability they have not researched.** A slider
/// that a capability unlocks is pinned at zero until that capability has been
/// developed, and a slider no capability names is always available — the agency
/// has always had it.
fn slider_unlocked(slider: Slider, unlocked: &[CapabilityId]) -> bool {
    let gating: Vec<&crate::research::Capability> = crate::research::CAPABILITIES
        .iter()
        .filter(|capability| {
            capability.unlocks.iter().any(|unlock| {
                matches!(
                    unlock,
                    crate::research::Unlock::VehicleSlider { slider: name }
                        if Slider::from_unlock(name) == Some(slider)
                )
            })
        })
        .collect();
    if gating.is_empty() {
        return true;
    }
    gating
        .iter()
        .any(|capability| unlocked.contains(&capability.id))
}

/// The range a slider may be set to, for an archetype.
#[must_use]
pub const fn archetype_band(archetype: ArchetypeId, slider: Slider) -> (u8, u8) {
    match archetype {
        // **A sounding rocket is small and cheap and unmanned.**
        ArchetypeId(0) => match slider {
            Slider::Propulsion => (0, 80),
            Slider::Payload => (0, 10),
            Slider::LifeSupport | Slider::HeatShield => (0, 0),
            Slider::Redundancy | Slider::Structure => (0, 30),
            Slider::Power | Slider::Avionics => (0, 40),
        },
        // **An orbital launcher is big, expensive and crewed.**
        ArchetypeId(1) => match slider {
            Slider::Propulsion | Slider::Payload | Slider::Structure => (20, 100),
            Slider::LifeSupport => (10, 80),
            Slider::Redundancy | Slider::Power => (10, 90),
            Slider::Avionics => (20, 90),
            Slider::HeatShield => (0, 60),
        },
        // **A capsule is a crewed vehicle that goes up and comes back.**
        ArchetypeId(2) => match slider {
            Slider::Propulsion | Slider::Power => (10, 70),
            Slider::Payload => (0, 30),
            Slider::LifeSupport | Slider::Redundancy | Slider::Avionics => (20, 100),
            Slider::Structure => (20, 90),
            Slider::HeatShield => (30, 100),
        },
        _ => (0, 100),
    }
}

/// The name of a vehicle archetype.
#[must_use]
pub fn archetype_name(archetype: ArchetypeId) -> &'static str {
    match archetype.0 {
        0 => "Sounding rocket",
        1 => "Orbital launcher",
        2 => "Orbital capsule",
        _ => "Unknown",
    }
}

/// Every archetype the game has.
pub const ARCHETYPES: [ArchetypeId; 3] = [ArchetypeId(0), ArchetypeId(1), ArchetypeId(2)];

/// What a vehicle's numbers work out to.
///
/// Computed from the sliders and the available capabilities, and never stored
/// as an authority.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct DerivedStats {
    /// What it can carry.
    pub payload_capacity: i64,
    /// How good it is, in an abstract unit that is not real orbital mechanics.
    pub delta_v: i64,
    /// How dependable it is, from 0 to 100.
    pub reliability: u8,
    /// What it costs.
    pub cost_estimate: Money,
    /// The capabilities it needs before it can be built.
    pub required_capabilities: Vec<CapabilityId>,
    /// How bad a failure would be, from 0 to 100.
    pub failure_severity: u8,
}

/// Computes a vehicle's numbers from its sliders.
///
/// **Every number here follows from the sliders, and nothing else.** A player
/// who turns the redundancy up gets a more reliable vehicle and a more
/// expensive one, and the two are the same decision seen from two sides.
///
/// The arithmetic is integer throughout. A vehicle's price is in whole
/// credits and its reliability is a whole percentage, and a design that
/// produced a price of 1,203.5 credits would be a design nobody could budget
/// for.
#[must_use]
pub fn derive_stats(sliders: SliderValues, archetype: ArchetypeId) -> DerivedStats {
    let values = sliders.in_order().map(u32::from);

    // **The payload is the payload slider, and the delta-v is the propulsion
    // slider**, with the structure making the structure pay for itself.
    let payload_capacity = i64::from(values[1]) * 40;
    let delta_v = i64::from(values[0]) * 12 + i64::from(values[5]) * 4;

    // **The reliability is the redundancy, the avionics and the structure**,
    // and it is the one number the player cannot buy with money alone.
    let reliability =
        u8::try_from((values[3] * 30 + values[6] * 25 + values[5] * 20) / 75).unwrap_or(100);

    // **The cost is every slider, and the expensive ones cost more.** A
    // vehicle is a sum of parts, and a player is choosing the parts.
    let cost = i64::from(values[0]) * 4_000
        + i64::from(values[1]) * 900
        + i64::from(values[2]) * 6_000
        + i64::from(values[3]) * 2_500
        + i64::from(values[4]) * 2_000
        + i64::from(values[5]) * 3_000
        + i64::from(values[6]) * 1_800
        + i64::from(values[7]) * 5_000;

    // **The archetype is a multiplier on the cost**, because an orbital
    // launcher is a different machine from a sounding rocket and not merely a
    // bigger one.
    let archetype_multiplier = match archetype {
        ArchetypeId(1) => 3,
        ArchetypeId(2) => 2,
        _ => 1,
    };

    // **The required capabilities are the ones the design leans on**: a
    // vehicle that can come back needs re-entry, and one that goes into orbit
    // needs orbital propulsion.
    let mut required_capabilities = Vec::new();
    if values[7] > 0 {
        required_capabilities.push(CapabilityId(1));
    }
    if values[0] >= 60 {
        required_capabilities.push(CapabilityId(2));
    }
    if values[6] >= 60 {
        required_capabilities.push(CapabilityId(4));
    }

    // **A failure is as bad as the weakest part**, and a vehicle with a heat
    // shield and no structure is a fireball.
    let failure_severity = u8::try_from(
        255u32
            .saturating_sub(values[5] * 2)
            .saturating_sub(values[2] * 2)
            .saturating_sub(values[3] * 2)
            / 3,
    )
    .unwrap_or(100);

    DerivedStats {
        payload_capacity,
        delta_v,
        reliability,
        cost_estimate: Money::from_credits(cost * archetype_multiplier),
        required_capabilities,
        failure_severity,
    }
}

/// How far along a vehicle is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BuildState {
    /// Designed but not started.
    Planned,
    /// Being built.
    Building,
    /// Finished and ready.
    Complete,
    /// Written off.
    Scrapped,
}

/// A vehicle the agency owns.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Vehicle {
    /// The vehicle's identifier.
    pub id: VehicleId,
    /// Its name.
    pub name: String,
    /// What kind of vehicle it is.
    pub archetype: ArchetypeId,
    /// The player's choices.
    pub sliders: SliderValues,
    /// What the choices work out to.
    pub derived: DerivedStats,
    /// What it cost.
    pub cost: Money,
    /// How far along the build is, in scaled percent.
    pub build_progress: u32,
    /// The fractional remainders carried between rate applications.
    pub remainders: crate::scale::Remainders,
    /// How far along it is.
    pub build_state: BuildState,
    /// The mission it is committed to.
    pub assigned_mission: Option<MissionId>,
}

impl Vehicle {
    /// A new, unbuilt vehicle.
    #[must_use]
    pub fn new(id: VehicleId, name: String, archetype: ArchetypeId, sliders: SliderValues) -> Self {
        let derived = derive_stats(sliders, archetype);
        Self {
            id,
            name,
            archetype,
            sliders,
            cost: derived.cost_estimate,
            build_progress: 0,
            remainders: crate::scale::Remainders::new(),
            derived,
            build_state: BuildState::Planned,
            assigned_mission: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slider_is_listed() {
        assert_eq!(SliderValues::zero().as_list().len(), 8);
    }

    #[test]
    fn a_new_vehicle_is_not_built() {
        let vehicle = Vehicle::new(
            VehicleId(0),
            "Orbital Launcher 1".to_owned(),
            ArchetypeId(1),
            SliderValues::zero(),
        );
        assert_eq!(vehicle.build_state, BuildState::Planned);
    }

    #[test]
    fn archetypes_have_names() {
        assert_eq!(archetype_name(ArchetypeId(1)), "Orbital launcher");
    }

    #[test]
    fn a_vehicle_derives_its_numbers_from_its_sliders() {
        // **The derived statistics follow from the sliders**, and nothing else.
        let plain = derive_stats(SliderValues::zero(), ArchetypeId(1));
        let fancy = derive_stats(
            SliderValues {
                propulsion: 80,
                payload: 60,
                life_support: 40,
                redundancy: 50,
                power: 30,
                structure: 70,
                avionics: 60,
                heat_shield: 20,
            },
            ArchetypeId(1),
        );
        assert!(fancy.delta_v > plain.delta_v, "more thrust, more delta-v");
        assert!(
            fancy.reliability > plain.reliability,
            "more redundancy, more reliability"
        );
        assert!(
            fancy.cost_estimate > plain.cost_estimate,
            "and more of everything costs more"
        );
    }

    #[test]
    fn a_sounding_rocket_cannot_be_crewed() {
        // **The archetype's band is a rule, not a suggestion.**
        let bounded = SliderValues {
            life_support: 90,
            heat_shield: 90,
            ..SliderValues::zero()
        }
        .bounded(ArchetypeId(0), &[]);
        assert_eq!(bounded.life_support, 0);
        assert_eq!(bounded.heat_shield, 0);
    }

    #[test]
    fn an_orbital_launcher_must_be_able_to_fly() {
        // **The band has a floor as well as a ceiling.**
        let bounded = SliderValues::zero().bounded(ArchetypeId(1), &[]);
        assert_eq!(
            bounded.propulsion, 20,
            "an orbital launcher needs propulsion"
        );
        assert_eq!(bounded.payload, 20);
    }

    #[test]
    fn a_slider_the_agency_has_not_developed_is_pinned_at_zero() {
        // **The capability gate.** A player cannot buy a heat shield before
        // the agency has developed re-entry.
        let design = SliderValues {
            heat_shield: 90,
            ..SliderValues::zero()
        };
        let bounded = design.bounded(ArchetypeId(2), &[]);
        assert_eq!(bounded.heat_shield, 0, "without the capability");

        let bounded = design.bounded(ArchetypeId(2), &[CapabilityId(1)]);
        assert_eq!(bounded.heat_shield, 90, "and with it");
    }
}
