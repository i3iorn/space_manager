//! The dialog that puts a question to the player.
//!
//! **An event that asks nothing is a sentence; an event that asks something is
//! a decision.** The simulation holds the question, the window it stays open,
//! and what silence costs; this module is only the view. The world waits while
//! the dialog is up — a decision the campaign answers by itself while the
//! player is reading the question is a decision nobody made.
//!
//! The dialog is a modal over the screen behind it, the way the menu and the
//! confirmation are, because the question is about the campaign as a whole and
//! not about the screen the player happens to be on. It is drawn under the
//! confirmation: leaving the game outranks answering the question, because the
//! campaign saves itself and the question comes back with it.
//!
//! See `docs/UI.md` §3 and `docs/SIMULATION.md` §12.

use domain::event::{Decision, Effect};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme;

/// The decision dialog's own state.
///
/// **The only thing the dialog remembers is where the player is looking.** The
/// question itself is read from the simulation every time it is drawn, so a
/// dialog can never outlive the question it was showing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionDialog {
    /// Which answer is selected.
    selected: usize,
}

impl Default for DecisionDialog {
    /// **The first answer starts selected**, the same rule the order palette
    /// follows: the list starts at its top, and the player walks it from there.
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionDialog {
    /// A new dialog, with the first answer selected.
    #[must_use]
    pub const fn new() -> Self {
        Self { selected: 0 }
    }

    /// The answer the player is looking at.
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }

    /// Moves the selection up, and stops at the first answer.
    pub fn up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Moves the selection down, and stops at the last answer.
    ///
    /// **The selection is clamped to the list**, because a list the player can
    /// walk off the end of is a list that loses the focus.
    pub fn down(&mut self, option_count: usize) {
        self.selected = self
            .selected
            .saturating_add(1)
            .min(option_count.saturating_sub(1));
    }
}

/// Draws the decision dialog as a modal.
pub fn render(
    frame: &mut Frame,
    area: Rect,
    dialog: &DecisionDialog,
    decision: &Decision,
    due: &str,
) {
    let popup = centred(area, 60, 60);

    frame.render_widget(Clear, popup);

    let mut lines = vec![
        Line::raw(""),
        Line::styled(format!("  {}", decision.headline), theme::title()),
    ];

    if let Some(body) = &decision.body {
        lines.push(Line::raw(""));
        lines.push(Line::styled(format!("  {body}"), theme::historical()));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        format!("  Answer by {due}."),
        theme::Meaning::Informational.style(),
    ));

    // **Silence is an answer, and it is usually the expensive one.** The
    // dialog says so, because a player who puts the question aside deserves
    // to know what putting it aside does.
    let silence = effect_words(&decision.expiry_effects);
    if !silence.is_empty() {
        lines.push(Line::styled(
            format!("  If the question is ignored: {silence}"),
            theme::Meaning::Bad.style(),
        ));
    }

    lines.push(Line::raw(""));
    for (index, option) in decision.options.iter().enumerate() {
        let selected = index == dialog.selected();
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
            Span::styled(option.label.as_str(), style),
        ]));
        // **The price is on the screen before the choice is made.** An answer
        // whose cost is only known afterwards is not a decision; it is a bill.
        let hint = effect_words(&option.effects);
        if !hint.is_empty() {
            lines.push(Line::styled(format!("      {hint}"), theme::historical()));
        }
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  Enter to answer, Esc to put it aside until later",
        theme::Meaning::Informational.style(),
    ));

    let body = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Decision - the world is waiting "),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(body, popup);
}

/// Says what a list of effects does, in the player's language.
///
/// **The effects are deltas, and the words are their translation.** The rule
/// every message in this interface follows: it is a sentence to a player, it
/// names no identifier, and it says the direction as well as the size.
fn effect_words(effects: &[Effect]) -> String {
    let mut words: Vec<String> = Vec::new();
    for effect in effects {
        match effect {
            Effect::AdjustTreasury { delta } => {
                let credits = delta.credits();
                if credits < 0 {
                    words.push(format!(
                        "it costs {} credits",
                        commas(credits.unsigned_abs())
                    ));
                } else {
                    words.push(format!(
                        "it pays {} credits",
                        commas(u64::try_from(credits).unwrap_or(0))
                    ));
                }
            }
            Effect::AdjustAllMorale { delta } | Effect::AdjustMorale { delta, .. } => {
                words.push(moves_word("morale", *delta));
            }
            Effect::AdjustAllLoyalty { delta } | Effect::AdjustLoyalty { delta, .. } => {
                words.push(moves_word("loyalty", *delta));
            }
            Effect::AdjustAllFatigue { delta } => {
                words.push(moves_word("fatigue", *delta));
            }
            Effect::AdjustApproval { delta } => {
                words.push(moves_word("the state's approval", *delta));
            }
            Effect::AdjustPublicOpinion { delta } => {
                words.push(moves_word("public opinion", *delta));
            }
            Effect::AdjustPrestige { delta } => {
                words.push(moves_word("prestige", *delta));
            }
            Effect::AdjustAllHealth { delta } | Effect::AdjustHealth { delta, .. } => {
                words.push(moves_word("health", *delta));
            }
            Effect::AdjustAllCondition { delta }
            | Effect::AdjustFacilityCondition { delta, .. } => {
                words.push(moves_word("the equipment's condition", *delta));
            }
            _ => {}
        }
    }
    let joined = words.join(", ");
    if joined.is_empty() {
        String::new()
    } else {
        format!("{joined}.")
    }
}

/// A clause for a number that moves in one of two directions.
fn moves_word(what: &str, delta: i16) -> String {
    if delta < 0 {
        format!("{what} falls by {}", delta.unsigned_abs())
    } else {
        format!("{what} lifts by {delta}")
    }
}

/// A number with thousands separators, because 15000 is hard to read.
fn commas(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(character);
    }
    out
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

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_answer_starts_selected() {
        let dialog = DecisionDialog::new();
        assert_eq!(dialog.selected(), 0);
    }

    #[test]
    fn the_selection_walks_the_answers_and_stops_at_the_ends() {
        let mut dialog = DecisionDialog::new();
        dialog.down(2);
        assert_eq!(dialog.selected(), 1);
        dialog.down(2);
        assert_eq!(dialog.selected(), 1, "the last answer is the end");
        dialog.up();
        assert_eq!(dialog.selected(), 0);
        dialog.up();
        assert_eq!(dialog.selected(), 0, "the first answer is the other end");
    }

    #[test]
    fn the_effects_are_said_in_the_players_language() {
        use domain::money::Money;
        let effects = vec![
            Effect::AdjustTreasury {
                delta: Money::from_credits(-5_000),
            },
            Effect::AdjustAllMorale { delta: 2 },
        ];
        assert_eq!(
            effect_words(&effects),
            "it costs 5,000 credits, morale lifts by 2."
        );
    }

    #[test]
    fn a_bill_that_pays_back_is_said_as_a_payment() {
        use domain::money::Money;
        let effects = vec![Effect::AdjustTreasury {
            delta: Money::from_credits(15_000),
        }];
        assert_eq!(effect_words(&effects), "it pays 15,000 credits.");
    }

    #[test]
    fn falling_numbers_are_said_as_falling() {
        let effects = vec![Effect::AdjustPublicOpinion { delta: -3 }];
        assert_eq!(effect_words(&effects), "public opinion falls by 3.");
    }

    #[test]
    fn the_commas_go_in_the_right_places() {
        assert_eq!(commas(999), "999");
        assert_eq!(commas(1_000), "1,000");
        assert_eq!(commas(15_000), "15,000");
        assert_eq!(commas(1_240_000), "1,240,000");
    }
}
