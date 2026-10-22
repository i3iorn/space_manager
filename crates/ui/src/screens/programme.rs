//! The programme screen: objectives, both dates, slack, budget, progress, risk.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::list_control::{Field, FieldValue};
use crate::theme;
use crate::widgets::slack_meter;

/// The fields the objective list can be filtered by.
///
/// **"Which of my objectives is in trouble?" is not a question about names**,
/// and a list of objectives that can only be searched by name is a list the
/// player has to read.
pub const FIELDS: &[Field] = &[
    Field::number("progress"),
    Field::number("slack"),
    Field::text("programme"),
    Field::flag("met"),
    Field::flag("unmet"),
];

/// The field names, as one line of help.
#[must_use]
pub fn field_names() -> String {
    FIELDS
        .iter()
        .map(|field| field.name)
        .collect::<Vec<_>>()
        .join(" ")
}

/// One objective in the list.
#[derive(Clone, Debug)]
pub struct Row {
    /// The programme it belongs to.
    pub programme: domain::ids::ProgrammeId,
    /// The programme's name.
    pub programme_title: String,
    /// The objective itself.
    pub objective: domain::ids::ObjectiveId,
    /// What it is.
    pub description: String,
    /// How far along it is, as a percentage.
    pub progress: u32,
    /// How much slack is left, in months.
    pub slack: Option<u32>,
    /// Whether it has been met, if it is due.
    pub met: Option<bool>,
}

impl Row {
    /// The value of a field, for the filter.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<FieldValue<'_>> {
        match name {
            "progress" => Some(FieldValue::Number(i64::from(self.progress))),
            "slack" => self.slack.map(|slack| FieldValue::Number(i64::from(slack))),
            "programme" => Some(FieldValue::Text(&self.programme_title)),
            "met" => Some(FieldValue::Flag(self.met == Some(true))),
            "unmet" => Some(FieldValue::Flag(self.met == Some(false))),
            _ => None,
        }
    }
}

/// The objectives that pass the filter, in the order they are shown.
#[must_use]
pub fn rows(app: &App) -> Vec<Row> {
    let state = app.sim().state();
    let control = app.list_control();
    let terms = control.terms(FIELDS);
    let mut rows = Vec::new();
    for programme in state.programmes.values() {
        for objective in &programme.objectives {
            let row = Row {
                programme: programme.id,
                programme_title: programme.title.clone(),
                objective: objective.id,
                description: objective.description.clone(),
                progress: objective.progress.percent(),
                slack: objective
                    .commitment
                    .slack()
                    .map(|slack| slack.whole_months().max(1) as u32),
                met: objective.met,
            };
            // **Every term has to hold.**
            let holds = terms.iter().all(|term| {
                let value = term.field().and_then(|name| row.field(name));
                term.holds(
                    &[row.description.as_str(), row.programme_title.as_str()],
                    value.as_ref(),
                )
            });
            if holds {
                rows.push(row);
            }
        }
    }
    rows
}

/// Draws the programme screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.sim().state();
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(area);

    let rows = rows(app);
    let mut lines = Vec::new();

    if state.programmes.is_empty() {
        lines.push(Line::raw("  The agency has no programmes yet."));
    }

    // **The selected objective**, which the slack meter is about.
    //
    // **The selection is the player's position in the *filtered* list**, so a
    // filter never leaves the meter pointing at an objective the player
    // cannot see.
    let selected_row = rows.get(app.selected()).cloned();

    // **The programmes, and the objectives that pass the filter under each.**
    let mut objective_index = 0usize;
    for programme in state.programmes.values() {
        let mine: Vec<&Row> = rows
            .iter()
            .filter(|row| row.programme == programme.id)
            .collect();
        if mine.is_empty() {
            continue;
        }
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
        for row in mine {
            let Some(objective) = programme
                .objectives
                .iter()
                .find(|objective| objective.id == row.objective)
            else {
                continue;
            };
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

    // **The research projects**, because the programme screen is where the
    // player looks for the work the programme is standing on.
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
    }

    // **The filter line**, so the player can see what is in force and what
    // they may type.
    let control = app.list_control();
    lines.push(Line::from(vec![
        Span::styled(" filter ", theme::emphasis()),
        Span::raw(format!("{:<24}", control.filter())),
        Span::styled(" fields ", theme::historical()),
        Span::styled(field_names(), theme::historical()),
    ]));

    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Programmes"))
        .wrap(Wrap { trim: false });
    frame.render_widget(body, columns[0]);

    // **The slack meter**, for the selected objective.
    //
    // **The slack is the resource the player manages**, and it is spent. The
    // meter is the one panel that says how much of it is left, and it is
    // beside the list because the two are read together.
    let selected = selected_row.and_then(|row| {
        state
            .programmes
            .get(&row.programme)
            .and_then(|programme| programme.objectives.iter().find(|o| o.id == row.objective))
    });
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
    rows(app).len()
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
