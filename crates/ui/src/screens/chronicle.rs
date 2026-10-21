//! The chronicle: history and forecast, filterable.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme;

/// Draws the chronicle screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.sim().state();
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // History.
    let mut lines = Vec::new();
    for entry in state.chronicle.iter().rev().take(40) {
        let meaning = match entry.severity {
            domain::chronicle::Severity::Info => theme::Meaning::Historical,
            domain::chronicle::Severity::Minor => theme::Meaning::Informational,
            domain::chronicle::Severity::Major => theme::Meaning::Attention,
            domain::chronicle::Severity::Critical => theme::Meaning::Bad,
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", state.calendar().date(entry.date)),
                theme::historical(),
            ),
            Span::styled(format!("{} ", meaning.glyph()), meaning.style()),
            Span::raw(entry.headline.clone()),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  Nothing has happened yet."));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        format!("  {} entries", state.chronicle.len()),
        theme::historical(),
    ));

    let history = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("History"))
        .wrap(Wrap { trim: false });
    frame.render_widget(history, columns[0]);

    // Forecast: what the player knows is coming. Never an outcome.
    let mut lines = Vec::new();
    for promise in &state.political.promises {
        if promise.is_resolved() {
            continue;
        }
        let Some(due) = promise.commitment.due() else {
            continue;
        };
        lines.push(Line::from(vec![
            Span::styled("○ ", theme::Meaning::Informational.style()),
            Span::raw(format!(
                "{} - promised for {}",
                promise.description,
                app.date(due)
            )),
        ]));
    }
    for programme in state.programmes.values() {
        for objective in &programme.objectives {
            let Some(due) = objective.commitment.due() else {
                continue;
            };
            lines.push(Line::from(vec![
                Span::styled("○ ", theme::Meaning::Informational.style()),
                Span::raw(format!(
                    "{} - committed for {}",
                    objective.description,
                    app.date(due)
                )),
            ]));
        }
    }
    for mission in state.missions.values() {
        lines.push(Line::from(vec![
            Span::styled("○ ", theme::Meaning::Informational.style()),
            Span::raw(format!(
                "Mission to {} - window {}",
                mission.destination,
                app.date(mission.launch_window)
            )),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  Nothing is scheduled."));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  The forecast shows what is scheduled. It does not show what will go wrong.",
        theme::historical(),
    ));

    let forecast = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Forecast"))
        .wrap(Wrap { trim: false });
    frame.render_widget(forecast, columns[1]);
}
