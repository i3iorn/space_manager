//! The programme screen: objectives, both dates, slack, budget, progress, risk.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme;
use crate::widgets::slack_meter;

/// Draws the programme screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.sim().state();
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(area);

    let mut lines = Vec::new();

    if state.programmes.is_empty() {
        lines.push(Line::raw("  The agency has no programmes yet."));
    }

    // **The selected objective**, which the slack meter is about.
    let mut objective_index = 0usize;

    for programme in state.programmes.values() {
        lines.push(Line::styled(
            format!(
                "  {} — {} ({})",
                programme.title,
                programme.category.label(),
                programme.status.label()
            ),
            theme::emphasis(),
        ));
        lines.push(Line::raw(format!(
            "    budget {} allocated, {} spent, {} committed",
            programme.budget.allocation, programme.budget.spent, programme.budget.committed
        )));
        lines.push(Line::raw(format!(
            "    staff {}   visibility {}",
            programme.staffing.len(),
            programme.visibility
        )));

        for objective in &programme.objectives {
            objective_lines(
                app,
                state,
                programme,
                objective,
                &mut lines,
                &mut objective_index,
            );
        }
        lines.push(Line::raw(""));
    }

    for project in state.projects.values() {
        lines.push(Line::styled(
            format!(
                "  {} — research, {}%",
                project.title,
                project.progress.percent()
            ),
            theme::emphasis(),
        ));
        lines.push(Line::raw(format!(
            "    {}   team {}   PI {}",
            domain::research::discipline_name(project.discipline),
            project.team.len(),
            app.person_name(project.principal_investigator)
        )));
        lines.push(Line::raw(""));
    }

    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Programmes"))
        .wrap(Wrap { trim: false });
    frame.render_widget(body, columns[0]);

    // **The slack meter**, for the selected objective.
    //
    // **The slack is the resource the player manages**, and it is spent. The
    // meter is the one panel that says how much of it is left, and it is
    // beside the list because the two are read together.
    let selected = state
        .programmes
        .values()
        .flat_map(|programme| programme.objectives.iter())
        .nth(app.selected());
    if let Some(objective) = selected {
        let description = objective.description.clone();
        slack_meter::SlackMeter::new(&description, objective.commitment.slack(), objective.met)
            .render(frame, columns[1]);
    } else {
        let empty = Paragraph::new("  No objective is selected.")
            .block(Block::default().borders(Borders::ALL).title("Slack"))
            .wrap(Wrap { trim: false });
        frame.render_widget(empty, columns[1]);
    }
}

/// How many objectives there are to move through.
///
/// **The objectives of every programme, in the order they are shown**, because
/// the selection counts across them and not per programme.
#[must_use]
pub fn objective_count(app: &App) -> usize {
    app.sim()
        .state()
        .programmes
        .values()
        .flat_map(|programme| programme.objectives.iter())
        .count()
}

/// The lines for one objective.
///
/// **The two dates are never on the same row.** The typography carries the
/// distinction: the commitment is a promise, the target is a plan.
fn objective_lines(
    app: &App,
    state: &domain::state::CampaignState,
    programme: &domain::programme::Programme,
    objective: &domain::programme::Objective,
    lines: &mut Vec<Line<'static>>,
    objective_index: &mut usize,
) {
    let selected = *objective_index == app.selected();
    *objective_index += 1;
    lines.push(Line::raw(format!("    -  {}", objective.description)));
    // **How the objective is done.**
    //
    // **"Reach orbit" is not a clue.** A player who is told that the agency
    // must reach orbit, and not told that a launch is what reaches orbit, has
    // to guess — and the guessing is the whole of the first hour. The domain
    // knows what finishes the objective, so the screen says it.
    lines.push(Line::styled(
        format!("         done by   {}", objective.completion.phrase()),
        theme::historical(),
    ));
    if let Some(target) = objective.commitment.target {
        lines.push(Line::styled(
            format!("         target    {}", app.date(target)),
            theme::target(),
        ));
    }
    if let Some(committed) = objective.commitment.committed {
        lines.push(Line::styled(
            format!("         COMMITTED {}", app.date(committed)),
            theme::committed(),
        ));
    }
    // **The slack is a duration, not a number of months** - it is the time
    // between the plan and the promise, and it is written in the units the
    // player thinks in.
    let slack = objective.commitment.slack();
    let slack_text = slack_meter::summary(slack, objective.met);
    let slack_style = slack_meter::style_for(slack, objective.met);
    lines.push(Line::from(vec![
        Span::raw(format!(
            "      {}   progress {}%   ",
            if selected { ">" } else { " " },
            objective.progress.percent()
        )),
        Span::styled(slack_text, slack_style),
        Span::raw(format!(
            "   promised to {}",
            objective.commitment.audience.label()
        )),
    ]));

    // **The projection is a plan, not a promise**: when the work would finish
    // at the rate the programme is running at now.
    if let Some(finish) = sim::derived::objective_projection(
        state,
        app.sim().effective_now(),
        programme.id,
        objective,
    ) {
        let calendar = app.sim().calendar().date(finish);
        lines.push(Line::raw(format!(
            "         at this rate it finishes {calendar}"
        )));
    } else if objective.progress.percent() < 100 {
        lines.push(Line::styled(
            "         at this rate it never finishes".to_owned(),
            theme::Meaning::Bad.style(),
        ));
    }
}
/// A line showing a commitment, used by the report.
#[must_use]
pub fn commitment_line(description: &str, due: &str, met: Option<bool>) -> Line<'static> {
    let (glyph, meaning) = match met {
        Some(true) => ("✓", theme::Meaning::Good),
        Some(false) => ("✗", theme::Meaning::Bad),
        None => ("○", theme::Meaning::Informational),
    };
    Line::from(vec![
        Span::styled(format!("{glyph} "), meaning.style()),
        Span::raw(format!("{description} — {due}")),
    ])
}
