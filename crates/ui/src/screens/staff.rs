//! The staff screen: a filterable list and a detail panel.
//!
//! The detail panel carries the **binding factor** — the reason a team is slow,
//! named in words. It is the most important thing the interface shows.
//!
//! **The list is filterable and sortable**, because an agency with three
//! hundred people has a list that cannot be read top to bottom, and because
//! the question the player most needs answered is usually "who is about to
//! leave".

use domain::ids::PersonId;
use domain::performance::PerformanceResult;
use domain::person::Role;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use sim::derived::{self, TaskKind};

use crate::app::App;
use crate::theme;
use crate::widgets::binding_factor::BindingFactor;

/// The sort keys, in the order they are offered.
pub const SORTS: &[&str] = &["name", "age", "role", "fatigue", "risk", "shift"];

/// One row of the staff list.
#[derive(Clone, Debug)]
pub struct Row {
    /// Who.
    pub id: PersonId,
    /// Their name.
    pub name: String,
    /// What they do.
    pub role: Role,
    /// How old they are.
    pub age: u32,
    /// How content they are.
    pub morale: i16,
    /// How tired they are.
    pub fatigue: u32,
    /// How hard they are working, as a percentage.
    pub shift: u32,
    /// The chance they leave this month, in basis points.
    pub risk: u32,
}

/// The rows of the staff list, filtered and sorted.
///
/// **The filter matches the name and the role**, because those are what a
/// player knows about a person before they look at them.
#[must_use]
pub fn rows(app: &App) -> Vec<Row> {
    let state = app.sim().state();
    let control = app.list_control();
    let mut rows: Vec<Row> = state
        .people
        .values()
        .map(|person| Row {
            id: person.id,
            name: person.name.clone(),
            role: person.role,
            age: person.age,
            morale: person.state.morale_percent(),
            fatigue: person.state.fatigue_percent(),
            shift: person.state.shift_percent(),
            risk: sim::phases::personnel::attrition_risk(state, person.id),
        })
        .filter(|row| control.matches(&row.name) || control.matches(row.role.label()))
        .collect();

    sort(&mut rows, control.sort());
    rows
}

/// Sorts the rows by the chosen key.
fn sort(rows: &mut [Row], key: usize) {
    match SORTS.get(key).copied() {
        Some("name") => rows.sort_by_key(|row| row.name.to_lowercase()),
        Some("age") => rows.sort_by_key(|row| row.age),
        Some("role") => rows.sort_by_key(|row| row.role),
        Some("fatigue") => rows.sort_by_key(|row| std::cmp::Reverse(row.fatigue)),
        Some("risk") => rows.sort_by_key(|row| std::cmp::Reverse(row.risk)),
        Some("shift") => rows.sort_by_key(|row| std::cmp::Reverse(row.shift)),
        _ => {}
    }
}

/// Draws the staff screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let rows = rows(app);
    render_list(app, &rows, frame, columns[0]);

    // The detail panel follows the selection, which is a row of the *visible*
    // list - so a filter never leaves the detail panel pointing at somebody
    // the player cannot see.
    let selected = rows.get(app.selected());
    let Some(row) = selected else {
        let empty = Paragraph::new("  Select someone to see how they are doing.")
            .block(Block::default().borders(Borders::ALL).title("Detail"))
            .wrap(Wrap { trim: false });
        frame.render_widget(empty, columns[1]);
        return;
    };

    let state = app.sim().state();
    let Some(person) = state.people.get(&row.id) else {
        return;
    };

    let rows_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Min(10)])
        .split(columns[1]);

    let team_size = team_size_for(state, person.id);
    let result: PerformanceResult =
        derived::person_performance(state, person, TaskKind::Research, team_size, team_size > 1);

    // **The shift, and what it does.** The player needs to see the
    // consequence of the throttle, not just the setting, and they need to see
    // that a change is in flight.
    let shift_line = match app.pending_shift(person.id) {
        Some(pending) => format!(
            "{}% → {}% (pending)",
            person.state.shift_percent(),
            percent_of(pending)
        ),
        None => format!("{}%", person.state.shift_percent()),
    };

    let details = Paragraph::new(vec![
        Line::styled(
            format!(" {} — {}", person.name, person.role.label()),
            theme::emphasis(),
        ),
        Line::raw(format!("  {}", person.assignment.description())),
        Line::raw(format!(
            "  morale {}  fatigue {}  health {}  fame {}",
            person.state.morale_percent(),
            person.state.fatigue_percent(),
            person.state.health_percent(),
            person.state.fame,
        )),
        Line::raw(format!(
            "  technical {}  creative {}  leadership {}",
            person.attributes.technical, person.attributes.creative, person.attributes.leadership,
        )),
        Line::raw(format!(
            "  physical {}  age {}",
            person.attributes.physical, person.age
        )),
        Line::raw(format!(
            "  specialisms: {}",
            person
                .specialisms
                .iter()
                .map(|s| s.label())
                .collect::<Vec<_>>()
                .join(", ")
        )),
        Line::raw(format!("  salary {}", person.contract.salary)),
        Line::from(vec![
            Span::styled("  shift ", theme::emphasis()),
            Span::raw(format!("{shift_line}   ← → to change")),
        ]),
        Line::raw(format!(
            "  output {}%   fatigue {}%/month",
            percent_of_scaled(sim::shift::output_factor(person.state.shift)),
            percent_of_scaled(sim::shift::fatigue_rate(person.state.shift)),
        )),
        Line::raw(format!("  leaves this month: {}", percent_of_bp(row.risk))),
    ])
    .block(Block::default().borders(Borders::ALL).title("Detail"))
    .wrap(Wrap { trim: false });
    frame.render_widget(details, rows_area[0]);

    // The performance explanation.
    BindingFactor::new(&result).render(frame, rows_area[1]);
}

/// The list of people.
fn render_list(app: &App, rows: &[Row], frame: &mut Frame, area: Rect) {
    let mut lines = vec![Line::styled(
        format!(
            "  {:<20}{:<12}{:>3}{:>5}{:>6}",
            "name", "role", "age", "fat", "risk"
        ),
        theme::historical(),
    )];
    for (index, row) in rows.iter().enumerate() {
        let focused = index == app.selected();
        // **The focus is a style and a marker.** The reverse video is the
        // signal; the marker is the second signal, so the focus is findable
        // in a monochrome terminal as well as a colourful one.
        let (marker, style) = if focused {
            (theme::FOCUS, theme::selected())
        } else {
            (theme::NO_FOCUS, ratatui::style::Style::default())
        };
        // **The risk is in the warning colour when it matters**, so a person
        // about to leave is findable without reading the number.
        let risk_style = if row.risk >= 500 {
            theme::Meaning::Attention.style()
        } else {
            style
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{marker} "), style),
            Span::styled(format!("{:<20}", row.name), style),
            Span::styled(format!("{:<12}", row.role.label()), style),
            Span::styled(format!("{:>3}", row.age), style),
            Span::styled(format!("{:>5}", row.fatigue), style),
            Span::styled(format!("{:>6}", percent_of_bp(row.risk)), risk_style),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  Nobody matches that."));
    }

    // **The filter and the sort are always on screen.** A player who cannot
    // see that a filter is in force cannot tell why the list is short.
    let control = app.list_control();
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(" filter ", theme::emphasis()),
        Span::raw(format!("{:<12}", control.filter())),
        Span::styled(" / ", theme::emphasis()),
        Span::styled("  sort ", theme::emphasis()),
        Span::raw(format!("{:<8}", control.sort_name(SORTS))),
        Span::styled(" s ", theme::historical()),
    ]));

    let list = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Staff"))
        .wrap(Wrap { trim: false });
    frame.render_widget(list, area);
}

/// How many people the given person works with.
fn team_size_for(state: &domain::state::CampaignState, person: PersonId) -> usize {
    match &state.people.get(&person).map(|p| p.assignment.clone()) {
        Some(domain::person::Assignment::Project { project }) => state
            .projects
            .get(project)
            .map_or(1, |project| project.team.len().max(1)),
        _ => 1,
    }
}

/// A scaled percentage, as a whole number.
fn percent_of_scaled(value: i64) -> i64 {
    value * 100 / i64::from(domain::scale::FULL_PERCENT)
}

/// A scaled percentage, as a whole number.
fn percent_of(shift: u16) -> u32 {
    u32::from(shift) * 100 / domain::scale::FULL_PERCENT as u32
}

/// A number of basis points, as a percentage with one decimal.
///
/// **The risk is small, and a small number shown as a whole number is a
/// number that reads as zero.** Half a percent a month is the ordinary rate,
/// and it is the rate the player is judging everybody else against.
fn percent_of_bp(basis_points: u32) -> String {
    format!("{}.{}%", basis_points / 100, basis_points % 100)
}
