//! The chronicle: history and forecast, filterable.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme;

/// The forecast: the next twelve months of scheduled obligations.
///
/// **The player is planning this year**, and a schedule that runs to the end of
/// the campaign is not a plan. Each entry carries its risk figure: the slack in
/// a commitment, and the chance of working for a mission.
fn forecast(app: &App, state: &domain::state::CampaignState, lines: &mut Vec<Line<'static>>) {
    let horizon = app
        .sim()
        .effective_now()
        .saturating_add(domain::time::Duration::from_months(12));
    let mut forecast_lines = Vec::new();
    for promise in &state.political.promises {
        if promise.is_resolved() {
            continue;
        }
        let Some(due) = promise.commitment.due() else {
            continue;
        };
        if due > horizon {
            continue;
        }
        forecast_lines.push(Line::from(vec![
            Span::styled("-< ", theme::Meaning::Informational.style()),
            Span::raw(format!(
                "{} - promised for {}",
                promise.description,
                app.date(due)
            )),
            Span::styled(
                format!("  {}", slack_figure(promise.commitment.slack())),
                theme::historical(),
            ),
        ]));
    }
    for programme in state.programmes.values() {
        for objective in &programme.objectives {
            let Some(due) = objective.commitment.due() else {
                continue;
            };
            if due > horizon {
                continue;
            }
            let slack = objective.commitment.slack();
            let risk = match slack {
                Some(slack) if domain::time::Duration::is_zero(slack) => "no slack left".to_owned(),
                Some(slack) => format!("{slack} of slack"),
                None => "no slack".to_owned(),
            };
            let style = if slack.is_some_and(domain::time::Duration::is_zero) {
                theme::Meaning::Bad.style()
            } else {
                theme::historical()
            };
            forecast_lines.push(Line::from(vec![
                Span::styled("-< ", theme::Meaning::Informational.style()),
                Span::raw(format!(
                    "{} - committed for {}",
                    objective.description,
                    app.date(due)
                )),
                Span::styled(format!("  {risk}"), style),
            ]));
        }
    }
    for mission in state.missions.values() {
        if mission.launch_window > horizon {
            continue;
        }
        forecast_lines.push(Line::from(vec![
            Span::styled("-< ", theme::Meaning::Informational.style()),
            Span::raw(format!(
                "Mission to {} - window {}",
                mission.destination,
                app.date(mission.launch_window)
            )),
            Span::styled(
                format!("  {}% chance of working", mission.probability / 100),
                theme::historical(),
            ),
        ]));
    }
    lines.extend(forecast_lines);
}
/// The slack figure, as a player reads it.
fn slack_figure(slack: Option<domain::time::Duration>) -> String {
    match slack {
        Some(slack) if slack.is_zero() => "no slack left".to_owned(),
        Some(slack) => format!("{slack} of slack"),
        None => "no slack".to_owned(),
    }
}
/// The sort keys, in the order they are offered.
///
/// **The record reads newest first by default**, because that is how a history
/// is read, and the other order is there for the player who wants the worst of
/// it at the top.
pub const SORTS: &[&str] = &["recent", "severity"];

/// The fields the chronicle can be filtered by.
pub const FIELDS: &[crate::list_control::Field] = &[
    crate::list_control::Field::number("year"),
    crate::list_control::Field::text("severity"),
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

/// The chronicle's filter.
///
/// **The history is long and the player is looking for one thing in it.** The
/// filter is three axes — the year, the category, and the text — because those
/// are the three questions a player actually asks of a record: *when*, *what
/// kind*, and *about what*.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChronicleFilter {
    /// The year to show, or every year.
    pub year: Option<u32>,
    /// The category to show, or every category.
    pub category: Option<domain::chronicle::Category>,
}

impl ChronicleFilter {
    /// Whether an entry passes the filter.
    #[must_use]
    pub fn matches(
        &self,
        entry: &domain::chronicle::ChronicleEntry,
        terms: &[crate::list_control::Term],
        state: &domain::state::CampaignState,
    ) -> bool {
        if let Some(year) = self.year {
            if state.calendar().date(entry.date).year != i32::try_from(year).unwrap_or(i32::MAX) {
                return false;
            }
        }
        if let Some(category) = self.category {
            if entry.category != category {
                return false;
            }
        }
        // **The terms.** A word is a search, and a condition is a question
        // about the entry: which year, and how bad.
        let year = state.calendar().date(entry.date).year;
        let severity = entry.severity.label().to_lowercase();
        // **The subject names are only worked out when there is a word to
        // look for.** A record is about people and places, and a player looking
        // for one of them is looking for a name.
        let mut subjects: Vec<String> = Vec::new();
        if terms.iter().any(crate::list_control::Term::is_text) {
            subjects = entry
                .subjects
                .iter()
                .map(|subject| subject_label(subject, state))
                .collect();
        }
        terms.iter().all(|term| {
            let value = term.field().and_then(|name| match name {
                "year" => Some(crate::list_control::FieldValue::Number(i64::from(year))),
                "severity" => Some(crate::list_control::FieldValue::Text(&severity)),
                _ => None,
            });
            let mut texts: Vec<&str> = vec![entry.headline.as_str(), severity.as_str()];
            texts.extend(subjects.iter().map(String::as_str));
            term.holds(&texts, value.as_ref())
        })
    }

    /// Moves to the next year, or back to every year.
    pub fn cycle_year(&mut self, years: &[u32]) {
        let Some(current) = self.year else {
            self.year = years.first().copied();
            return;
        };
        let Some(index) = years.iter().position(|year| *year == current) else {
            self.year = years.first().copied();
            return;
        };
        self.year = if index + 1 < years.len() {
            years.get(index + 1).copied()
        } else {
            None
        };
    }

    /// Moves to the next category, or back to every category.
    pub fn cycle_category(&mut self) {
        let Some(current) = self.category else {
            self.category = domain::chronicle::Category::ALL.first().copied();
            return;
        };
        let all = domain::chronicle::Category::ALL;
        let Some(index) = all.iter().position(|category| *category == current) else {
            self.category = all.first().copied();
            return;
        };
        self.category = if index + 1 < all.len() {
            all.get(index + 1).copied()
        } else {
            None
        };
    }

    /// The filter, as a player reads it.
    #[must_use]
    pub fn summary(&self) -> String {
        let year = self
            .year
            .map_or_else(|| "every year".to_owned(), |year| year.to_string());
        let category = self.category.map_or_else(
            || "everything".to_owned(),
            |category| category.label().to_owned(),
        );
        format!("{year} · {category}")
    }
}

/// The name of a subject, for the text filter.
///
/// **A name, never an identifier.** The filter is for finding a thing in the
/// history, and the player finds it by the name they know it by.
fn subject_label(
    subject: &domain::chronicle::SubjectRef,
    state: &domain::state::CampaignState,
) -> String {
    match subject {
        domain::chronicle::SubjectRef::Programme(id) => state.programmes.get(id).map_or_else(
            || "a programme that has closed".to_owned(),
            |programme| programme.title.clone(),
        ),
        domain::chronicle::SubjectRef::Person(id) => state.people.get(id).map_or_else(
            || "someone who has left".to_owned(),
            |person| person.name.clone(),
        ),
        domain::chronicle::SubjectRef::Facility(id) => state.facilities.get(id).map_or_else(
            || "a facility that has been demolished".to_owned(),
            |facility| facility.name.clone(),
        ),
        domain::chronicle::SubjectRef::Vehicle(id) => state.vehicles.get(id).map_or_else(
            || "a vehicle that has been scrapped".to_owned(),
            |vehicle| vehicle.name.clone(),
        ),
    }
}
/// Draws the chronicle screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.sim().state();
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // The right column: the forecast, and the legend.
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(11)])
        .split(columns[1]);

    // History.
    //
    // **The filter is three axes** - the year, the category, and the text -
    // and the line above the history says which of them is in force, because a
    // filtered list with no visible filter is a list the player cannot trust.
    let filter = app.chronicle_filter();
    let control = app.list_control();
    let terms = control.terms(FIELDS);
    let mut lines = vec![Line::styled(
        format!("  showing {}", filter.summary()),
        theme::historical(),
    )];

    // **The record reads newest first, unless the player asks for the worst
    // of it at the top.**
    let mut entries: Vec<&domain::chronicle::ChronicleEntry> =
        state.chronicle.iter().rev().collect();
    if control.sort() == 1 {
        entries.sort_by_key(|entry| std::cmp::Reverse(severity_rank(entry.severity)));
    }

    for entry in entries.into_iter().take(40) {
        if !filter.matches(entry, &terms, state) {
            continue;
        }
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
    // **The filter and the sort**, so the player can see what is in force and
    // what they may type.
    lines.push(Line::from(vec![
        Span::styled(" filter ", theme::emphasis()),
        Span::raw(format!("{:<20}", control.filter())),
        Span::styled(" sort ", theme::emphasis()),
        Span::raw(format!("{:<8}", control.sort_name(SORTS))),
        Span::styled(" s ", theme::historical()),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" fields ", theme::historical()),
        Span::styled(field_names(), theme::historical()),
    ]));

    let history = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("History"))
        .wrap(Wrap { trim: false });
    frame.render_widget(history, columns[0]);

    let mut lines = Vec::new();
    forecast(app, state, &mut lines);

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
    frame.render_widget(forecast, right[0]);

    // **The legend.** The categories the record is sorted into, and what each
    // of them means, so that a player reading "Financial" knows what kind of
    // thing they are looking at.
    let legend = Paragraph::new(legend_lines())
        .block(Block::default().borders(Borders::ALL).title("Legend"))
        .wrap(Wrap { trim: false });
    frame.render_widget(legend, right[1]);
}

/// The legend: the categories, and what they mean.
fn legend_lines() -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for category in domain::chronicle::Category::ALL {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<14}", category.label()), theme::title()),
            Span::raw(category_meaning(category)),
        ]));
    }
    lines
}

/// How bad a chronicle entry is, for the sort.
fn severity_rank(severity: domain::chronicle::Severity) -> u8 {
    match severity {
        domain::chronicle::Severity::Info => 0,
        domain::chronicle::Severity::Minor => 1,
        domain::chronicle::Severity::Major => 2,
        domain::chronicle::Severity::Critical => 3,
    }
}

/// What a category of the record means.
fn category_meaning(category: domain::chronicle::Category) -> &'static str {
    use domain::chronicle::Category;
    match category {
        Category::Technical => "equipment, vehicles, and failures",
        Category::Personnel => "the people, and who left",
        Category::Discovery => "what the agency found out",
        Category::Political => "the state, and what it promised",
        Category::Financial => "the money, and what it cost",
        Category::Environmental => "the weather, and the world",
        Category::Public => "the public, and the press",
        Category::External => "everything outside the agency",
    }
}
