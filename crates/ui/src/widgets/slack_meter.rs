//! The slack meter.
//!
//! **The slack is the resource the player manages, and it is spent.** The meter
//! shows what is left of the gap between the plan and the promise, and it is
//! the one number on the programme screen that gets worse on its own.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use domain::time::Duration;

use crate::theme;

/// The slack meter, and what it says about a commitment.
pub struct SlackMeter<'a> {
    /// What the commitment is.
    pub description: &'a str,
    /// The gap between the plan and the promise.
    pub slack: Option<Duration>,
    /// Whether the commitment has been met, and how.
    pub met: Option<bool>,
}

impl<'a> SlackMeter<'a> {
    /// A meter for a commitment.
    #[must_use]
    pub const fn new(description: &'a str, slack: Option<Duration>, met: Option<bool>) -> Self {
        Self {
            description,
            slack,
            met,
        }
    }

    /// Draws the meter.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = vec![
            Line::styled(format!("  {}", self.description), theme::emphasis()),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(self.glyph(), style_for(self.slack, self.met)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(self.gauge(), style_for(self.slack, self.met)),
            ]),
        ];

        let body = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Slack"))
            .wrap(Wrap { trim: false });
        frame.render_widget(body, area);
    }

    /// The word for where the commitment stands.
    fn glyph(&self) -> String {
        match self.met {
            Some(true) => "✓ met — the promise was kept".to_owned(),
            Some(false) => "✗ missed — the promise was broken".to_owned(),
            None => match self.slack {
                Some(slack) => format!("○ {slack} of slack left"),
                None => "— no slack: the plan and the promise are the same day".to_owned(),
            },
        }
    }

    /// The bar.
    ///
    /// **A bar of blocks, and the colour is the second signal.** A commitment
    /// with no slack is a bar that is empty, and it is the emptiest bar on the
    /// screen, and the player is meant to feel that.
    fn gauge(&self) -> String {
        const WIDTH: usize = 20;
        let filled = match self.slack {
            Some(slack) => {
                // **A year of slack is a full bar.** More than a year is a
                // comfortable buffer and a year is a gamble.
                let months = slack.whole_months();
                let ratio = months.min(12) as f64 / 12.0;
                (ratio * WIDTH as f64).round() as usize
            }
            None => 0,
        };
        let mut bar = String::new();
        for _ in 0..filled {
            bar.push('█');
        }
        for _ in filled..WIDTH {
            bar.push('░');
        }
        bar
    }
}

/// The style for a commitment, by how much slack is left.
///
/// **A commitment with three months of slack is a warning**, and one with none
/// is a bad.
#[must_use]
pub fn style_for(slack: Option<Duration>, met: Option<bool>) -> Style {
    match met {
        Some(true) => theme::Meaning::Good.style(),
        Some(false) => theme::Meaning::Bad.style(),
        None => match slack {
            Some(slack) if slack.whole_months() < 3 => theme::Meaning::Attention.style(),
            Some(_) => Style::default().fg(Color::Cyan),
            None => theme::Meaning::Bad.style(),
        },
    }
}

/// The word for where the commitment stands, for the caller.
#[must_use]
pub fn summary(slack: Option<Duration>, met: Option<bool>) -> String {
    match met {
        Some(true) => "met".to_owned(),
        Some(false) => "missed".to_owned(),
        None => match slack {
            Some(slack) => format!("{slack} of slack"),
            None => "no slack".to_owned(),
        },
    }
}
