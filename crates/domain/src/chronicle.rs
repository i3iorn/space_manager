//! The chronicle: the campaign's permanent, queryable history.
//!
//! Every event and every significant non-event writes an entry. Entries are
//! append-only and never edited, and `SubjectRef` is what makes the chronicle
//! queryable — filtering by a programme returns its entire history.
//! See `docs/DOMAIN_MODEL.md` §14.

use serde::{Deserialize, Serialize};

use crate::ids::{ChronicleId, FacilityId, PersonId, ProgrammeId, VehicleId};
use crate::time::Instant;

/// What kind of thing happened.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Category {
    /// Something broke.
    Technical,
    /// Something happened to a person.
    Personnel,
    /// Something was found out.
    Discovery,
    /// Something happened in the state.
    Political,
    /// Something happened to the money.
    Financial,
    /// Something in the world.
    Environmental,
    /// Something the public did or thought.
    Public,
    /// Something outside the agency.
    External,
}

impl Category {
    /// Every category.
    pub const ALL: [Self; 8] = [
        Self::Technical,
        Self::Personnel,
        Self::Discovery,
        Self::Political,
        Self::Financial,
        Self::Environmental,
        Self::Public,
        Self::External,
    ];

    /// The category's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Technical => "Technical",
            Self::Personnel => "Personnel",
            Self::Discovery => "Discovery",
            Self::Political => "Political",
            Self::Financial => "Financial",
            Self::Environmental => "Environmental",
            Self::Public => "Public",
            Self::External => "External",
        }
    }
}

/// How much an entry matters.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Severity {
    /// Ordinary.
    Info,
    /// Worth noticing.
    Minor,
    /// Important.
    Major,
    /// The kind of thing a campaign is remembered for.
    Critical,
}

impl Severity {
    /// Every severity.
    pub const ALL: [Self; 4] = [Self::Info, Self::Minor, Self::Major, Self::Critical];

    /// The severity's name, as shown to the player.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Minor => "Minor",
            Self::Major => "Major",
            Self::Critical => "Critical",
        }
    }
}

/// Something an entry is about.
///
/// This is what makes the chronicle queryable: filtering by a programme returns
/// its entire history.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum SubjectRef {
    /// A programme.
    Programme(ProgrammeId),
    /// A person.
    Person(PersonId),
    /// A facility.
    Facility(FacilityId),
    /// A vehicle.
    Vehicle(VehicleId),
}

/// One entry in the campaign's history.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ChronicleEntry {
    /// The entry's identifier.
    pub id: ChronicleId,
    /// When it happened.
    pub date: Instant,
    /// What kind of thing it was.
    pub category: Category,
    /// One line.
    pub headline: String,
    /// Optional detail.
    pub body: Option<String>,
    /// What it is about.
    pub subjects: Vec<SubjectRef>,
    /// How much it matters.
    pub severity: Severity,
    /// Free-form tags, for filtering.
    pub tags: Vec<String>,
}

impl ChronicleEntry {
    /// A new entry.
    #[must_use]
    pub fn new(
        id: ChronicleId,
        date: Instant,
        category: Category,
        headline: String,
        severity: Severity,
    ) -> Self {
        Self {
            id,
            date,
            category,
            headline,
            body: None,
            subjects: Vec::new(),
            severity,
            tags: Vec::new(),
        }
    }

    /// Adds a subject.
    #[must_use]
    pub fn about(mut self, subject: SubjectRef) -> Self {
        self.subjects.push(subject);
        self
    }

    /// Adds detail.
    #[must_use]
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_can_be_about_several_subjects() {
        let entry = ChronicleEntry::new(
            ChronicleId(1),
            Instant::from_months(4),
            Category::Technical,
            "Stage 2 failed to ignite".to_owned(),
            Severity::Major,
        )
        .about(SubjectRef::Programme(ProgrammeId(0)))
        .about(SubjectRef::Person(PersonId(3)));

        assert_eq!(entry.subjects.len(), 2);
    }

    #[test]
    fn severities_are_ordered() {
        let mut severities: Vec<Severity> = Severity::ALL.to_vec();
        severities.sort();
        assert_eq!(severities[0], Severity::Info);
        assert_eq!(severities[3], Severity::Critical);
    }
}
