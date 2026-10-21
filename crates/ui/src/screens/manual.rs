//! The manual browser.
//!
//! **A list on the left and the explanation on the right.** The player narrows
//! the list by typing, moves through it with the arrow keys, and reads.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::manual::{Manual, Row, Topic};
use crate::theme;

/// Draws the manual.
pub fn render(manual: &mut Manual, frame: &mut Frame, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(34), Constraint::Percentage(66)])
        .split(area);

    render_list(manual, frame, columns[0]);
    render_detail(manual, frame, columns[1]);
}

/// The list of topics, grouped by section.
fn render_list(manual: &mut Manual, frame: &mut Frame, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize;
    manual.ensure_visible(inner_height);

    let mut lines = Vec::new();
    for row in manual.rows().iter().skip(manual.offset()) {
        match row {
            Row::Heading(section) => {
                lines.push(Line::styled(
                    format!("  {}", section.title().to_uppercase()),
                    theme::title(),
                ));
            }
            Row::Topic(index) => {
                let Some(topic) = manual.visible().get(*index) else {
                    continue;
                };
                let style = if *index == manual.index() {
                    theme::selected()
                } else {
                    ratatui::style::Style::default()
                };
                lines.push(Line::styled(format!("  {}", topic.title), style));
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::raw("  Nothing matches that."));
    }

    let list = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Manual"))
        .wrap(Wrap { trim: false });
    frame.render_widget(list, area);
}

/// The selected topic, in full.
fn render_detail(manual: &mut Manual, frame: &mut Frame, area: Rect) {
    let Some(topic) = manual.selected() else {
        let empty = Paragraph::new("  Nothing selected.")
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: false });
        frame.render_widget(empty, area);
        return;
    };

    let mut lines = vec![
        Line::styled(format!("  {}", topic.title), theme::title()),
        Line::raw(""),
        Line::styled(format!("  {}", topic.summary), theme::emphasis()),
        Line::raw(""),
    ];

    // **The body is written in paragraphs**, separated by a blank line, and
    // the blank lines are the author's - not the layout's.
    for paragraph in topic.body.split("\n\n") {
        for line in paragraph.lines() {
            lines.push(prose(line));
        }
        lines.push(Line::raw(""));
    }

    // **The explanation scrolls.** The longest one is over a thousand
    // characters, and a detail pane in a small terminal cannot show it all -
    // and an explanation the player cannot finish reading is not an
    // explanation.
    let inner_width = area.width.saturating_sub(4).max(1);
    let total = wrapped_height(&lines, inner_width);
    let pane = usize::from(area.height.saturating_sub(2));
    let max_scroll = total.saturating_sub(pane);
    if manual.scroll() > max_scroll {
        manual.set_scroll(max_scroll);
    }

    let title = if max_scroll > 0 {
        format!("Details  {} of {}", manual.scroll() + 1, total)
    } else {
        "Details".to_owned()
    };

    let detail = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false })
        .scroll((u16::try_from(manual.scroll()).unwrap_or(0), 0));
    frame.render_widget(detail, area);
}

/// How many rows a set of lines takes once it is wrapped.
fn wrapped_height(lines: &[Line<'_>], width: u16) -> usize {
    lines
        .iter()
        .map(|line| {
            let line_width = u16::try_from(line.width()).unwrap_or(u16::MAX);
            if line_width == 0 {
                1
            } else {
                usize::from(line_width.div_ceil(width))
            }
        })
        .sum()
}

/// Renders a line of the manual's prose.
///
/// **The prose carries two marks**: `**bold**` and `` `code` ``. They are
/// turned into styles rather than shown as punctuation, because a wall of
/// asterisks is not an explanation.
fn prose(text: &str) -> Line<'static> {
    let mut spans = vec![Span::raw("  ")];
    let mut current = String::new();
    let mut bold = false;
    let mut code = false;

    for character in text.chars() {
        match character {
            '*' => {
                flush(&mut spans, &mut current, bold, code);
                bold = !bold;
            }
            '`' => {
                flush(&mut spans, &mut current, bold, code);
                code = !code;
            }
            _ => current.push(character),
        }
    }
    flush(&mut spans, &mut current, bold, code);

    Line::from(spans)
}

/// Pushes the run of text out as a span, in the style it was marked with.
fn flush(spans: &mut Vec<Span<'static>>, current: &mut String, bold: bool, code: bool) {
    if current.is_empty() {
        return;
    }
    let text = std::mem::take(current);
    let style = if code {
        theme::emphasis()
    } else if bold {
        ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD)
    } else {
        ratatui::style::Style::default()
    };
    spans.push(Span::styled(text, style));
}

/// Draws the manual over everything else.
///
/// **The manual is the last thing drawn**, because it is a modal over
/// everything - including the report, the palette and the keymap.
pub fn render_over(manual: &mut Manual, frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
    render(manual, frame, area);
}

/// The line under the manual, saying how to use it.
#[must_use]
pub fn footer() -> Vec<Line<'static>> {
    vec![Line::from(vec![
        Span::styled(" type ", theme::emphasis()),
        Span::raw("to filter"),
        Span::styled("  ↑ ↓ ", theme::emphasis()),
        Span::raw("to move"),
        Span::styled("  pgdn ", theme::emphasis()),
        Span::raw("to scroll"),
        Span::styled("  F1 ", theme::emphasis()),
        Span::raw("keys"),
        Span::styled("  esc ", theme::emphasis()),
        Span::raw("close"),
    ])]
}

/// The topic's title, for a line somewhere narrow.
#[must_use]
pub fn label(topic: &Topic) -> String {
    topic.title.to_owned()
}
