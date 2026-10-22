//! Staff: the people the agency employs.
//!
//! A person has exactly one assignment at a time. Unassigned is a cost, not a
//! neutral state: an unassigned person accrues fatigue at half rate and drifts
//! in morale. See `docs/DOMAIN_MODEL.md` §5.

use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::ids::{
    DivisionId, FacilityId, PersonId, ProgrammeId, ResearchProjectId, TestCampaignId,
};
use crate::money::Money;
use crate::performance::{Modifier, ModifierKind};
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

/// The traits a person can have.
///
/// **A trait is a small, permanent difference in how somebody works.** It is
/// not a stat and not a job: it is the thing that makes two people with the
/// same attributes different to have on a team, and it is the cheapest way for
/// the player to see why.
///
/// The values are small, because the eight factors run from -100 to +100 and
/// the reference composite is built from a good team. **A trait is worth a few
/// points, not a career.**
pub static TRAITS: LazyLock<Vec<TraitDef>> = LazyLock::new(|| {
    vec![
        TraitDef {
            id: TraitId(0),
            name: "Prodigy".to_owned(),
            description: "Understands things faster than anyone expected.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::Skill, 6),
                Modifier::new(ModifierKind::Creativity, 3),
            ],
            tags: vec!["ability".to_owned()],
        },
        TraitDef {
            id: TraitId(1),
            name: "Methodical".to_owned(),
            description: "Slow and careful, and never wrong twice.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::Skill, 4),
                Modifier::new(ModifierKind::Creativity, -3),
            ],
            tags: vec!["ability".to_owned()],
        },
        TraitDef {
            id: TraitId(2),
            name: "Restless".to_owned(),
            description: "Never satisfied, and always looking for another way.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::Creativity, 6),
                Modifier::new(ModifierKind::Morale, -4),
            ],
            tags: vec!["temperament".to_owned()],
        },
        TraitDef {
            id: TraitId(3),
            name: "Perfectionist".to_owned(),
            description: "Will not put up with a bad instrument.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::Tools, 5),
                Modifier::new(ModifierKind::Morale, -3),
            ],
            tags: vec!["temperament".to_owned()],
        },
        TraitDef {
            id: TraitId(4),
            name: "Iron constitution".to_owned(),
            description: "Never ill, never broken.".to_owned(),
            modifiers: vec![Modifier::new(ModifierKind::Health, 7)],
            tags: vec!["health".to_owned()],
        },
        TraitDef {
            id: TraitId(5),
            name: "Frail".to_owned(),
            description: "Bodies that were not built for this.".to_owned(),
            modifiers: vec![Modifier::new(ModifierKind::Health, -7)],
            tags: vec!["health".to_owned()],
        },
        TraitDef {
            id: TraitId(6),
            name: "Charming".to_owned(),
            description: "Gets on with everyone, and is asked to.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::RoleFit, 4),
                Modifier::new(ModifierKind::Support, 3),
            ],
            tags: vec!["personality".to_owned()],
        },
        TraitDef {
            id: TraitId(7),
            name: "Awkward".to_owned(),
            description: "Finds the work easy and the people hard.".to_owned(),
            modifiers: vec![Modifier::new(ModifierKind::RoleFit, -5)],
            tags: vec!["personality".to_owned()],
        },
        TraitDef {
            id: TraitId(8),
            name: "Team player".to_owned(),
            description: "Lifts the people around them.".to_owned(),
            modifiers: vec![Modifier::new(ModifierKind::Support, 6)],
            tags: vec!["personality".to_owned()],
        },
        TraitDef {
            id: TraitId(9),
            name: "Loner".to_owned(),
            description: "Works well alone, and less well with others.".to_owned(),
            modifiers: vec![Modifier::new(ModifierKind::Support, -6)],
            tags: vec!["personality".to_owned()],
        },
        TraitDef {
            id: TraitId(10),
            name: "Night owl".to_owned(),
            description: "At their best after midnight.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::Morale, 5),
                Modifier::new(ModifierKind::Health, -3),
            ],
            tags: vec!["temperament".to_owned()],
        },
        TraitDef {
            id: TraitId(11),
            name: "Burns bright".to_owned(),
            description: "Brilliant for a while, and then not.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::Creativity, 5),
                Modifier::new(ModifierKind::Health, -4),
                Modifier::new(ModifierKind::Fatigue, -5),
            ],
            tags: vec!["temperament".to_owned(), "health".to_owned()],
        },
        TraitDef {
            id: TraitId(12),
            name: "Cynic".to_owned(),
            description: "Expects the worst and is rarely disappointed.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::Morale, -6),
                Modifier::new(ModifierKind::Skill, 3),
            ],
            tags: vec!["temperament".to_owned()],
        },
        TraitDef {
            id: TraitId(13),
            name: "Natural engineer".to_owned(),
            description: "Reads a machine like a book.".to_owned(),
            modifiers: vec![
                Modifier::new(ModifierKind::Tools, 4),
                Modifier::new(ModifierKind::Creativity, 3),
            ],
            tags: vec!["ability".to_owned()],
        },
    ]
});

/// A trait by its identifier.
#[must_use]
pub fn trait_def(id: TraitId) -> Option<&'static TraitDef> {
    TRAITS.iter().find(|definition| definition.id == id)
}

/// A trait's name, or a placeholder for one the campaign has never heard of.
#[must_use]
pub fn trait_name(id: TraitId) -> &'static str {
    trait_def(id).map_or("Something unusual", |definition| definition.name.as_str())
}

/// A personality hook: a small thing about a person that changes how they work.
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
    ///
    /// **`None` means the agency policy decides**, and it is not the same as a
    /// full load: a person the policy has not moved works whatever everybody
    /// else works. See [`Policy`].
    pub shift: Option<u16>,
}

/// One of the agency's standing instructions.
///
/// **A policy is a standing instruction and not a one-off.** The per-person
/// shift is the surgical tool; this is the tool for an agency of three hundred
/// people, and it is the answer to "my team is burning out".
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Policy {
    /// **The agency's shift.** Everybody works this shift, unless they have
    /// been moved off it individually.
    Shift {
        /// The shift, as a scaled percentage of a full load.
        shift: u16,
        /// Only the people at or over this fatigue are moved.
        ///
        /// **This is the thermostat.** With no threshold everybody works the
        /// shift; with one, the people who are fresher than the threshold keep
        /// their own, and the people who are tireder than it are eased off —
        /// which means a tired person recovers, comes back up to the threshold,
        /// and is eased off again. The cycle is slow, and the point of it is
        /// that the agency never has anybody work themselves to pieces.
        only_over: Option<u8>,
    },
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
            shift: None,
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

    /// The shift this person works, when the policy does not decide it.
    ///
    /// **`None` means the policy decides.**
    #[must_use]
    pub const fn shift_override(&self) -> Option<u16> {
        self.shift
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
    ///
    /// **No identifier appears here, and that is deliberate.** The domain does
    /// not know what the programmes and the facilities are called — it holds
    /// the state's shape, not the state's content — so all it can say is what
    /// *kind* of thing the person is on. The name is added by
    /// `sim::derived::assignment_description`, which does have the state.
    ///
    /// A player who reads "programme 3" has been shown an implementation
    /// detail, and an implementation detail in a message is a bug.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::Unassigned => "Unassigned".to_owned(),
            Self::Division(_) => "a division".to_owned(),
            Self::Programme { role, .. } => match role {
                ProgrammeRole::Lead => "leads a programme".to_owned(),
                ProgrammeRole::Member => "works on a programme".to_owned(),
                ProgrammeRole::Adviser => "advises on a programme".to_owned(),
            },
            Self::Project { .. } => "a research project".to_owned(),
            Self::Facility { .. } => "a facility".to_owned(),
            Self::TestCampaign { .. } => "a test campaign".to_owned(),
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
