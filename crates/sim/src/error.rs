//! The engine's error types.
//!
//! Two kinds of failure, and the difference matters:
//!
//! * A [`OrderError`] means the player asked for something that could not be
//!   done. It is **information for the player**, delivered through the Instant
//!   report. The Instant still resolves, and the rejected order changes
//!   nothing at all — not a partial effect, not a silent clamp.
//! * A [`SimError`] means the Instant could not be resolved. The state is left
//!   untouched, because the commit is atomic, and the player is told.
//!
//! See `docs/SIMULATION.md` §4.1 and §2.2.

use domain::PersonId;
use thiserror::Error;

/// An order the player issued that could not be honoured.
///
/// The messages are written to be read by a player, not by a developer. "Inspector
/// Okonjo is already assigned to the Lunar Landing programme" tells the player
/// what to do next; "person already assigned" does not.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OrderError {
    /// The person named in the order no longer exists.
    #[error("{name} is no longer with the agency.")]
    PersonGone {
        /// The person who was named.
        person: PersonId,
        /// Their name at the time of the order.
        name: String,
    },

    /// The person is already on what they have been put on.
    #[error("{name} is already on {current}.")]
    AlreadyAssigned {
        /// The name of the person.
        name: String,
        /// Where they are already assigned.
        current: String,
    },
    /// The person has no assignment and cannot be given the requested one.
    #[error("{name} cannot take that assignment.")]
    CannotAssign {
        /// The name of the person.
        name: String,
    },

    /// The named programme does not exist.
    #[error("There is no programme called {name}.")]
    NoSuchProgramme {
        /// The name of the programme.
        name: String,
    },

    /// The named facility does not exist.
    #[error("There is no facility called {name}.")]
    NoSuchFacility {
        /// The name of the facility.
        name: String,
    },

    /// The named vehicle does not exist.
    #[error("There is no vehicle called {name}.")]
    NoSuchVehicle {
        /// The name of the vehicle.
        name: String,
    },

    /// The programme is not in a state that allows this action.
    #[error("{name} is {status}, so that cannot be done now.")]
    WrongStatus {
        /// The name of the programme.
        name: String,
        /// The programme's current status, in the player's language.
        status: String,
    },

    /// The treasury cannot cover the cost.
    #[error("The treasury cannot cover {cost} — it holds {available}.")]
    InsufficientFunds {
        /// What the action costs.
        cost: String,
        /// What is available.
        available: String,
    },

    /// A required capability has not been unlocked yet.
    #[error("The agency has not yet developed {capability}.")]
    CapabilityUnavailable {
        /// The name of the capability.
        capability: String,
    },

    /// The order is well-formed but makes no sense in this situation.
    #[error("{reason}")]
    Rejected {
        /// Why it was rejected, in the player's language.
        reason: String,
    },
}

/// A failure that prevented a Instant from resolving.
///
/// The state is left untouched when this is returned: the commit is atomic, so
/// either the whole Instant happened or none of it did.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SimError {
    /// The saved campaign could not be read.
    #[error("The campaign could not be read: {reason}.")]
    CorruptState {
        /// What was wrong.
        reason: String,
    },

    /// The save was made under a different engine version.
    ///
    /// **The message says what to do about it**, because a refusal the player
    /// cannot act on is a refusal that only annoys them. The two versions are
    /// in the message so that a player reporting a bug can quote them.
    #[error(
        "This campaign was saved with a different version of the game (it is {found}, and this is {expected}), so it cannot be replayed here."
    )]
    VersionMismatch {
        /// The version this build of the game expects.
        expected: String,
        /// The version the save was made under.
        found: String,
    },

    /// The scenario named on the command line does not exist.
    #[error("There is no scenario called {name}.")]
    NoSuchScenario {
        /// The name of the scenario.
        name: String,
    },

    /// Something the engine does not expect went wrong.
    #[error("The simulation failed: {reason}")]
    Internal {
        /// What went wrong.
        reason: String,
    },
}
