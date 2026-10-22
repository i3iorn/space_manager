//! The confirmation before the game does something irreversible.
//!
//! **Quitting is not the same as losing.** The game saves itself — at every
//! month boundary, and when the player quits — so leaving is a decision about
//! when to stop and not a decision about whether to throw the evening away.
//! The dialog says so, because a player who believes quitting loses the
//! campaign will not quit when they should.
//!
//! **Loading a saved campaign throws the current one away**, and that is the
//! same kind of decision, so it is the same dialog.
//!
//! The dialog is a modal over the screen behind it, so the player can see the
//! agency they are about to leave. It is drawn last, over everything else,
//! because it is the last question the game asks.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme;

/// What the player is being asked.
///
/// **The dialog asks about one thing at a time**, and what it says depends on
/// what it is asking.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Purpose {
    /// Leaving the game.
    Quit,
    /// Replacing the campaign with the one in a file.
    Load {
        /// The file the campaign would come from.
        path: String,
    },
}

/// What the player answered.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    /// Stay in the campaign.
    Stay,
    /// Do the thing that was asked about.
    Go,
}

/// The confirmation's state.
///
/// **The safe answer is the one that starts selected.** A player who opens the
/// dialog and presses Enter without reading it has not quit, and that is the
/// mistake the dialog exists to catch.
#[derive(Clone, Debug)]
pub struct Confirmation {
    /// What the player is being asked.
    purpose: Purpose,
    /// Which answer is selected.
    choice: Choice,
}

impl Default for Confirmation {
    /// **The safe answer is the one that starts selected.**
    fn default() -> Self {
        Self::new()
    }
}

impl Confirmation {
    /// A confirmation to quit, with the safe answer selected.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            purpose: Purpose::Quit,
            choice: Choice::Stay,
        }
    }

    /// A confirmation to load a campaign, with the safe answer selected.
    #[must_use]
    pub fn load(path: String) -> Self {
        Self {
            purpose: Purpose::Load { path },
            choice: Choice::Stay,
        }
    }

    /// What the player is being asked.
    #[must_use]
    pub const fn purpose(&self) -> &Purpose {
        &self.purpose
    }

    /// The selected answer.
    #[must_use]
    pub const fn choice(&self) -> Choice {
        self.choice
    }

    /// Moves the selection to the other answer.
    pub fn move_selection(&mut self) {
        self.choice = match self.choice {
            Choice::Stay => Choice::Go,
            Choice::Go => Choice::Stay,
        };
    }

    /// Selects an answer outright.
    pub fn choose(&mut self, choice: Choice) {
        self.choice = choice;
    }

    /// Whether the player has chosen to go ahead.
    #[must_use]
    pub const fn is_go(&self) -> bool {
        matches!(self.choice, Choice::Go)
    }
}

/// Draws the confirmation as a modal.
pub fn render(frame: &mut Frame, area: Rect, confirmation: &Confirmation) {
    let popup = centred(area, 60, 40);

    frame.render_widget(Clear, popup);

    let (title, question, warning, go_label) = match confirmation.purpose() {
        Purpose::Quit => (
            " Quit ",
            "Leave the campaign?",
            "The campaign is saved when you quit, and the game saves itself every month. \
             You can carry on from it."
                .to_owned(),
            "Quit",
        ),
        Purpose::Load { path } => (
            " Load ",
            "Load the saved campaign?",
            format!(
                "The campaign in {path} replaces the one you are playing. Anything you have done since is lost."
            ),
            "Load",
        ),
    };

    let mut lines = vec![
        Line::raw(""),
        Line::styled(format!("  {question}"), theme::title()),
        Line::raw(""),
        Line::styled(format!("  {warning}"), theme::historical()),
        Line::raw(""),
    ];

    for choice in [Choice::Stay, Choice::Go] {
        let selected = confirmation.choice() == choice;
        let marker = if selected {
            theme::FOCUS
        } else {
            theme::NO_FOCUS
        };
        let label = match choice {
            Choice::Stay => "Keep playing",
            Choice::Go => go_label,
        };
        let style = if selected {
            theme::selected()
        } else {
            theme::historical()
        };
        lines.push(Line::from(vec![
            Span::raw(format!("  {marker} ")),
            Span::styled(format!("{label:<14}"), style),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  Enter to choose, Esc to stay, Ctrl+C to leave now",
        theme::historical(),
    ));

    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(body, popup);
}

/// A rectangle of the given percentage size, centred.
fn centred(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_safe_answer_starts_selected() {
        // **The mistake the dialog exists to catch** is a player who opens it
        // and presses Enter without reading it.
        let confirmation = Confirmation::new();
        assert_eq!(confirmation.choice(), Choice::Stay);
        assert!(!confirmation.is_go());
    }

    #[test]
    fn the_selection_moves_both_ways() {
        let mut confirmation = Confirmation::new();
        confirmation.move_selection();
        assert_eq!(confirmation.choice(), Choice::Go);
        assert!(confirmation.is_go());
        confirmation.move_selection();
        assert_eq!(confirmation.choice(), Choice::Stay);
        assert!(!confirmation.is_go());
    }

    #[test]
    fn the_load_confirmation_is_also_safe_to_press_enter_on() {
        // **The same rule, for the same reason.** Loading a campaign throws
        // the current one away, and a player who presses Enter without reading
        // must not lose it.
        let confirmation = Confirmation::load("campaign.json".to_owned());
        assert_eq!(confirmation.choice(), Choice::Stay);
        assert!(!confirmation.is_go());
        assert_eq!(
            confirmation.purpose(),
            &Purpose::Load {
                path: "campaign.json".to_owned()
            }
        );
    }
}
