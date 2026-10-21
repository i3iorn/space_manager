//! Typed identity for every entity in the game.
//!
//! Ids are monotonic `u64` counters allocated per type. They are never random and
//! never reused, so a deleted facility's id stays retired and the chronicle's
//! subject links remain resolvable. See `docs/DOMAIN_MODEL.md` §4.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Defines a typed id newtype and its accessor.
///
/// Every id is a newtype over `u64` with its own allocation counter, so a
/// `ProgrammeId` can never be passed where a `PersonId` is expected.
macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
        pub struct $name(pub u64);

        impl $name {
            /// The raw counter value this id was allocated from.
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_type!(
    /// The player's agency. There is one per campaign.
    AgencyId
);
id_type!(
    /// A member of staff.
    PersonId
);
id_type!(
    /// A strategic programme.
    ProgrammeId
);
id_type!(
    /// A division of the agency, headed by a person.
    DivisionId
);
id_type!(
    /// A research project.
    ResearchProjectId
);
id_type!(
    /// A test campaign against an existing capability.
    TestCampaignId
);
id_type!(
    /// A built facility.
    FacilityId
);
id_type!(
    /// A vehicle design.
    VehicleId
);
id_type!(
    /// A mission.
    MissionId
);
id_type!(
    /// A person, programme, or facility that a chronicle entry is about.
    SubjectId
);
id_type!(
    /// An entry in the campaign's chronicle.
    ChronicleId
);
id_type!(
    /// A kind of event the simulation can draw.
    EventId
);
id_type!(
    /// An order the player has issued.
    OrderId
);
id_type!(
    /// A dated commitment that falls due at an instant.
    CommitmentId
);

id_type!(
    /// A dated piece of a programme.
    ObjectiveId
);

/// The per-type id allocation counters.
///
/// The counters are part of the saved state, so a loaded campaign allocates the
/// same ids the original did. Ids are never reused after an entity is deleted.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdCounters {
    agency: u64,
    person: u64,
    programme: u64,
    division: u64,
    research_project: u64,
    test_campaign: u64,
    facility: u64,
    vehicle: u64,
    mission: u64,
    subject: u64,
    chronicle: u64,
    event: u64,
    order: u64,
    commitment: u64,
    objective: u64,
}

/// Generates the id allocation methods for [`IdCounters`].
///
/// Each method takes the next value of its counter and increments it. The method
/// name, the field it advances, and the id type it returns are all given
/// explicitly, because `next_$field` style concatenation does not expand the way
/// it reads.
macro_rules! next_id {
    ($($method:ident => $field:ident : $ty:ident),* $(,)?) => {
        $(
            /// Allocates the next id of this kind.
            #[must_use]
            pub fn $method(&mut self) -> $ty {
                let id = self.$field;
                self.$field += 1;
                $ty(id)
            }
        )*
    };
}

impl IdCounters {
    next_id! {
        next_agency => agency: AgencyId,
        next_person => person: PersonId,
        next_programme => programme: ProgrammeId,
        next_division => division: DivisionId,
        next_research_project => research_project: ResearchProjectId,
        next_test_campaign => test_campaign: TestCampaignId,
        next_facility => facility: FacilityId,
        next_vehicle => vehicle: VehicleId,
        next_mission => mission: MissionId,
        next_subject => subject: SubjectId,
        next_chronicle => chronicle: ChronicleId,
        next_event => event: EventId,
        next_order => order: OrderId,
        next_commitment => commitment: CommitmentId,
        next_objective => objective: ObjectiveId,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_allocated_monotonically() {
        let mut counters = IdCounters::default();
        let first = counters.next_person();
        let second = counters.next_person();
        assert!(first < second);
        assert_eq!(first.get(), 0);
        assert_eq!(second.get(), 1);
    }

    #[test]
    fn ids_of_different_kinds_come_from_different_counters() {
        // The compile-time guarantee that a `ProgrammeId` cannot be used as a
        // `PersonId` is the whole point of the newtype, so it needs no runtime
        // test. What is worth pinning is that the two kinds have independent
        // counters and neither disturbs the other.
        let mut counters = IdCounters::default();
        let person = counters.next_person();
        let programme = counters.next_programme();
        assert_eq!(person.get(), 0);
        assert_eq!(programme.get(), 0);
        assert_eq!(counters.next_person().get(), 1);
    }

    #[test]
    fn counters_round_trip_through_serde() {
        let mut counters = IdCounters::default();
        let _ = counters.next_person();
        let _ = counters.next_person();
        let json = serde_json::to_string(&counters).expect("counters serialise");
        let restored: IdCounters = serde_json::from_str(&json).expect("counters deserialise");
        assert_eq!(restored, counters);
    }
}
