//! The month report, shown as a modal over the current screen.
//!
//! The report leads with what happened and ends with what to do. The last line
//! is a suggestion derived from the state — not advice about the game, but a
//! factual statement of the trade the player is facing.

use domain::report::MonthReport;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::theme;

/// Draws the month report as a modal.
pub fn render(frame: &mut Frame, area: Rect, report: &MonthReport) {
    let popup = centered(area, 80, 80);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(popup);

    frame.render_widget(Clear, popup);

    let mut lines = vec![
        Line::raw(""),
        Line::styled(format!("  {}", report.headline), theme::emphasis()),
        Line::raw(""),
    ];

    for entry in &report.events {
        let meaning = match entry.severity {
            domain::chronicle::Severity::Info => theme::Meaning::Historical,
            domain::chronicle::Severity::Minor => theme::Meaning::Informational,
            domain::chronicle::Severity::Major => theme::Meaning::Attention,
            domain::chronicle::Severity::Critical => theme::Meaning::Bad,
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", meaning.glyph()), meaning.style()),
            Span::raw(entry.headline.clone()),
        ]));
    }

    for line in report.personnel.iter().chain(report.programmes.iter()) {
        lines.push(Line::raw(format!("    {}", line.text)));
    }

    for warning in &report.warnings {
        lines.push(Line::styled(
            format!("  ⚠ {warning}"),
            theme::Meaning::Attention.style(),
        ));
    }

    for rejection in &report.rejections {
        lines.push(Line::styled(
            format!("  ✗ {rejection}"),
            theme::Meaning::Bad.style(),
        ));
    }

    if !report.opportunities.is_empty() {
        lines.push(Line::raw(""));
        for opportunity in &report.opportunities {
            lines.push(Line::styled(
                format!("  ⓘ {opportunity}"),
                theme::Meaning::Informational.style(),
            ));
        }
    }

    let body = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", report.date)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(body, rows[0]);

    let footer = Paragraph::new(Line::raw("  Enter to continue"))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(footer, rows[1]);
}

/// A rectangle of the given percentage size, centred.
fn centered(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
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
