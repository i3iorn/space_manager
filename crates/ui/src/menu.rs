//! The main menu: the things the player does to the campaign, not with it.
//!
//! **The seven screens are the game; the menu is the system.** Restarting,
//! saving, loading, and changing the clock are decisions about the campaign as a
//! whole, and they do not belong on a screen about staff or money. The menu is a
//! modal over whatever screen the player is on, so they can look at the agency
//! while they decide what to do with it.
//!
//! **It is not the first thing the player sees.** The game opens with the
//! briefing and the campaign, because a main menu between the player and the
//! game is a screen they have to dismiss before they can start. The menu is one
//! key away, `M`, and it is there when they want it.
//!
//! See `docs/UI.md` §3.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme;

/// The things the main menu offers.
///
/// **The order is the order of the decision.** The player opens the menu to do
/// something, and the things that change the campaign come before the thing that
/// leaves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    /// Start the campaign again from its scenario.
    Restart,
    /// Write the campaign to its file.
    Save,
    /// Replace the campaign with the one in its file.
    Load,
    /// Change the clock's dilation.
    Settings,
    /// Leave the game.
    Quit,
    /// Close the menu and go back to the screen behind it.
    Back,
}

impl Item {
    /// Every item, in the order they are shown.
    pub const ALL: [Self; 6] = [
        Self::Restart,
        Self::Save,
        Self::Load,
        Self::Settings,
        Self::Quit,
        Self::Back,
    ];

    /// The item's label, as the player reads it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Restart => "Restart the campaign",
            Self::Save => "Save the campaign",
            Self::Load => "Load the saved campaign",
            Self::Settings => "Change settings",
            Self::Quit => "Quit the game",
            Self::Back => "Back to the game",
        }
    }

    /// The line that says what choosing the item does.
    #[must_use]
    pub const fn detail(self) -> &'static str {
        match self {
            Self::Restart => {
                "A new campaign from the same scenario. The one you are playing is lost."
            }
            Self::Save => "Write the campaign to its file now.",
            Self::Load => "Replace the campaign with the one in its file.",
            Self::Settings => "The clock's dilation. The campaign does not care how fast it runs.",
            Self::Quit => "The campaign is saved when you quit.",
            Self::Back => "Close the menu. The campaign carries on.",
        }
    }
}

/// The main menu's state.
///
/// **The safe answer is the one that starts selected**, the same rule as the
/// confirmation dialog: a player who opens the menu and presses Enter without
/// reading it has not restarted anything.
#[derive(Clone, Debug)]
pub struct Menu {
    /// Which item is selected.
    selected: usize,
    /// Whether the settings sub-panel is showing.
    settings_open: bool,
}

impl Default for Menu {
    /// **The safe answer is the one that starts selected.**
    fn default() -> Self {
        Self::new()
    }
}

impl Menu {
    /// A menu with the safe answer selected.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: Item::ALL.len() - 1,
            settings_open: false,
        }
    }

    /// The selected item.
    #[must_use]
    pub fn item(&self) -> Item {
        Item::ALL[self.selected.min(Item::ALL.len() - 1)]
    }

    /// Which item is selected, as a position.
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }

    /// Moves the selection up, and stops at the top.
    pub fn up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Moves the selection down, and stops at the bottom.
    pub fn down(&mut self) {
        self.selected = (self.selected + 1).min(Item::ALL.len() - 1);
    }

    /// Whether the settings sub-panel is showing.
    #[must_use]
    pub const fn settings_open(&self) -> bool {
        self.settings_open
    }

    /// Opens the settings sub-panel.
    pub fn open_settings(&mut self) {
        self.settings_open = true;
    }

    /// Closes the settings sub-panel, and leaves the menu open.
    pub fn close_settings(&mut self) {
        self.settings_open = false;
    }
}

/// Draws the main menu as a modal.
pub fn render(frame: &mut Frame, area: Rect, menu: &Menu, dilation: &str, paused: bool) {
    let popup = centred(area, 70, 60);

    frame.render_widget(Clear, popup);

    if menu.settings_open() {
        render_settings(frame, popup, dilation, paused);
        return;
    }

    let mut lines = vec![Line::raw("")];
    for item in Item::ALL {
        let selected = menu.item() == item;
        let marker = if selected {
            theme::FOCUS
        } else {
            theme::NO_FOCUS
        };
        let style = if selected {
            theme::selected()
        } else {
            theme::historical()
        };
        lines.push(Line::from(vec![
            Span::raw(format!("  {marker} ")),
            Span::styled(format!("{:<26}", item.label()), style),
        ]));
        lines.push(Line::styled(
            format!("      {}", item.detail()),
            theme::historical(),
        ));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  Enter to choose, Esc to close, M to close",
        theme::Meaning::Informational.style(),
    ));

    let body = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Menu - the campaign, not the screen "),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(body, popup);
}

/// Draws the settings sub-panel.
///
/// **The settings are the host's, not the campaign's.** The only one that is the
/// player's to change is how fast the clock runs, and it is shown here so the
/// player can find it without knowing the shortcut.
fn render_settings(frame: &mut Frame, area: Rect, dilation: &str, paused: bool) {
    let state = if paused { "paused" } else { "running" };
    let lines = vec![
        Line::raw(""),
        Line::styled("  Clock", theme::title()),
        Line::raw(""),
        Line::from(vec![
            Span::raw("    Dilation: "),
            Span::styled(dilation.to_owned(), theme::emphasis()),
        ]),
        Line::from(vec![
            Span::raw("    State:    "),
            Span::styled(state.to_owned(), theme::emphasis()),
        ]),
        Line::raw(""),
        Line::styled(
            "    + and - step the dilation; Space pauses.",
            theme::historical(),
        ),
        Line::raw(""),
        Line::styled(
            "  Esc to go back to the menu",
            theme::Meaning::Informational.style(),
        ),
    ];

    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Settings "))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });
    frame.render_widget(body, area);
}

/// A rectangle of the given percentage size, centred.
fn centred(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = area.height * height_percent / 100;
    let horizontal = area.width * width_percent / 100;
    Rect {
        x: area.x + area.width.saturating_sub(horizontal) / 2,
        y: area.y + area.height.saturating_sub(vertical) / 2,
        width: horizontal.max(1),
        height: vertical.max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_safe_answer_starts_selected() {
        // **The mistake the menu exists to catch** is a player who opens it and
        // presses Enter without reading it. Back is the safe answer.
        let menu = Menu::new();
        assert_eq!(menu.item(), Item::Back);
    }

    #[test]
    fn the_selection_walks_the_items_and_stops() {
        let mut menu = Menu::new();
        menu.up();
        assert_eq!(menu.item(), Item::Quit);
        for _ in 0..10 {
            menu.up();
        }
        assert_eq!(menu.item(), Item::Restart, "it stops at the top");
        for _ in 0..10 {
            menu.down();
        }
        assert_eq!(menu.item(), Item::Back, "and at the bottom");
    }

    #[test]
    fn the_settings_open_and_close() {
        let mut menu = Menu::new();
        assert!(!menu.settings_open());
        menu.open_settings();
        assert!(menu.settings_open());
        menu.close_settings();
        assert!(!menu.settings_open());
    }

    #[test]
    fn every_item_says_what_it_does() {
        for item in Item::ALL {
            assert!(!item.label().is_empty());
            assert!(!item.detail().is_empty());
        }
    }
}
