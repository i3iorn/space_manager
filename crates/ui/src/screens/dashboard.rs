//! The dashboard — the screen that answers "where do I stand right now".
//!
//! Shows the headline, the upcoming obligations, and the financial position.
//!
//! # The two halves
//!
//! **The left half is the agenda and the right half is the money.** The
//! player comes back to this screen after every month report, and the two
//! questions they have are "what is due" and "where is the money going".
//! Putting the budget beside the agenda means both are answered without
//! leaving the screen.

use domain::money::Money;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme;

/// Draws the dashboard.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    // **The banner is four rows, the headline three, and the rest four.**
    //
    // **There is no picture on the dashboard.** The finest solid pixel a
    // terminal can draw is a whole cell across, so an icon in the corner of a
    // panel is either a blob or it eats the panel's width — nine columns of a
    // sixty-column panel for a shape too coarse to read. **The flare is in the
    // panels, the glyphs and the colours**, and the panels are worth more than
    // a picture would be.
    let banner_rows = 4;
    let headline_rows = 3;
    let panel_rows = 4;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(banner_rows), Constraint::Min(0)])
        .split(area);

    // **The banner.** What to do next, read from the campaign.
    //
    // **It is at the top of the home screen because it is the first thing the
    // player needs and the last thing they can work out.** A player who does
    // not know what the agency should be doing next has no way to find out
    // except by reading a manual, and a manual is a place nobody goes when
    // they are stuck.
    render_banner(app, frame, rows[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    render_agenda(app, frame, columns[0], headline_rows, panel_rows);
    render_budget(app, frame, columns[1]);
}

/// The banner: one piece of advice, and the key that shows the next one.
///
/// **The advice is situational and it is never stored.** It is read from the
/// campaign every time the screen is drawn, so it always matches the campaign
/// the player is actually in, and it is never a script the player has to
/// follow.
fn render_banner(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("What now");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(text) = app.advice() else {
        frame.render_widget(Paragraph::new("  Nothing needs saying."), inner);
        return;
    };
    // **The key that shows the next tip is on the same line as the tip**, so
    // the banner stays the height it is whatever the tip says. A hint about how
    // to use the hint, on a line of its own, is a line the banner cannot
    // afford.
    let mut spans = vec![Span::styled("  › ", theme::title()), Span::raw(text)];
    if app.advice_count() > 1 {
        spans.push(Span::styled("  — press n for another", theme::historical()));
    }
    let banner = Paragraph::new(Line::from(spans)).wrap(Wrap { trim: false });
    frame.render_widget(banner, inner);
}

/// The left half: the headline, the live feed, what is due, and the standing.
fn render_agenda(app: &App, frame: &mut Frame, area: Rect, headline_rows: u16, panel_rows: u16) {
    let state = app.sim().state();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(headline_rows),
            Constraint::Min(panel_rows),
            Constraint::Min(panel_rows),
            Constraint::Min(panel_rows),
            Constraint::Length(panel_rows),
        ])
        .split(area);

    // The headline. The player's eye goes here first.
    let headline = Paragraph::new(Line::styled(
        format!(" {}", state.agency.focus),
        theme::emphasis(),
    ))
    .block(Block::default().borders(Borders::ALL).title("Headline"));
    frame.render_widget(headline, rows[0]);

    // **The month, as a panel.** It used to be a modal over the whole
    // dashboard, and it took every key until it was dismissed — so a player
    // who wanted to look at the budget had to dismiss the month first, and a
    // player who wanted to order something had to dismiss it twice.
    render_month(app, frame, rows[1]);

    // **The live feed.** What has just happened, newest first.
    //
    // **A feed is not a list.** The player watches this one, and it moves
    // without them doing anything: the newest thing is at the top, and it
    // appears the moment it happens. The chronicle is still the durable
    // record; this is the view of it that moves.
    let feed = Paragraph::new(feed_lines(app, 12))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Just happened"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(feed, rows[2]);

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
    frame.render_widget(due, rows[3]);

    // The agency's standing: what the world thinks of it, and how close it is
    // to the crisis.
    //
    // **The money is on the other half of the screen**, so this panel is for
    // the things the budget does not cover.
    let standing = Block::default().borders(Borders::ALL).title("Standing");
    let inner = standing.inner(rows[4]);
    frame.render_widget(standing, rows[4]);
    let panel = Paragraph::new(standing_panel(state)).wrap(Wrap { trim: false });
    frame.render_widget(panel, inner);
}

/// The month's report, as a panel.
///
/// **The report is not a prompt.** It used to be a modal over the whole
/// dashboard, and it took every key until it was dismissed — so a player who
/// wanted to look at the budget had to dismiss the month first, and a player
/// who wanted to order something had to dismiss it twice. It is a panel now,
/// and the dashboard is never blocked.
///
/// **The events are not repeated here.** They are in the feed, three rows
/// below, and a thing said twice is a thing said twice. What is here is what
/// the feed does not carry: the month's one thing that needed deciding, the
/// warnings, the refusals, and the opportunities.
fn render_month(app: &App, frame: &mut Frame, area: Rect) {
    let Some(report) = app.report() else {
        let panel = Paragraph::new("  The month has not turned yet.")
            .block(Block::default().borders(Borders::ALL).title("This month"))
            .wrap(Wrap { trim: false });
        frame.render_widget(panel, area);
        return;
    };

    let mut lines = Vec::new();
    if !report.headline.is_empty() {
        lines.push(Line::styled(
            format!("  {}", report.headline),
            theme::emphasis(),
        ));
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
    for opportunity in &report.opportunities {
        lines.push(Line::styled(
            format!("  ⓘ {opportunity}"),
            theme::Meaning::Informational.style(),
        ));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  Nothing needed saying."));
    }

    let panel = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" This month · {} ", report.date)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(panel, area);
}

/// The live feed, as the player reads it.
///
/// **The player's own decisions are in the feed**, in a different colour from
/// the world's news, because a campaign is the world *and* what the player did
/// about it.
fn feed_lines(app: &App, limit: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for item in sim::feed::feed(app.sim(), limit) {
        let date = app.date(item.at());
        let (glyph, style) = if item.is_decision() {
            ("»", theme::title())
        } else {
            match item.severity() {
                domain::chronicle::Severity::Info | domain::chronicle::Severity::Minor => (
                    theme::Meaning::Informational.glyph(),
                    theme::Meaning::Informational.style(),
                ),
                domain::chronicle::Severity::Major => (
                    theme::Meaning::Attention.glyph(),
                    theme::Meaning::Attention.style(),
                ),
                domain::chronicle::Severity::Critical => {
                    (theme::Meaning::Bad.glyph(), theme::Meaning::Bad.style())
                }
            }
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {date} "), theme::historical()),
            Span::styled(format!("{glyph} "), style),
            Span::styled(item.text().to_owned(), style),
        ]));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  Nothing has happened yet."));
    }
    lines
}

/// The standing panel: the public's opinion, the prestige, and the crisis.
///
/// **The public's opinion is shown here because the world reads it.** The
/// weighting that decides which events fire looks at the public's opinion, so
/// a number the player cannot see is a number the player cannot act on. The
/// state's approval is a different thing and is in the top bar.
fn standing_panel(state: &domain::state::CampaignState) -> Vec<Line<'static>> {
    let opinion = state.public.opinion.percent();
    let opinion_style = if opinion < -30 {
        theme::Meaning::Bad.style()
    } else if opinion < 0 {
        theme::Meaning::Attention.style()
    } else {
        theme::historical()
    };
    let lines = vec![
        Line::from(vec![
            Span::raw("  public opinion "),
            Span::styled(format!("{opinion:+}"), opinion_style),
            Span::raw("   prestige "),
            Span::styled(
                format!("{:+}", state.public.prestige.percent()),
                theme::historical(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  {}", state.treasury.crisis_stage.label()),
                if state.treasury.crisis_stage == domain::finance::CrisisStage::None {
                    theme::historical()
                } else {
                    theme::Meaning::Attention.style()
                },
            ),
            // **The cut, on the same line as the stage that caused it.** The
            // panel is two rows tall and the player must not have to look
            // somewhere else to find out that the state has stopped paying.
            if state.treasury.crisis_stage.in_austerity() {
                Span::styled(
                    format!(": funding cut {}%", sim::balance::AUSTERITY_CUT_PERCENT),
                    theme::Meaning::Attention.style(),
                )
            } else {
                Span::raw("")
            },
        ]),
    ];
    lines
}

/// The right half: the budget.
///
/// **This is the panel the player asked for.** The top bar has the balance,
/// the net and the runway, and the finance strip used to have the totals; what
/// was missing was the breakdown — *where is the money going*, line by line,
/// and what has already been promised.
///
/// Four things, in the order the player needs them:
///
/// 1. **The position.** The balance, what comes in, what goes out, the net, and
///    the runway.
/// 2. **Where it goes.** Every outgoing line, with a bar showing its share of
///    the total, because a number without a share does not answer "where".
/// 3. **Where it comes from.** Every income source.
/// 4. **What is committed.** The orders in flight, and what they will cost when
///    they land.
fn render_budget(app: &App, frame: &mut Frame, area: Rect) {
    let panel = Paragraph::new(budget_panel(app.sim().state()))
        .block(Block::default().borders(Borders::ALL).title("Budget"))
        .wrap(Wrap { trim: false });
    frame.render_widget(panel, area);
}

/// The budget panel.
fn budget_panel(state: &domain::state::CampaignState) -> Vec<Line<'static>> {
    let mut lines = position_lines(state);
    lines.extend(outgoing_lines(state));
    lines.extend(incoming_lines(state));
    lines.extend(committed_lines(state));
    lines.extend(budget_lines(state));
    lines
}

/// **1. The position.** The balance, what comes in, what goes out, the net, and
/// the runway.
fn position_lines(state: &domain::state::CampaignState) -> Vec<Line<'static>> {
    let treasury = &state.treasury;
    let net = treasury.net();
    let runway = treasury.runway_months();

    vec![
        Line::from(vec![
            Span::raw("  balance "),
            Span::styled(credits(treasury.balance), theme::title()),
        ]),
        Line::from(vec![
            Span::raw("  income "),
            Span::styled(
                format!("{:<9}", credits(treasury.total_income())),
                theme::historical(),
            ),
            Span::raw("outgoings "),
            Span::styled(credits(treasury.total_outgoings()), theme::historical()),
        ]),
        Line::from(vec![
            Span::raw("  net "),
            Span::styled(
                format!("{:<12}", credits(net)),
                if net.is_negative() {
                    theme::Meaning::Bad.style()
                } else {
                    theme::Meaning::Good.style()
                },
            ),
            Span::raw("runway "),
            Span::styled(
                if runway < u32::MAX {
                    format!("{runway} mo")
                } else {
                    "surplus".to_owned()
                },
                if runway < 3 {
                    theme::Meaning::Bad.style()
                } else {
                    theme::historical()
                },
            ),
        ]),
    ]
}

/// **2. Where it goes.** Every outgoing line, with a bar showing its share of
/// the total.
fn outgoing_lines(state: &domain::state::CampaignState) -> Vec<Line<'static>> {
    let outgoings = &state.treasury.outgoings;
    let total = state.treasury.total_outgoings();

    let mut lines = vec![
        Line::raw(""),
        Line::styled("  Where it goes", theme::title()),
    ];
    if outgoings.is_empty() {
        lines.push(Line::raw("    Nothing is being spent."));
    }
    for (label, amount) in outgoings {
        let share = share_of(*amount, total);
        lines.push(Line::from(vec![
            Span::raw(format!("  {label:<10}")),
            Span::raw(format!("{:>9}  ", credits(*amount))),
            Span::styled(bar(share), theme::Meaning::Informational.style()),
            Span::raw(format!(" {share:>3}%")),
        ]));
    }
    lines
}

/// **3. Where it comes from.** Every income source.
fn incoming_lines(state: &domain::state::CampaignState) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::raw(""),
        Line::styled("  Where it comes from", theme::title()),
    ];
    // **The cut is shown where the money is.** The state allocation is the
    // line the cut comes off, and a number that is a fifth lower than it was
    // last month, with nothing to say why, is the sort of thing a player
    // concludes is a bug.
    let cut = state.treasury.crisis_stage.in_austerity();
    for (source, amount) in &state.treasury.income {
        let mut spans = vec![
            Span::raw(format!("  {:<18}", source.label())),
            Span::raw(format!("{:>9}", credits(*amount))),
        ];
        if cut && *source == domain::finance::IncomeSource::CivilisationBase {
            spans.push(Span::styled(
                format!("  -{}% (austerity)", sim::balance::AUSTERITY_CUT_PERCENT),
                theme::Meaning::Attention.style(),
            ));
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// **4. What is committed.** The orders in flight, and what they will cost.
fn committed_lines(state: &domain::state::CampaignState) -> Vec<Line<'static>> {
    if state.pending_orders.is_empty() {
        return Vec::new();
    }
    let committed: Money = state
        .pending_orders
        .values()
        .fold(Money::ZERO, |total, order| {
            total.checked_add(order.cost).unwrap_or(total)
        });
    let count = state.pending_orders.len();
    vec![
        Line::raw(""),
        Line::styled("  Committed", theme::title()),
        Line::from(vec![
            Span::raw(format!(
                "  {count} order{} in flight",
                if count == 1 { "" } else { "s" }
            )),
            Span::raw(format!("{:>12}", credits(committed))),
        ]),
    ]
}

/// The programme budgets: the money set aside, and how much of it is unspent.
fn budget_lines(state: &domain::state::CampaignState) -> Vec<Line<'static>> {
    let budgeted: Vec<_> = state
        .programmes
        .values()
        .filter(|programme| !programme.budget.allocation.is_zero())
        .collect();
    if budgeted.is_empty() {
        return Vec::new();
    }
    let mut lines = vec![
        Line::raw(""),
        Line::styled("  Programme budgets", theme::title()),
    ];
    for programme in budgeted {
        lines.push(Line::from(vec![
            Span::raw(format!("  {:<18}", programme.title)),
            Span::raw(format!("{:>9} left", credits(programme.budget.remaining()))),
        ]));
    }
    lines
}
/// A money figure, as the player reads it: whole credits, grouped, no cents.
///
/// **The panel is narrow, and the cents are noise.** A player budgeting a
/// salary does not need the minor units; they need the thousands.
fn credits(money: Money) -> String {
    let whole = money.credits();
    let sign = if whole < 0 { "-" } else { "" };
    let digits = whole.unsigned_abs().to_string();
    let mut grouped = String::new();
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    format!("{sign}{grouped}")
}

/// What share of the total an amount is, in whole percent.
fn share_of(amount: Money, total: Money) -> u32 {
    if total.is_zero() {
        return 0;
    }
    let share = amount.credits().abs() * 100 / total.credits().abs();
    u32::try_from(share).unwrap_or(100)
}

/// A bar showing a share of the total.
fn bar(share: u32) -> String {
    let filled = (share * 8) / 100;
    let filled = usize::try_from(filled).unwrap_or(8);
    format!("{}{}", "▇".repeat(filled), "░".repeat(8 - filled))
}
