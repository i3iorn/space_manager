//! Staff: the people the agency employs.
//!
//! A person has exactly one assignment at a time. Unassigned is a cost, not a
//! neutral state: an unassigned person accrues fatigue at half rate and drifts
//! in morale. See `docs/DOMAIN_MODEL.md` §5.

use serde::{Deserialize, Serialize};

use crate::ids::{
    DivisionId, FacilityId, PersonId, ProgrammeId, ResearchProjectId, TestCampaignId,
};
use crate::money::Money;
use crate::performance::Modifier;
use crate::scale::{
    Remainders, percent_to_units, signed_percent_to_units, signed_units_to_percent,
    units_to_percent,
};

/// What a person does for the agency.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Role {
    /// Studies the natural world and the agency's own technology.
    Scientist,
    /// Designs and builds things.
    Engineer,
    /// Trains for and flies missions.
    Astronaut,
    /// Keeps the agency running.
    Administrator,
    /// Leads people and programmes.
    Manager,
}

impl Role {
    /// Every role, in the order the interface lists them.
    pub const ALL: [Self; 5] = [
        Self::Scientist,
        Self::Engineer,
        Self::Astronaut,
        Self::Administrator,
        Self::Manager,
    ];

    /// The role's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Scientist => "Scientist",
            Self::Engineer => "Engineer",
            Self::Astronaut => "Astronaut",
            Self::Administrator => "Administrator",
            Self::Manager => "Manager",
        }
    }

    /// The name of the division this role belongs to.
    #[must_use]
    pub const fn division(self) -> DivisionId {
        match self {
            Self::Scientist => DivisionId(0),
            Self::Engineer => DivisionId(1),
            Self::Astronaut => DivisionId(2),
            Self::Administrator => DivisionId(3),
            Self::Manager => DivisionId(4),
        }
    }
}

/// A narrow field a person is genuinely good at.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Specialism {
    /// Engines and rockets.
    Propulsion,
    /// Structures and materials.
    Structures,
    /// Avionics and guidance.
    Avionics,
    /// Keeping people alive.
    LifeSupport,
    /// Power generation and storage.
    Power,
    /// Rocks, planets, and what they are made of.
    Geology,
    /// Medicine and biology.
    Medicine,
    /// Public relations.
    PublicRelations,
    /// Computing.
    Computing,
    /// Making things at scale.
    Manufacturing,
}

impl Specialism {
    /// Every specialism.
    pub const ALL: [Self; 10] = [
        Self::Propulsion,
        Self::Structures,
        Self::Avionics,
        Self::LifeSupport,
        Self::Power,
        Self::Geology,
        Self::Medicine,
        Self::PublicRelations,
        Self::Computing,
        Self::Manufacturing,
    ];

    /// The specialism's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Propulsion => "Propulsion",
            Self::Structures => "Structures",
            Self::Avionics => "Avionics",
            Self::LifeSupport => "Life support",
            Self::Power => "Power",
            Self::Geology => "Geology",
            Self::Medicine => "Medicine",
            Self::PublicRelations => "Public relations",
            Self::Computing => "Computing",
            Self::Manufacturing => "Manufacturing",
        }
    }
}

/// Where a person comes from.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Origin {
    /// The world or settlement they grew up on.
    pub homeworld: String,
    /// Their species or people.
    pub species: String,
}

/// The five numbers that describe a person's ability.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Attributes {
    /// Technical ability.
    pub technical: u8,
    /// Originality.
    pub creative: u8,
    /// The ability to lead and to organise.
    pub leadership: u8,
    /// Physical condition and stamina.
    pub physical: u8,
    /// The ability to deal with people and institutions.
    pub diplomatic: u8,
}

impl Attributes {
    /// The attribute best suited to a role.
    #[must_use]
    pub const fn primary_for(self, role: Role) -> u8 {
        match role {
            Role::Scientist => self.creative,
            Role::Engineer => self.technical,
            Role::Astronaut => self.physical,
            Role::Administrator => self.diplomatic,
            Role::Manager => self.leadership,
        }
    }
}

/// The identifier of a trait.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct TraitId(pub u16);

/// A personality hook: a small thing about a person that changes how they work.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TraitDef {
    /// The trait's identifier.
    pub id: TraitId,
    /// The trait's name.
    pub name: String,
    /// What the trait means.
    pub description: String,
    /// The modifiers the trait applies.
    pub modifiers: Vec<Modifier>,
    /// Free-form tags, for filtering and for the chronicle.
    pub tags: Vec<String>,
}

/// How a person is doing.
///
/// The values are **scaled integers** at 256 units per percent, so that a rate of
/// 0.008% an hour is not truncated away. See `docs/DOMAIN_MODEL.md` §2.3.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PersonState {
    /// How content they are, from `-25_600` to `+25_600`.
    pub morale: i16,
    /// How committed they are to the agency, from `-25_600` to `+25_600`.
    pub loyalty: i16,
    /// How tired they are, from `0` to `25_600`.
    pub fatigue: u16,
    /// Their physical and mental condition, from `0` to `25_600`.
    pub health: u16,
    /// How well known they are, from `-25_600` to `+25_600`.
    pub fame: i16,
    /// **How hard they work**, as a percentage of a normal load.
    ///
    /// **This is the player's throttle.** A hundred percent is a full load. Below
    /// it the person contributes less and tires less; above it they contribute
    /// more and tire much more, and the extra effort is worth less than the
    /// extra fatigue it costs.
    ///
    /// The range is zero to two hundred percent, in steps of ten. Zero is the
    /// same as being idle, and it is how a person is parked on a team without
    /// being taken off it.
    pub shift: u16,
}

impl PersonState {
    /// A person in reasonable condition, working a full load.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            morale: signed_percent_to_units(20),
            loyalty: signed_percent_to_units(30),
            fatigue: 0,
            health: percent_to_units(100) as u16,
            fame: 0,
            shift: percent_to_units(100) as u16,
        }
    }

    /// The fatigue as a whole percentage.
    #[must_use]
    pub const fn fatigue_percent(&self) -> u32 {
        units_to_percent(self.fatigue as u32)
    }

    /// The health as a whole percentage.
    #[must_use]
    pub const fn health_percent(&self) -> u32 {
        units_to_percent(self.health as u32)
    }

    /// The morale as a whole percentage.
    #[must_use]
    pub const fn morale_percent(&self) -> i16 {
        signed_units_to_percent(self.morale)
    }

    /// The shift as a whole percentage.
    #[must_use]
    pub const fn shift_percent(&self) -> u32 {
        units_to_percent(self.shift as u32)
    }

    /// The shift, clamped to the range a person can be put on.
    ///
    /// **The clamp is here because the value comes from a player**, and a
    /// player who asks for four hundred percent gets two hundred.
    #[must_use]
    pub const fn clamped_shift(shift: u16) -> u16 {
        let maximum = (percent_to_units(200)) as u16;
        if shift > maximum { maximum } else { shift }
    }
}

impl Default for PersonState {
    fn default() -> Self {
        Self::new()
    }
}

/// What a person is doing right now.
///
/// Only one assignment at a time. Enforced as an invariant, because a rejected
/// order has to be able to say *why* it was rejected, and that reason can
/// only be produced if the state is unambiguous.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Assignment {
    /// Not on anything. A cost, not a neutral state.
    Unassigned,
    /// Part of a division.
    Division(DivisionId),
    /// On a programme.
    Programme {
        /// The programme.
        programme: ProgrammeId,
        /// What they do for it.
        role: ProgrammeRole,
    },
    /// On a research project.
    Project {
        /// The project.
        project: ResearchProjectId,
    },
    /// Running a facility.
    Facility {
        /// The facility.
        facility: FacilityId,
    },
    /// On a test campaign.
    TestCampaign {
        /// The campaign.
        campaign: TestCampaignId,
    },
}

/// What a person does for a programme.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ProgrammeRole {
    /// Runs the programme.
    Lead,
    /// Does the work.
    Member,
    /// Advises the programme.
    Adviser,
}

impl Assignment {
    /// A short description of where the person is, for the interface.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::Unassigned => "Unassigned".to_owned(),
            Self::Division(id) => format!("Division {}", id.0),
            Self::Programme { programme, role } => {
                let role = match role {
                    ProgrammeRole::Lead => "leads",
                    ProgrammeRole::Member => "works on",
                    ProgrammeRole::Adviser => "advises",
                };
                format!("{role} programme {}", programme.0)
            }
            Self::Project { project } => format!("Project {}", project.0),
            Self::Facility { facility } => format!("Facility {}", facility.0),
            Self::TestCampaign { campaign } => format!("Test campaign {}", campaign.0),
        }
    }
}

/// The terms of a person's employment.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Contract {
    /// What they are paid each month.
    pub salary: Money,
    /// How long is left on the contract, if it is fixed.
    pub term_remaining: Option<u32>,
    /// What they have asked for.
    pub demands: Vec<String>,
}

/// A person's standing in the agency.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Career {
    /// Their seniority.
    pub grade: u8,
    /// How long they have served.
    pub years_of_service: u32,
    /// The qualifications they hold.
    pub certifications: Vec<String>,
    /// How many times they have been promoted.
    pub promotions: u32,
}

/// One thing that has happened to a person.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The month it happened.
    pub month: u32,
    /// What happened.
    pub event: String,
}

/// A member of staff.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Person {
    /// Their identifier.
    pub id: PersonId,
    /// Their name.
    pub name: String,
    /// What they do.
    pub role: Role,
    /// Where they come from.
    pub origin: Origin,
    /// Their age.
    pub age: u32,
    /// Their ability.
    pub attributes: Attributes,
    /// What they are good at.
    pub specialisms: Vec<Specialism>,
    /// Who they are.
    pub traits: Vec<TraitId>,
    /// How they are doing.
    pub state: PersonState,
    /// Their standing.
    pub career: Career,
    /// What they are doing.
    pub assignment: Assignment,
    /// Their terms.
    pub contract: Contract,
    /// What has happened to them.
    pub history: Vec<HistoryEntry>,
    /// The fractional remainders carried between rate applications.
    ///
    /// These are part of the person, not part of the clock. They are what makes
    /// a rate exact: without them a rate applied over a fraction of its period
    /// loses the fraction, every time, for the rest of the campaign.
    pub remainders: Remainders,
}

impl Person {
    /// A new member of staff.
    #[must_use]
    pub fn new(id: PersonId, name: String, role: Role, salary: Money) -> Self {
        Self {
            id,
            name,
            role,
            origin: Origin {
                homeworld: "the Core".to_owned(),
                species: "Human".to_owned(),
            },
            age: 28,
            attributes: Attributes {
                technical: 50,
                creative: 50,
                leadership: 50,
                physical: 50,
                diplomatic: 50,
            },
            specialisms: Vec::new(),
            traits: Vec::new(),
            state: PersonState::new(),
            career: Career {
                grade: 1,
                years_of_service: 0,
                certifications: Vec::new(),
                promotions: 0,
            },
            assignment: Assignment::Unassigned,
            contract: Contract {
                salary,
                term_remaining: None,
                demands: Vec::new(),
            },
            history: Vec::new(),
            remainders: Remainders::new(),
        }
    }

    /// Whether the person is on anything at all.
    #[must_use]
    pub const fn is_assigned(&self) -> bool {
        !matches!(self.assignment, Assignment::Unassigned)
    }

    /// The person's name, for messages.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn person() -> Person {
        Person::new(
            PersonId(1),
            "Marsha Okonjo".to_owned(),
            Role::Scientist,
            Money::from_credits(1_200),
        )
    }

    #[test]
    fn a_new_person_is_unassigned() {
        let person = person();
        assert!(!person.is_assigned());
        assert_eq!(person.assignment, Assignment::Unassigned);
    }

    #[test]
    fn a_new_person_is_in_reasonable_condition() {
        let person = person();
        assert_eq!(person.state.health_percent(), 100);
        assert_eq!(person.state.fatigue, 0);
        assert!(person.state.morale > 0);
    }

    #[test]
    fn every_role_belongs_to_a_division() {
        let mut divisions = Vec::new();
        for role in Role::ALL {
            let division = role.division();
            assert!(!divisions.contains(&division), "two roles share a division");
            divisions.push(division);
        }
    }

    #[test]
    fn every_role_has_a_distinct_label() {
        let mut labels: Vec<&str> = Role::ALL.iter().map(|r| r.label()).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), Role::ALL.len());
    }
}
