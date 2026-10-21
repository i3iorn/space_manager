//! The dashboard — the screen that answers "where do I stand right now".
//!
//! Shows the headline, the upcoming obligations, and the financial position.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme;

/// Draws the dashboard.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.sim().state();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(3),
        ])
        .split(area);

    // The headline. The player's eye goes here first.
    let headline = Paragraph::new(Line::styled(
        format!(" {}", state.agency.focus),
        theme::emphasis(),
    ))
    .block(Block::default().borders(Borders::ALL).title("Headline"));
    frame.render_widget(headline, rows[0]);

    // What is due.
    let mut lines = Vec::new();
    for project in state.projects.values() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("○ ", theme::Meaning::Informational.style()),
            Span::raw(format!(
                "{} — research, {}/{} team",
                project.title,
                project.team.len(),
                project.team.len().max(1)
            )),
        ]));
    }
    for programme in state.programmes.values() {
        for objective in &programme.objectives {
            let Some(due) = objective.commitment.due() else {
                continue;
            };
            let slack = objective
                .commitment
                .slack()
                .map_or_else(|| "no slack".to_owned(), |slack| format!("slack {slack}"));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("○ ", theme::Meaning::Informational.style()),
                Span::raw(format!(
                    "{} — committed {} ({slack})",
                    objective.description,
                    app.date(due)
                )),
            ]));
        }
    }
    for promise in &state.political.promises {
        if promise.is_resolved() {
            continue;
        }
        let Some(due) = promise.commitment.due() else {
            continue;
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("○ ", theme::Meaning::Informational.style()),
            Span::raw(format!(
                "{} - promised for {}",
                promise.description,
                app.date(due)
            )),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  Nothing is scheduled."));
    }

    let due = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Due soon"))
        .wrap(Wrap { trim: false });
    frame.render_widget(due, rows[1]);

    // The financial position.
    let financial = finance_panel(state);
    frame.render_widget(financial, rows[2]);
}

/// The financial panel: the money, the public's opinion, and the agency's standing.
///
/// **The public's opinion is shown here because the world reads it.** The
/// weighting that decides which events fire looks at the public's opinion, so
/// a number the player cannot see is a number the player cannot act on. The
/// state's approval is a different thing and is in the top bar.
fn finance_panel(state: &domain::state::CampaignState) -> Paragraph<'static> {
    let treasury = &state.treasury;
    let net = treasury.net();
    let runway = treasury.runway_months();
    let opinion = state.public.opinion.percent();
    let opinion_style = if opinion < -30 {
        theme::Meaning::Bad.style()
    } else if opinion < 0 {
        theme::Meaning::Attention.style()
    } else {
        theme::historical()
    };
    Paragraph::new(vec![
        Line::styled(
            format!(
                "  income {}   outgoings {}   net {}   {}   {}",
                treasury.total_income(),
                treasury.total_outgoings(),
                net,
                if runway < u32::MAX {
                    format!("runway {runway} months")
                } else {
                    "in surplus".to_owned()
                },
                treasury.crisis_stage.label(),
            ),
            if net.is_negative() {
                theme::Meaning::Bad.style()
            } else {
                Style::default()
            },
        ),
        Line::raw(""),
        Line::from(vec![
            Span::raw("  public opinion "),
            Span::styled(format!("{opinion:+}"), opinion_style),
            Span::raw("   prestige "),
            Span::styled(
                format!("{:+}", state.public.prestige.percent()),
                theme::historical(),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Finance"))
}
