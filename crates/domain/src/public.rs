//! Public opinion and prestige.
//!
//! Public opinion moves fast and prestige moves slowly, which is how a public
//! can be enthusiastic about an agency that is quietly failing.
//! See `docs/DOMAIN_MODEL.md` §13.

use serde::{Deserialize, Serialize};

use crate::scale::signed_units_to_percent;

/// How the public feels about the agency.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PublicOpinion {
    /// The public's approval, from `-25_600` to `+25_600`.
    pub value: i16,
}

impl PublicOpinion {
    /// The public is indifferent.
    #[must_use]
    pub const fn neutral() -> Self {
        Self { value: 0 }
    }

    /// The opinion as a whole percentage.
    #[must_use]
    pub const fn percent(&self) -> i16 {
        signed_units_to_percent(self.value)
    }
}

/// The agency's standing in the world.
///
/// Prestige is slow to build and slow to lose, and it decays by a point a year.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Prestige {
    /// The agency's standing, from `-25_600` to `+25_600`.
    pub value: i16,
}

impl Prestige {
    /// No standing yet.
    #[must_use]
    pub const fn none() -> Self {
        Self { value: 0 }
    }

    /// The prestige as a whole percentage.
    #[must_use]
    pub const fn percent(&self) -> i16 {
        signed_units_to_percent(self.value)
    }
}

/// How the public sees the agency.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PublicState {
    /// What the public thinks.
    pub opinion: PublicOpinion,
    /// What the world thinks.
    pub prestige: Prestige,
}

impl PublicState {
    /// A new agency's public position.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            opinion: PublicOpinion::neutral(),
            prestige: Prestige::none(),
        }
    }
}

impl Default for PublicState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_agency_is_unknown_to_everyone() {
        let state = PublicState::new();
        assert_eq!(state.opinion.value, 0);
        assert_eq!(state.prestige.value, 0);
    }
}
