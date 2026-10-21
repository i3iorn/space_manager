//! The screens the player moves between.
//!
//! The number assigned to each screen is **fixed** — staff is always `2`,
//! programmes always `3` — so a player who learns one screen's number keeps it.
//! Numbers are assigned in the order the player learns them, not the order the
//! screens were built.
//!
//! See `docs/UI.md` §3.

/// The screen the player is currently on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Screen {
    /// Where do I stand right now.
    Dashboard,
    /// The people, and why they are slow.
    Staff,
    /// The strategic layer.
    Programme,
    /// Research projects, test campaigns, and facilities.
    Research,
    /// Vehicles, missions, and the risk of launching them.
    Vehicles,
    /// The campaign's history and what is coming.
    Chronicle,
    /// Fork the campaign, and compare the two branches.
    Branches,
}

impl Screen {
    /// Every screen, in the order the number keys are assigned.
    pub const ALL: [Self; 7] = [
        Self::Dashboard,
        Self::Staff,
        Self::Programme,
        Self::Research,
        Self::Vehicles,
        Self::Chronicle,
        Self::Branches,
    ];

    /// The number key that jumps to this screen.
    #[must_use]
    pub const fn key(self) -> u8 {
        match self {
            Self::Dashboard => 1,
            Self::Staff => 2,
            Self::Programme => 3,
            Self::Research => 4,
            Self::Vehicles => 5,
            Self::Chronicle => 6,
            Self::Branches => 7,
        }
    }

    /// The screen a number key selects, if the key selects one.
    #[must_use]
    pub const fn from_key(key: u8) -> Option<Self> {
        match key {
            1 => Some(Self::Dashboard),
            2 => Some(Self::Staff),
            3 => Some(Self::Programme),
            4 => Some(Self::Research),
            5 => Some(Self::Vehicles),
            6 => Some(Self::Chronicle),
            7 => Some(Self::Branches),
            _ => None,
        }
    }

    /// The screen's title, shown in its tab or frame.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Staff => "Staff",
            Self::Programme => "Programme",
            Self::Research => "Research",
            Self::Vehicles => "Vehicles",
            Self::Chronicle => "Chronicle",
            Self::Branches => "Branches",
        }
    }

    /// The next screen in the cycle.
    #[must_use]
    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    /// The previous screen in the cycle.
    #[must_use]
    pub fn previous(self) -> Self {
        let index = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_screen_has_a_unique_number_key() {
        let mut keys: Vec<u8> = Screen::ALL.iter().map(|s| s.key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), Screen::ALL.len());
    }

    #[test]
    fn number_keys_round_trip() {
        for screen in Screen::ALL {
            assert_eq!(Screen::from_key(screen.key()), Some(screen));
        }
    }

    #[test]
    fn the_screen_cycle_returns_to_the_start() {
        let mut screen = Screen::Dashboard;
        for _ in 0..Screen::ALL.len() {
            screen = screen.next();
        }
        assert_eq!(screen, Screen::Dashboard);
    }

    #[test]
    fn the_screen_cycle_backwards_returns_to_the_start() {
        let mut screen = Screen::Dashboard;
        for _ in 0..Screen::ALL.len() {
            screen = screen.previous();
        }
        assert_eq!(screen, Screen::Dashboard);
    }
}
