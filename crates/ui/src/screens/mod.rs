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
pub mod research;
pub mod staff;
pub mod vehicles;

use crate::app::App;
use crate::screen::Screen;

/// How many items the player can move through on a screen.
///
/// **A list the player can walk off the end of is a list that loses the
/// focus**, and the focus is the one thing the player must never lose: it is
/// how they say which person they mean. A screen with no list of its own has
/// no items, and its selection is always the first one.
#[must_use]
pub fn item_count(app: &App, screen: Screen) -> usize {
    match screen {
        Screen::Staff => staff::rows(app).len(),
        Screen::Programme => programme::objective_count(app),
        // **The branches screen's list is the decisions the player has made.**
        Screen::Branches => app.decisions().len(),
        _ => 0,
    }
}

/// The sort keys a screen offers.
///
/// **A screen with no list of its own has no sort keys**, and the sort cycles
/// through nothing, which is the same as not offering a sort at all.
#[must_use]
pub fn sort_keys(screen: Screen) -> &'static [&'static str] {
    match screen {
        Screen::Staff => staff::SORTS,
        Screen::Research => research::SORTS,
        Screen::Chronicle => chronicle::SORTS,
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
