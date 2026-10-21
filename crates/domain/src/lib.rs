//! The domain model for `space_manager`.
//!
//! This crate holds the **data** of the game: the entities, their fields, the ids
//! that identify them, and the invariants that keep them coherent. It holds no
//! behaviour that *decides* anything — everything that computes lives in the `sim`
//! crate.
//!
//! # Rules for this crate
//!
//! * **It depends on nothing of ours.** Not on `sim`, not on `ui`. A CI check
//!   enforces this; see `docs/ARCHITECTURE.md` §3.
//! * **No `unwrap`, no `expect`, no `panic!`** outside tests. A panic here
//!   corrupts a campaign.
//! * **Everything that can be an integer is an integer.** Money, scores,
//!   capacities, percentages, and time. Floating point is presentation-only.
//! * **Every collection is a `BTreeMap`.** A total iteration order is a
//!   determinism requirement, not a style preference.
//!
//! # Where to look
//!
//! | Module | Holds |
//! |---|---|
//! | [`ids`] | Typed entity ids and the allocation counters |
//! | [`time`] | The campaign calendar |
//! | [`money`] | Integer money and deterministic splitting |
//! | [`person`] | Staff, attributes, traits, assignment, contract |
//! | [`performance`] | The modifier vocabulary the performance model reads |
//! | [`programme`] | Programmes, objectives, commitments, milestones |
//! | [`research`] | Research projects, disciplines, capabilities, test campaigns |
//! | [`facility`] | Facilities, construction choices, specifications |
//! | [`vehicle`] | Vehicle archetypes, subsystem sliders, derived statistics |
//! | [`mission`] | Missions and their outcomes |
//! | [`finance`] | Budgets, the treasury, income sources |
//! | [`political`] | Approval, political capital, promises |
//! | [`public`] | Public opinion and prestige |
//! | [`chronicle`] | The campaign's permanent, queryable history |
//! | [`event`] | Event definitions, expressed as data |
//! | [`order`] | The player's requests, and only their requests |
//! | [`report`] | The structured month report |
//! | [`state`] | `CampaignState`, the aggregate root |
//! | [`invariant`] | The invariant catalogue |
//!
//! The field-level contract is `docs/DOMAIN_MODEL.md`. The design intent behind
//! these types is `docs/MANAGEABLE_PARTS.md`.

#![deny(missing_docs)]

pub mod chronicle;
pub mod event;
pub mod facility;
pub mod finance;
pub mod ids;
pub mod invariant;
pub mod mission;
pub mod money;
pub mod order;
pub mod performance;
pub mod person;
pub mod political;
pub mod programme;
pub mod public;
pub mod report;
pub mod research;
pub mod scale;
pub mod state;
pub mod time;
pub mod vehicle;

pub use ids::{
    AgencyId, DivisionId, FacilityId, IdCounters, MissionId, PersonId, ProgrammeId,
    ResearchProjectId, SubjectId, TestCampaignId, VehicleId,
};
pub use state::{Agency, CampaignState};
