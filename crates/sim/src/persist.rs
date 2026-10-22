//! Save, load, and version checks.
//!
//! A save carries four things: the engine version, the root seed, the campaign
//! state, and the **order log**.
//!
//! The log is the source of truth for what the player did; the state is a cache
//! of it. That is what makes exact replay and forking possible, and it is why
//! the log records *intent* rather than effect: a standing order is written
//! once, at the Instant it was issued, and re-evaluated internally every Instant.
//!
//! See `docs/PERSISTENCE.md` for the format and the versioning rules.

use domain::state::CampaignState;
use domain::time::Instant;

use crate::scheduler::Scheduler;
use serde::{Deserialize, Serialize};

use crate::error::SimError;

/// The save **format** version — how the file itself is laid out.
///
/// Independent of the **engine** version, which is whether the outcomes are
/// reproducible. A save can be readable by a newer engine and still be
/// un-replayable.
///
/// **It was 2 when the scheduler joined the save**, **3 when the orders in
/// flight joined the state**, **4 when the shift joined a person**, **5 when
/// the remainder joined an objective**, **6 when the condition of a
/// facility was scaled**, **7 when the testing joined the state**, **8 when
/// the knowledge became per discipline rather than per capability**, **9
/// when the policy order became a standing instruction and the shift became
/// an override of it**, **10 when a shift change could put somebody back
/// on the policy**, and **11 when the shift policy became a thermostat with a
/// margin instead of a comparison that flipped every hour**. A file of an older
/// format is refused rather than half-read.
pub const SAVE_FORMAT: u32 = 11;

/// The engine version.
///
/// **A hash over everything that can change an outcome**: the resolution order,
/// the integration quantum, the order kinds, the scheduled kinds, the RNG
/// algorithm, and the stream-hash function. It is computed by
/// [`crate::version::sim_version`] rather than written down by hand, so a
/// change to any of the inputs is picked up automatically — and a save made
/// under a different set is refused rather than silently producing a different
/// campaign.
#[must_use]
pub fn sim_version() -> String {
    crate::version::sim_version()
}

/// One entry in the order log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggedOrder {
    /// The Instant the player issued the order.
    pub issued_at: Instant,
    /// A monotonic counter, so the order of orders is never ambiguous.
    pub seq: u64,
    /// The order kind, as a stable name.
    pub kind: String,
    /// The order's parameters.
    pub payload: serde_json::Value,
}

/// A hash over the whole campaign state.
///
/// The procedure is fixed and documented: the state is serialised to JSON — in
/// which every map is ordered by its key and every struct is in declaration
/// order — and the bytes are hashed with FNV-1a. The same procedure is used by
/// the determinism test, the golden tests, and the replay test, so they all
/// agree on what "the same campaign" means.
///
/// **No floating point takes part.** The campaign state holds no floats, and
/// this is one of the reasons it must not.
#[must_use]
pub fn state_hash(state: &CampaignState) -> u64 {
    let bytes = serde_json::to_vec(state).expect("the campaign state always serialises");
    fnv1a(&bytes)
}

/// FNV-1a, 64-bit.
///
/// The same hash the random streams use, and pinned for the same reason.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// The contents of a save file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveEnvelope {
    /// The save format version.
    pub format: u32,
    /// The engine version the save was made under.
    pub sim_version: String,
    /// The instant the campaign had reached when it was saved.
    pub now: Instant,
    /// The campaign's root seed.
    pub root_seed: u64,
    /// The campaign state.
    pub state: CampaignState,
    /// The scenario the campaign started from.
    ///
    /// **A campaign has to be forkable after it is loaded**, and forking
    /// replays the log from the beginning — and the beginning is the
    /// scenario's start state.
    pub scenario: crate::scenario::Scenario,
    /// The order log - every decision the player ever made.
    pub log: Vec<LoggedOrder>,
    /// Everything still waiting to happen.
    ///
    /// **The scheduler is part of the save [D].** A campaign saved with a
    /// launch in flight must restore that launch, or the campaign silently
    /// changes.
    pub scheduler: Scheduler,
}

impl SaveEnvelope {
    /// Builds an envelope for a campaign.
    #[must_use]
    pub fn new(
        now: Instant,
        root_seed: u64,
        state: CampaignState,
        scenario: crate::scenario::Scenario,
        log: Vec<LoggedOrder>,
        scheduler: Scheduler,
    ) -> Self {
        Self {
            format: SAVE_FORMAT,
            sim_version: sim_version(),
            now,
            root_seed,
            state,
            scenario,
            log,
            scheduler,
        }
    }

    /// Serialises the envelope to JSON.
    ///
    /// # Errors
    ///
    /// Returns [`SimError::Internal`] if the state cannot be serialised.
    pub fn to_json(&self) -> Result<String, SimError> {
        serde_json::to_string_pretty(self).map_err(|error| SimError::Internal {
            reason: format!("the campaign could not be written: {error}"),
        })
    }

    /// Reads an envelope from JSON, validating it in order.
    ///
    /// The checks run in the order a player would need them explained: the file
    /// parses, the format is supported, the engine version matches, and the
    /// state is intact. A damaged save must never crash the game.
    ///
    /// # Errors
    ///
    /// Returns a [`SimError`] describing the first problem found.
    pub fn from_json(text: &str) -> Result<Self, SimError> {
        let envelope: Self =
            serde_json::from_str(text).map_err(|error| SimError::CorruptState {
                reason: format!("the file is not a campaign: {error}"),
            })?;

        // **A file of any other format is refused rather than half-read.**
        // An older file is the dangerous one: it may have the same bytes in the
        // same places and mean something else, which is exactly what happened
        // when the knowledge was per capability rather than per discipline.
        if envelope.format != SAVE_FORMAT {
            return Err(SimError::CorruptState {
                reason: if envelope.format > SAVE_FORMAT {
                    format!(
                        "the campaign was saved in format {}, but this version reads format {SAVE_FORMAT}",
                        envelope.format
                    )
                } else {
                    format!(
                        "the campaign was saved in format {}, and this version reads format {SAVE_FORMAT}. \
                         It was made by an older build of the game, and it cannot be read safely.",
                        envelope.format
                    )
                },
            });
        }

        if envelope.sim_version != sim_version() {
            return Err(SimError::VersionMismatch {
                expected: sim_version(),
                found: envelope.sim_version,
            });
        }

        Ok(envelope)
    }

    /// Whether the campaign can be replayed under the running engine.
    #[must_use]
    pub fn is_replayable(&self) -> bool {
        self.sim_version == sim_version()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SaveEnvelope {
        let mut state = CampaignState::new(domain::state::Agency::new(
            domain::ids::AgencyId(0),
            "SEA".to_owned(),
            2039,
        ));
        state.treasury.balance = domain::money::Money::from_credits(190_000);
        SaveEnvelope::new(
            Instant::from_months(7),
            6_125_084_923_715_548,
            state,
            crate::scenario::Scenario::first_year(),
            Vec::new(),
            Scheduler::new(),
        )
    }

    #[test]
    fn the_state_hash_is_stable_for_the_same_state() {
        let envelope = sample();
        assert_eq!(state_hash(&envelope.state), state_hash(&envelope.state));
    }

    #[test]
    fn the_state_hash_changes_when_the_state_changes() {
        let mut envelope = sample();
        let before = state_hash(&envelope.state);
        envelope.state.treasury.balance = domain::money::Money::from_credits(1);
        assert_ne!(state_hash(&envelope.state), before);
    }

    #[test]
    fn an_envelope_round_trips_through_json() {
        let original = sample();
        let text = original.to_json().expect("the envelope serialises");
        let restored = SaveEnvelope::from_json(&text).expect("the envelope deserialises");
        assert_eq!(restored, original);
    }

    #[test]
    fn a_garbled_file_is_rejected_rather_than_crashing() {
        let result = SaveEnvelope::from_json("{ this is not json");
        assert!(matches!(result, Err(SimError::CorruptState { .. })));
    }

    #[test]
    fn a_newer_format_is_refused_with_an_explanation() {
        let mut envelope = sample();
        envelope.format = SAVE_FORMAT + 1;
        let text = envelope.to_json().expect("the envelope serialises");
        let result = SaveEnvelope::from_json(&text);
        assert!(matches!(result, Err(SimError::CorruptState { .. })));
    }

    #[test]
    fn an_older_format_is_refused_rather_than_half_read() {
        // **The older file is the dangerous one.** It may have the same bytes
        // in the same places and mean something else, which is what happened
        // when the knowledge was per capability rather than per discipline.
        let mut envelope = sample();
        envelope.format = SAVE_FORMAT - 1;
        let text = envelope.to_json().expect("the envelope serialises");
        let result = SaveEnvelope::from_json(&text);
        let Err(SimError::CorruptState { reason }) = result else {
            panic!("an older file must be refused");
        };
        assert!(
            reason.contains("older build of the game"),
            "and the refusal says why: {reason}"
        );
    }

    #[test]
    fn a_foreign_engine_version_is_refused() {
        let mut envelope = sample();
        envelope.sim_version = "some-other-build".to_owned();
        let text = envelope.to_json().expect("the envelope serialises");
        let result = SaveEnvelope::from_json(&text);
        assert!(matches!(result, Err(SimError::VersionMismatch { .. })));
    }
}
