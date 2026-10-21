//! The confirmation before the game quits.
//!
//! **A campaign is not saved.** The player has spent an evening on one, and
//! the keys that quit the game are two of the most-used keys on the keyboard.
//! A dialog that asks *are you sure* is the difference between a game that
//! protects the player's afternoon and one that does not.
//!
//! The dialog is a modal over the screen behind it, so the player can see the
//! agency they are about to leave. It is drawn last, over everything else,
//! because it is the last question the game asks.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme;

/// What the player answered.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    /// Stay in the campaign.
    Stay,
    /// Leave the game.
    Quit,
}

/// The confirmation's state.
///
/// **The safe answer is the one that starts selected.** A player who opens
/// the dialog and presses Enter without reading it has not quit, and that is
/// the mistake the dialog exists to catch.
#[derive(Clone, Copy, Debug)]
pub struct Confirmation {
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
    /// A confirmation with the safe answer selected.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            choice: Choice::Stay,
        }
    }

    /// The selected answer.
    #[must_use]
    pub const fn choice(&self) -> Choice {
        self.choice
    }

    /// Moves the selection to the other answer.
    pub fn move_selection(&mut self) {
        self.choice = match self.choice {
            Choice::Stay => Choice::Quit,
            Choice::Quit => Choice::Stay,
        };
    }

    /// Selects an answer outright.
    pub fn choose(&mut self, choice: Choice) {
        self.choice = choice;
    }

    /// Whether the player has chosen to leave.
    #[must_use]
    pub const fn is_quit(&self) -> bool {
        matches!(self.choice, Choice::Quit)
    }
}

/// Draws the confirmation as a modal.
pub fn render(frame: &mut Frame, area: Rect, confirmation: &Confirmation) {
    let popup = centred(area, 60, 40);

    frame.render_widget(Clear, popup);

    let mut lines = vec![
        Line::raw(""),
        Line::styled("  Leave the campaign?", theme::title()),
        Line::raw(""),
        Line::styled(
            "  The campaign is not saved. Quitting ends it.",
            theme::historical(),
        ),
        Line::raw(""),
    ];

    for choice in [Choice::Stay, Choice::Quit] {
        let selected = confirmation.choice() == choice;
        let marker = if selected {
            theme::FOCUS
        } else {
            theme::NO_FOCUS
        };
        let label = match choice {
            Choice::Stay => "Keep playing",
            Choice::Quit => "Quit",
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
        .block(Block::default().borders(Borders::ALL).title(" Quit "))
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
        assert!(!confirmation.is_quit());
    }

    #[test]
    fn the_selection_moves_both_ways() {
        let mut confirmation = Confirmation::new();
        confirmation.move_selection();
        assert_eq!(confirmation.choice(), Choice::Quit);
        assert!(confirmation.is_quit());
        confirmation.move_selection();
        assert_eq!(confirmation.choice(), Choice::Stay);
        assert!(!confirmation.is_quit());
    }
}
