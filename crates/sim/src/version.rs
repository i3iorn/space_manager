//! What decides what a campaign does.
//!
//! **The version is a hash of the behaviour-defining inputs.** A change to any
//! of them changes every replay: the same seed and the same orders would
//! produce a different campaign. A save made under a different set of inputs
//! must therefore be **refused**, not silently loaded into a campaign that
//! quietly diverges from the one that was saved.
//!
//! The inputs are:
//!
//! * **The phase order.** The order the ten phases run in is the architecture.
//! * **The integration quantum.** The grid the continuous rates are applied
//!   on. It does not change the *total* of a rate - the remainder makes that
//!   exact - but it changes the *path*, and the path decides a project's
//!   completion day.
//! * **The random algorithms.** The hash, the seed expansion, and the key
//!   encoding. These are pinned in code, and the tag says so.
//! * **The order kinds and the scheduled kinds.** A new kind, a renamed kind,
//!   or a changed priority changes what a campaign does.
//!
//! See `docs/PERSISTENCE.md` §4.

use crate::rates;
use crate::tick;

/// The tag for the pinned random algorithms.
///
/// The hash is `FNV-1a`, the expansion is `SplitMix64`, and the generator is
/// `ChaCha12`. If any of them changes, every replay changes, and this tag must
/// change with them.
pub const RNG_ALGORITHM_VERSION: &str = "fnv1a/splitmix64/chacha12/v1";

/// The behaviour-defining inputs, as a list of strings.
///
/// The list is hashed to make the version. It is exposed so that a change to
/// it is visible in a diff and in a test failure.
#[must_use]
pub fn inputs() -> Vec<String> {
    let mut inputs = vec![
        format!("phases:{}", tick::PHASE_ORDER.join(",")),
        format!("quantum:{}", rates::QUANTUM.millis()),
        format!("rng:{RNG_ALGORITHM_VERSION}"),
        format!("orders:{}", domain::order::order_kind_names().join(",")),
        format!("scheduled:{}", scheduled_kinds().join(",")),
    ];
    inputs.sort();
    inputs
}

/// The engine version, as a short hash of the inputs.
///
/// A save carries this, and a load refuses a save whose version does not match
/// — because the campaign it describes cannot be reproduced.
#[must_use]
pub fn sim_version() -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for input in inputs() {
        for byte in input.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The names of the scheduled kinds, in a stable order.
fn scheduled_kinds() -> Vec<&'static str> {
    vec![
        "MonthBoundary",
        "CommitmentDue",
        "OrderLands",
        "MissionPhase",
        "ProjectComplete",
        "VehicleComplete",
        "AttritionCheck",
        "EventCheck",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_is_stable() {
        assert_eq!(sim_version(), sim_version());
        assert_eq!(sim_version().len(), 16);
    }

    #[test]
    fn the_version_covers_the_integration_quantum() {
        // **The quantum changes the path of every rate**, and the path decides
        // when a project finishes. A change to it must change the version.
        let inputs = inputs();
        assert!(
            inputs.iter().any(|input| input.starts_with("quantum:")),
            "the quantum must be one of the version's inputs: {inputs:?}"
        );
    }

    #[test]
    fn the_version_covers_the_phase_order() {
        let inputs = inputs();
        assert!(
            inputs.iter().any(|input| input.starts_with("phases:")),
            "the phase order must be one of the version's inputs: {inputs:?}"
        );
    }

    #[test]
    fn the_order_kinds_are_listed() {
        let kinds = domain::order::order_kind_names();
        assert!(kinds.contains(&"Hire"));
        assert!(kinds.contains(&"BuildFacility"));
    }
}
