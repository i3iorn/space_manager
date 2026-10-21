//! `CampaignState`: the aggregate root.
//!
//! The only thing the simulation mutates, the only thing a save contains, and
//! the only thing the UI reads. Every collection is a `BTreeMap`, because a
//! total iteration order is a determinism requirement and not a style
//! preference. See `docs/DOMAIN_MODEL.md` §17.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::chronicle::ChronicleEntry;
use crate::facility::Facility;
use crate::finance::Treasury;
use crate::ids::{
    AgencyId, ChronicleId, FacilityId, IdCounters, MissionId, OrderId, PersonId, ProgrammeId,
    ResearchProjectId, TestCampaignId, VehicleId,
};
use crate::mission::Mission;
use crate::order::Order;
use crate::person::Person;
use crate::political::PoliticalState;
use crate::programme::Programme;
use crate::public::PublicState;
use crate::research::{CapabilityId, ResearchProject, TestCampaign};
use crate::time::Calendar;
use crate::vehicle::Vehicle;

/// The player's agency.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Agency {
    /// The agency's identifier.
    pub id: AgencyId,
    /// Its name.
    pub name: String,
    /// The strategy it has committed to.
    pub focus: String,
    /// The year the campaign began.
    pub founded: i32,
}

impl Agency {
    /// A new agency.
    #[must_use]
    pub fn new(id: AgencyId, name: String, founded: i32) -> Self {
        Self {
            id,
            name,
            focus: "The science-first authority".to_owned(),
            founded,
        }
    }
}

/// Everything the campaign knows.
///
/// The aggregate root. It is cloned at the month boundary rather than mutated
/// in place, which is what makes the commit atomic and the branch feature
/// trivial.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CampaignState {
    /// The player's agency.
    pub agency: Agency,
    /// The campaign calendar. Its `current` month is the only record of when the
    /// campaign is — there is no second copy of the date.
    /// The agency's money.
    pub treasury: Treasury,
    /// The agency's political position.
    pub political: PoliticalState,
    /// How the public sees the agency.
    pub public: PublicState,
    /// Everyone the agency employs.
    pub people: BTreeMap<PersonId, Person>,
    /// The agency's programmes.
    pub programmes: BTreeMap<ProgrammeId, Programme>,
    /// Its research projects.
    pub projects: BTreeMap<ResearchProjectId, ResearchProject>,
    /// Its test campaigns.
    pub test_campaigns: BTreeMap<TestCampaignId, TestCampaign>,
    /// Its facilities.
    pub facilities: BTreeMap<FacilityId, Facility>,
    /// Its vehicles.
    pub vehicles: BTreeMap<VehicleId, Vehicle>,
    /// Its missions.
    pub missions: BTreeMap<MissionId, Mission>,
    /// The orders the player has issued that have not yet landed.
    ///
    /// **These are part of the campaign, and part of the save.** A campaign
    /// saved with a hire in flight must restore that hire, or the campaign
    /// silently changes.
    pub pending_orders: BTreeMap<OrderId, Order>,
    /// How many random event checks have been drawn.
    ///
    /// **This is the bucket the event stream is keyed on.** The index is a
    /// stable identifier: the n-th check of a campaign always draws the same
    /// numbers, whether or not the check happens at the same instant — and the
    /// check's interval tightens under stress, so the instants move and the
    /// index does not.
    pub event_checks: u64,
    /// The campaign's history. Append-only.
    pub chronicle: Vec<ChronicleEntry>,
    /// How much knowledge the agency holds, per discipline-derived capability.
    pub knowledge: BTreeMap<CapabilityId, u32>,
    /// The capabilities the agency has unlocked.
    pub unlocked: BTreeSet<CapabilityId>,

    /// What the testing has bought, by capability: the capability and the
    /// percentage points of failure risk it has taken out.
    ///
    /// **A tested capability is a safer capability**, and the risk it has taken
    /// out has to be remembered after the campaign that found it is gone.
    pub tested: BTreeMap<CapabilityId, i16>,
    /// The id allocation counters.
    pub id_counters: IdCounters,
}

impl CampaignState {
    /// A new campaign.
    ///
    /// The state holds **no date**. The current instant lives in the simulation,
    /// and the calendar is a projection of it derived from the agency's founding
    /// year — see `docs/DOMAIN_MODEL.md` §2.
    #[must_use]
    pub fn new(agency: Agency) -> Self {
        Self {
            agency,
            treasury: Treasury::new(),
            political: PoliticalState::new(),
            public: PublicState::new(),
            people: BTreeMap::new(),
            programmes: BTreeMap::new(),
            projects: BTreeMap::new(),
            test_campaigns: BTreeMap::new(),
            facilities: BTreeMap::new(),
            vehicles: BTreeMap::new(),
            missions: BTreeMap::new(),
            pending_orders: BTreeMap::new(),
            event_checks: 0,
            chronicle: Vec::new(),
            knowledge: BTreeMap::new(),
            unlocked: BTreeSet::new(),
            tested: BTreeMap::new(),
            id_counters: IdCounters::default(),
        }
    }

    /// The calendar the campaign's dates are projected onto.
    #[must_use]
    pub const fn calendar(&self) -> Calendar {
        Calendar::new(self.agency.founded)
    }

    /// The number of staff the agency employs.
    #[must_use]
    pub fn headcount(&self) -> usize {
        self.people.len()
    }

    /// The number of people with nothing to do.
    #[must_use]
    pub fn unassigned_count(&self) -> usize {
        self.people
            .values()
            .filter(|person| !person.is_assigned())
            .count()
    }

    /// The names of the people who are notable enough to be listed on their own.
    ///
    /// The player knows the famous ones intimately and the rest through
    /// aggregate screens, which is how 300 people stay manageable.
    #[must_use]
    pub fn notable(&self) -> Vec<&Person> {
        self.people
            .values()
            .filter(|person| person.state.fame.abs() >= 25 || person.career.grade >= 4)
            .collect()
    }

    /// Appends an entry to the chronicle.
    ///
    /// The chronicle is append-only: nothing edits or deletes an entry.
    pub fn chronicle(&mut self, entry: ChronicleEntry) {
        self.chronicle.push(entry);
    }

    /// Allocates the next chronicle entry identifier.
    pub fn next_chronicle_id(&mut self) -> ChronicleId {
        let id = self.chronicle.len() as u64;
        ChronicleId(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Instant;

    fn state() -> CampaignState {
        CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039))
    }

    #[test]
    fn a_new_campaign_starts_in_its_first_month() {
        assert_eq!(state().calendar().date(Instant::ZERO).year, 2039);
    }

    #[test]
    fn the_chronicle_is_append_only() {
        let mut campaign = state();
        let before = campaign.chronicle.len();
        campaign.chronicle(ChronicleEntry::new(
            ChronicleId(0),
            Instant::ZERO,
            crate::chronicle::Category::Discovery,
            "Something happened".to_owned(),
            crate::chronicle::Severity::Info,
        ));
        assert_eq!(campaign.chronicle.len(), before + 1);
    }

    #[test]
    fn unassigned_people_are_counted_separately() {
        let mut campaign = state();
        campaign.people.insert(
            PersonId(0),
            Person::new(
                PersonId(0),
                "Marsha Okonjo".to_owned(),
                crate::person::Role::Scientist,
                crate::money::Money::from_credits(1_200),
            ),
        );
        assert_eq!(campaign.headcount(), 1);
        assert_eq!(campaign.unassigned_count(), 1);
    }
}
