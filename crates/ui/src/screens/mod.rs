//! The screens.
//!
//! One module per screen. A screen renders itself and owns the keys it handles;
//! everything global lives in `app`.

pub mod branches;
pub mod chronicle;
pub mod dashboard;
pub mod keymap;
pub mod manual;
pub mod programme;
pub mod report;
pub mod research;
pub mod staff;
pub mod vehicles;

use crate::screen::Screen;

/// The sort keys a screen offers.
///
/// **A screen with no list of its own has no sort keys**, and the sort cycles
/// through nothing, which is the same as not offering a sort at all.
#[must_use]
pub fn sort_keys(screen: Screen) -> &'static [&'static str] {
    match screen {
        Screen::Staff => staff::SORTS,
        _ => &[],
    }
}

/// The topic the info line shows for a screen.
///
/// **Each screen has one thing it is for**, and the info line says what it is.
/// On the staff screen that is the binding factor, because that is the thing
/// the player has to understand to understand the game; on the dashboard it is
/// the agenda, because that is what the dashboard is for.
#[must_use]
pub fn focus_topic(screen: Screen) -> &'static str {
    match screen {
        Screen::Dashboard => "screen.dashboard.due",
        Screen::Staff => "screen.staff.performance",
        Screen::Programme => "screen.programme",
        Screen::Research => "screen.research.projects",
        Screen::Vehicles => "screen.vehicles",
        Screen::Chronicle => "screen.chronicle",
        Screen::Branches => "screen.branches",
    }
}
