//! The opening briefing.
//!
//! **The game opens with the framing and a clear first goal.** The player is
//! the Director of an agency with a deadline, and the first thing they see
//! says so — in the game's own words, and not as a wall of text.
//!
//! It is a modal over the dashboard, and any key dismisses it. **The campaign
//! does not advance while it is open**, because the world waits while the
//! player is reading.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme;

/// The lines of the briefing, for the campaign and its goal.
#[must_use]
pub fn lines(scenario_name: &str, goal: &str) -> Vec<Line<'static>> {
    vec![
        Line::styled(
            "Welcome, Director.".to_owned(),
            theme::title(),
        ),
        Line::raw(""),
        Line::styled(
            "You have been appointed Director of the Stellar Expansion Authority, the most ambitious \
             organisation in the history of your civilisation.".to_owned(),
            theme::historical(),
        ),
        Line::raw(""),
        Line::styled(
            "For centuries our people studied the heavens from afar. Today, curiosity ends and \
             exploration begins.".to_owned(),
            theme::historical(),
        ),
        Line::raw(""),
        Line::styled(
            "Your agency has been tasked with leading our species into space.".to_owned(),
            theme::emphasis(),
        ),
        Line::raw(""),
        Line::styled(
            format!("Campaign: {scenario_name}"),
            theme::title(),
        ),
        Line::styled(
            format!("Your first goal: {goal}"),
            theme::Meaning::Informational.style(),
        ),
        Line::raw(""),
        Line::styled(
            "Every mission carries risk. A failed launch can destroy years of work; a successful \
             one can unlock new resources for the future.".to_owned(),
            theme::historical(),
        ),
        Line::raw(""),
        Line::styled(
            "There is no single path to success. Press any key to begin.",
            theme::Meaning::Attention.style(),
        ),
    ]
}

/// Draws the briefing over the screen behind it.
pub fn render(frame: &mut Frame, area: Rect, scenario_name: &str, goal: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("The Stellar Expansion Authority");
    let paragraph = Paragraph::new(lines(scenario_name, goal))
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });
    // **The briefing is a modal, and it clears what is behind it.**
    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}
