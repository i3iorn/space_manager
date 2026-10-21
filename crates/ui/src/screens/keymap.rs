//! The keymap.
//!
//! **The player should not have to leave the game to find the keys.** A
//! pointer to a document is a pointer to a document the player is not reading,
//! so the keymap is a modal in the game.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme;

/// Draws the keymap as a modal.
///
/// **The modal is sized to its content, not to a percentage of the screen.** A
/// keymap that is clipped in a small terminal is a keymap the player cannot
/// read, and a terminal is exactly as tall as the player made it.
pub fn render(frame: &mut Frame, area: Rect) {
    let lines = content();
    let popup = sized(area, &lines);

    frame.render_widget(Clear, popup);
    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Keys"))
        .wrap(Wrap { trim: false });
    frame.render_widget(body, popup);
}

/// The keymap's lines.
fn content() -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for (key, what) in ENTRIES {
        lines.push(Line::from(vec![
            Span::styled(format!("  {key:<12}"), theme::emphasis()),
            Span::raw((*what).to_owned()),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  The clock runs the campaign. Space stops it.",
        theme::Meaning::Informational.style(),
    ));
    lines.push(Line::raw(""));
    lines.push(Line::styled("  [ any key to close ]", theme::historical()));

    lines
}

/// The keys, and what they do.
const ENTRIES: &[(&str, &str)] = &[
    ("1 .. 7", "jump to a screen"),
    ("Tab", "next screen"),
    ("↑ ↓", "move the selection"),
    ("← →", "the shift, on the staff screen"),
    ("← →", "move between the fields of a form"),
    ("Enter", "confirm, or fork on the branches screen"),
    ("Esc", "back, or close what is open"),
    ("o", "open the order palette"),
    ("Space", "pause and resume the clock"),
    ("+ / -", "one dilation step faster or slower"),
    ("Ctrl+1 .. 9", "set the dilation directly"),
    ("F1", "this keymap, on every layout"),
    ("? or shift+/", "this keymap, if your layout has them"),
    ("F2", "the manual - everything the game explains"),
    ("I", "the manual, for this screen only"),
    ("/", "filter the list"),
    ("s", "change how the list is sorted"),
    ("q", "quit"),
];

/// The keys in the ledger, and what they do.
///
/// **Only the keys a player reaches for in the first ten minutes earn a place
/// here.** The ledger is one line, and a line that lists everything lists
/// nothing.
pub const LEDGER: &[(&str, &str)] = &[
    ("o", "order"),
    ("space", "pause"),
    ("+ -", "speed"),
    ("1-7", "screens"),
    ("F1", "keys"),
    ("F2", "manual"),
    ("q", "quit"),
];

/// Draws the key ledger: one line, at the bottom, on every screen.
///
/// **The most common keys are always on screen.** A player should not have to
/// remember the keymap, and should not have to open it to find the one key
/// they were told about.
pub fn render_ledger(frame: &mut Frame, area: Rect) {
    let mut spans = Vec::with_capacity(LEDGER.len() * 2);
    for (index, (key, what)) in LEDGER.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(format!(" {key} "), theme::emphasis()));
        spans.push(Span::raw((*what).to_owned()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// The rectangle the keymap needs.
///
/// **As wide as its widest line, and as tall as its wrapped lines.** A
/// percentage of the screen is a guess; the content is not. The height accounts
/// for the wrapping, because a keymap whose last entry is cut off is a keymap
/// the player cannot read.
fn sized(area: Rect, lines: &[Line<'static>]) -> Rect {
    // The room inside the border.
    const PADDING: u16 = 4;

    let widest = lines
        .iter()
        .map(|line| u16::try_from(line.width()).unwrap_or(u16::MAX))
        .max()
        .unwrap_or(0);

    let width = widest.saturating_add(PADDING).min(area.width);
    let text_width = width.saturating_sub(PADDING).max(1);

    let height: u16 = lines
        .iter()
        .map(|line| {
            let line_width = u16::try_from(line.width()).unwrap_or(u16::MAX);
            if line_width == 0 {
                1
            } else {
                line_width.div_ceil(text_width)
            }
        })
        .sum();
    let height = height.saturating_add(2).min(area.height);

    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}
