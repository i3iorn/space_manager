//! The research and facilities screen: two panes side by side.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::list_control::{Field, FieldValue};
use crate::theme;

/// The sort keys, in the order they are offered.
pub const SORTS: &[&str] = &["name", "progress", "team"];

/// The fields the research list can be filtered by.
///
/// **"Which project is nearly finished?" is not a question about names.**
pub const FIELDS: &[Field] = &[
    Field::number("progress"),
    Field::number("team"),
    Field::text("discipline"),
    Field::flag("test"),
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

/// One thing on the research list: a project, or a test campaign.
#[derive(Clone, Debug)]
pub enum Item {
    /// A research project.
    Project {
        /// Its title.
        title: String,
        /// The field it is in.
        discipline: String,
        /// How far along it is, as a percentage.
        progress: u32,
        /// How many people are on it.
        team: usize,
        /// What it costs a month.
        cost: String,
        /// What has been spent on it.
        spent: domain::money::Money,
    },
    /// A test campaign.
    Test {
        /// The capability it is testing.
        capability: String,
        /// How far along it is, as a percentage.
        progress: u32,
        /// How many people are on it.
        team: usize,
        /// How much risk it has taken out.
        risk_reduction: i16,
    },
}

impl Item {
    /// The text the filter searches.
    #[must_use]
    pub fn text(&self) -> &str {
        match self {
            Self::Project { title, .. } => title,
            Self::Test { capability, .. } => capability,
        }
    }

    /// The value of a field, for the filter.
    #[must_use]
    pub fn field(&self, name: &str) -> Option<FieldValue<'_>> {
        match (name, self) {
            ("progress", Self::Project { progress, .. } | Self::Test { progress, .. }) => {
                Some(FieldValue::Number(i64::from(*progress)))
            }
            ("team", Self::Project { team, .. } | Self::Test { team, .. }) => {
                Some(FieldValue::Number(i64::try_from(*team).unwrap_or(i64::MAX)))
            }
            ("discipline", Self::Project { discipline, .. }) => Some(FieldValue::Text(discipline)),
            ("test", Self::Test { .. }) => Some(FieldValue::Flag(true)),
            ("test", Self::Project { .. }) => Some(FieldValue::Flag(false)),
            _ => None,
        }
    }
}

/// The research list, filtered and sorted.
#[must_use]
pub fn items(app: &App) -> Vec<Item> {
    let state = app.sim().state();
    let control = app.list_control();
    let terms = control.terms(FIELDS);
    let mut items: Vec<Item> = Vec::new();
    for project in state.projects.values() {
        items.push(Item::Project {
            title: project.title.clone(),
            discipline: domain::research::discipline_name(project.discipline).to_owned(),
            progress: project.progress.percent(),
            team: project.team.len(),
            cost: sim::balance::research_cost_per_month(project.team.len()).to_string(),
            spent: project.funding.spent,
        });
    }
    for campaign in state.test_campaigns.values() {
        items.push(Item::Test {
            capability: domain::research::capability_name(campaign.target_capability).to_owned(),
            progress: campaign.progress.percent(),
            team: campaign.team.len(),
            risk_reduction: campaign.risk_reduction,
        });
    }

    items.retain(|item| {
        terms.iter().all(|term| {
            let value = term.field().and_then(|name| item.field(name));
            term.holds(&[item.text()], value.as_ref())
        })
    });

    sort(&mut items, control.sort());
    items
}

/// Sorts the list by the chosen key.
fn sort(items: &mut [Item], key: usize) {
    match SORTS.get(key).copied() {
        Some("name") => items.sort_by_key(|item| item.text().to_lowercase()),
        Some("progress") => items.sort_by_key(|item| std::cmp::Reverse(progress_of(item))),
        Some("team") => items.sort_by_key(|item| std::cmp::Reverse(team_of(item))),
        _ => {}
    }
}

/// How far along an item is.
fn progress_of(item: &Item) -> u32 {
    match item {
        Item::Project { progress, .. } | Item::Test { progress, .. } => *progress,
    }
}

/// How many people are on an item.
fn team_of(item: &Item) -> usize {
    match item {
        Item::Project { team, .. } | Item::Test { team, .. } => *team,
    }
}

/// Draws the research and facilities screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_research(app, frame, columns[0]);
    render_facilities(app, frame, columns[1]);
}

/// The research pane: the projects and the test campaigns.
fn render_research(app: &App, frame: &mut Frame, area: Rect) {
    let mut lines = Vec::new();
    for item in items(app) {
        match item {
            Item::Project {
                title,
                discipline,
                progress,
                team,
                cost,
                spent,
            } => {
                lines.push(Line::styled(
                    format!("  {title} — {progress}%"),
                    theme::emphasis(),
                ));
                lines.push(Line::raw(format!("    {discipline}   team {team}")));
                lines.push(Line::raw(format!("    cost {cost}   spent {spent}")));
            }
            Item::Test {
                capability,
                progress,
                team,
                risk_reduction,
            } => {
                lines.push(Line::styled(
                    format!("  Test: {capability} — {progress}%"),
                    theme::emphasis(),
                ));
                lines.push(Line::raw(format!(
                    "    team {team}   risk reduction {risk_reduction} points"
                )));
            }
        }
    }
    if lines.is_empty() {
        lines.push(Line::raw("  No research is running."));
    }

    // **The filter and the sort**, so the player can see what is in force and
    // what they may type.
    let control = app.list_control();
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(" filter ", theme::emphasis()),
        Span::raw(format!("{:<24}", control.filter())),
        Span::styled(" sort ", theme::emphasis()),
        Span::raw(format!("{:<8}", control.sort_name(SORTS))),
        Span::styled(" s ", theme::historical()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" fields ", theme::historical()),
        Span::styled(field_names(), theme::historical()),
    ]));

    let research = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Research"))
        .wrap(Wrap { trim: false });
    frame.render_widget(research, area);
}

/// The facilities pane: the buildings, and how hard they are being pushed.
fn render_facilities(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.sim().state();
    let mut lines = Vec::new();
    for facility in state.facilities.values() {
        let status = if facility.is_operational() {
            Span::styled("✓ ", theme::Meaning::Good.style())
        } else {
            Span::styled("⚠ ", theme::Meaning::Attention.style())
        };
        lines.push(Line::from(vec![
            status,
            Span::styled(facility.name.clone(), theme::emphasis()),
        ]));
        lines.push(Line::raw(format!(
            "    {}   condition {}%   upkeep {}   staffed {}/{}",
            facility.kind.label(),
            facility.condition_percent(),
            facility.upkeep,
            facility.staff.len(),
            facility.specification.staff_needed
        )));
        if let Some(construction) = &facility.construction {
            lines.push(Line::raw(format!(
                "    building — {} method, {} scale, {}% done",
                construction.plan.method.label(),
                construction.plan.scale.label(),
                construction.progress.percent()
            )));
        } else {
            lines.push(Line::raw(format!(
                "    output {}   quality {}   reliability {}   places {}",
                facility.effective_output(),
                facility.specification.quality,
                facility.specification.reliability,
                facility.specification.capacity
            )));
        }
        lines.push(Line::raw(""));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  The agency has no facilities."));
    }

    // **The industrial capacity, and how hard it is being pushed.** It is the
    // reason a bigger laboratory is worth building, and the tools factor falls
    // when the agency is over it.
    let (capacity, demand) = sim::derived::industrial_capacity(state);
    let pressure = if demand > capacity {
        Span::styled("over capacity", theme::Meaning::Attention.style())
    } else {
        Span::styled("room to spare", theme::historical())
    };
    lines.push(Line::from(vec![
        Span::styled("  capacity ", theme::emphasis()),
        Span::raw(format!("{demand} working, {capacity} places   ")),
        pressure,
    ]));

    let facilities = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Facilities"))
        .wrap(Wrap { trim: false });
    frame.render_widget(facilities, area);
}
