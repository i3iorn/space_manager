//! Named random streams.
//!
//! The whole campaign draws from **one root seed**, chosen at campaign start,
//! shown to the player, and stored in the save. Streams are *derived*, never
//! shared:
//!
//! ```text
//! stream = PRNG(seed = expand(hash(root_seed, domain, bucket, discriminator)))
//! ```
//!
//! **Why derived streams matter [D]:** with a single sequential stream, adding
//! one random draw anywhere shifts every subsequent draw in the campaign — so a
//! new feature silently invalidates every replay, every golden test, and every
//! balance run. With named substreams, the `events` stream for Instant 41 is the
//! same number whether or not the research system gained a new roll.
//!
//! # Pinned algorithms
//!
//! Two things here are pinned in code rather than taken from a dependency,
//! because Rust does not guarantee their stability across releases:
//!
//! * **The hash** is FNV-1a, written out below. It is deliberately *not*
//!   `std::collections::hash_map::DefaultHasher`, whose algorithm Rust reserves
//!   the right to change.
//! * **The seed expansion** is `SplitMix64`, also written out below. It is
//!   deliberately *not* `SeedableRng::seed_from_u64`, whose algorithm is not
//!   stability-guaranteed.
//!
//! The generator itself is `ChaCha12Rng`, whose algorithm is specified by the
//! `ChaCha` construction, and it is seeded through `from_seed`, which is part of
//! that specification.
//!
//! See `docs/SIMULATION.md` §3.2.

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha12Rng;

/// The FNV-1a 64-bit offset basis.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// The FNV-1a 64-bit prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The `SplitMix64` increment, the "golden gamma" constant.
const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// The system which is drawing a random number.
///
/// The domain is part of the stream key, so two systems drawing in the same
/// Instant never interfere with each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Domain {
    /// Random events drawn each Instant.
    Events,
    /// Mission success and failure.
    Mission,
    /// Research breakthroughs.
    Research,
    /// Personnel outcomes — attrition, health, performance noise.
    Personnel,
    /// Financial variation.
    Finance,
    /// The wider world — rivals, suppliers, markets.
    World,
    /// Procedural generation of the campaign's starting state.
    Generation,
}

impl Domain {
    /// The stable byte tag used in the stream key.
    ///
    /// These strings are part of the stream key and therefore part of the
    /// campaign's identity. **Changing one changes every replay.**
    const fn tag(self) -> &'static [u8] {
        match self {
            Self::Events => b"events",
            Self::Mission => b"mission",
            Self::Research => b"research",
            Self::Personnel => b"personnel",
            Self::Finance => b"finance",
            Self::World => b"world",
            Self::Generation => b"generation",
        }
    }
}

/// The campaign's random source.
///
/// Holds the root seed and derives an independent generator for every
/// `(domain, bucket, discriminator)` triple.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng {
    root_seed: u64,
}

impl Rng {
    /// Creates a random source from the campaign's root seed.
    #[must_use]
    pub const fn new(root_seed: u64) -> Self {
        Self { root_seed }
    }

    /// The campaign's root seed.
    #[must_use]
    pub const fn root_seed(self) -> u64 {
        self.root_seed
    }

    /// Derives the 64-bit seed for one stream.
    ///
    /// The key is encoded canonically as
    /// `root_seed || domain_len || domain_bytes || bucket || discriminator`,
    /// with every integer little-endian and the domain length included so that
    /// no two distinct keys can produce the same byte sequence.
    ///
    /// **The bucket must be a stable identifier, never a time [D].** A bucket
    /// that is an instant makes the stream depend on *when* the draw happened,
    /// which is exactly what must not vary — and the event check's interval
    /// tightens under stress, so its instants move while its index does not.
    /// The buckets in use are the month index, the event check's index, and an
    /// entity's id.
    #[must_use]
    pub fn derive_seed(&self, domain: Domain, bucket: u64, discriminator: u64) -> u64 {
        let tag = domain.tag();
        let mut bytes = Vec::with_capacity(8 + 8 + tag.len() + 8 + 8);
        bytes.extend_from_slice(&self.root_seed.to_le_bytes());
        bytes.extend_from_slice(&(tag.len() as u64).to_le_bytes());
        bytes.extend_from_slice(tag);
        bytes.extend_from_slice(&bucket.to_le_bytes());
        bytes.extend_from_slice(&discriminator.to_le_bytes());
        fnv1a(&bytes)
    }

    /// The generator for one stream.
    ///
    /// Each call returns an independent generator positioned at the start of its
    /// own sequence, so the same triple always produces the same numbers.
    #[must_use]
    pub fn stream(&self, domain: Domain, bucket: u64, discriminator: u64) -> ChaCha12Rng {
        let seed = self.derive_seed(domain, bucket, discriminator);
        ChaCha12Rng::from_seed(expand_seed(seed))
    }
}

/// FNV-1a, 64-bit.
///
/// Pinned here rather than taken from `DefaultHasher`, whose algorithm Rust
/// reserves the right to change between releases.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Expands a 64-bit seed into a 32-byte generator seed using `SplitMix64`.
///
/// `SplitMix64` is a fully specified algorithm, so this expansion is stable
/// forever. `SeedableRng::seed_from_u64` is not: its algorithm is not
/// stability-guaranteed across releases of `rand`.
fn expand_seed(seed: u64) -> [u8; 32] {
    let mut out = [0_u8; 32];
    let mut state = seed;
    for chunk in out.chunks_exact_mut(8) {
        state = state.wrapping_add(GOLDEN_GAMMA);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        chunk.copy_from_slice(&z.to_le_bytes());
    }
    out
}

/// Draws a uniform integer in `0..=10_000`, the scale the simulation uses for
/// probabilities.
///
/// Probabilities are stored in basis points, so a 14% risk is exactly 1,400 and
/// the comparison is exact rather than floating-point.
#[must_use]
pub fn draw_basis_points(rng: &mut ChaCha12Rng) -> u32 {
    rng.random_range(0..=10_000_u32)
}

/// Draws a uniform integer in `0..n`.
///
/// `n` must be non-zero.
#[must_use]
pub fn draw_below(rng: &mut ChaCha12Rng, n: u32) -> u32 {
    assert!(n > 0, "draw_below requires a non-zero bound");
    rng.random_range(0..n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The expected seeds for a fixed root seed.
    ///
    /// These are golden values. They exist to catch a change to the hash, the
    /// expansion, or the key encoding — any of which would silently invalidate
    /// every replay and every golden test. If this test fails, the change was
    /// either a mistake or a deliberate one that requires regenerating the
    /// campaign fixtures.
    const ROOT_SEED: u64 = 6_125_084_923_715_548;

    #[test]
    fn derived_seeds_match_the_golden_values() {
        let rng = Rng::new(ROOT_SEED);
        let expected: [(Domain, u64, u64, u64); 4] = [
            (Domain::Events, 0, 0, 16_467_730_958_536_966_810),
            (Domain::Events, 1, 0, 3_712_997_812_928_092_891),
            (Domain::Mission, 7, 0, 17_406_483_081_410_981_127),
            (Domain::Personnel, 12, 3, 3_341_387_576_758_544_575),
        ];
        for (domain, bucket, discriminator, want) in expected {
            let got = rng.derive_seed(domain, bucket, discriminator);
            assert_eq!(got, want, "seed for {domain:?}/{bucket}/{discriminator}");
        }
    }

    #[test]
    fn the_bucket_is_not_a_time() {
        // **The bucket must be a stable identifier, never an instant.** Two
        // campaigns that reach the same state at different instants must draw
        // the same numbers, and the way to guarantee that is to key on something
        // that identifies the draw rather than when it happened.
        //
        // The month index and the event check's index are the buckets in use,
        // and both are counts — never milliseconds.
        let rng = Rng::new(99);
        let first = rng.derive_seed(Domain::Events, 41, 0);
        let second = rng.derive_seed(Domain::Events, 41, 0);
        assert_eq!(first, second, "the same bucket gives the same seed");
    }
    #[test]
    fn distinct_keys_produce_distinct_seeds() {
        let rng = Rng::new(1);
        let a = rng.derive_seed(Domain::Events, 1, 0);
        let b = rng.derive_seed(Domain::Events, 2, 0);
        let c = rng.derive_seed(Domain::Events, 1, 1);
        let d = rng.derive_seed(Domain::Mission, 1, 0);
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn the_same_key_produces_the_same_sequence() {
        let rng = Rng::new(99);
        let mut first = rng.stream(Domain::Events, 5, 0);
        let mut second = rng.stream(Domain::Events, 5, 0);
        for _ in 0..16 {
            assert_eq!(
                draw_basis_points(&mut first),
                draw_basis_points(&mut second)
            );
        }
    }

    #[test]
    fn different_domains_do_not_interfere() {
        let rng = Rng::new(99);
        let mut events = rng.stream(Domain::Events, 5, 0);
        let mut research = rng.stream(Domain::Research, 5, 0);
        let events_first: Vec<u32> = (0..8).map(|_| draw_basis_points(&mut events)).collect();
        let research_first: Vec<u32> = (0..8).map(|_| draw_basis_points(&mut research)).collect();
        assert_ne!(events_first, research_first);

        // Drawing from one stream must not shift the other.
        let mut events_again = rng.stream(Domain::Events, 5, 0);
        let events_again_first: Vec<u32> = (0..8)
            .map(|_| draw_basis_points(&mut events_again))
            .collect();
        assert_eq!(events_first, events_again_first);
    }

    #[test]
    fn basis_points_stay_in_range() {
        let mut rng = Rng::new(4_242).stream(Domain::Finance, 1, 0);
        for _ in 0..1_000 {
            let value = draw_basis_points(&mut rng);
            assert!(value <= 10_000);
        }
    }
}
