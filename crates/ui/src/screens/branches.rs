//! The branches screen: fork the campaign, and compare.
//!
//! **Forking is the answer to "what if I had not cancelled that programme?"**
//! The player looks back at a decision, forks the campaign at that instant, and
//! watches the two campaigns play out side by side.
//!
//! The branch is the *same campaign* up to the fork point — the same state, the
//! same chronicle, the same recruits — because otherwise every difference
//! would be noise.
//!
//! # The comparison
//!
//! **Two timelines side by side.** The left column is the original and the
//! right is the branch, and the rows are the things a player would want to
//! compare: the money, the people, the knowledge, the approval. Below them
//! are the chronicle entries that only one of them has, because a difference
//! the player cannot see is a difference the player cannot use.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme;

/// Draws the branches screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let mut lines = Vec::new();

    lines.push(Line::styled(
        "  Fork the campaign and watch the two branches diverge.",
        theme::emphasis(),
    ));
    lines.push(Line::raw(""));

    // The decisions: the timeline of the campaign.
    let decisions = app.decisions();
    if decisions.is_empty() {
        lines.push(Line::raw("  There is nothing to fork yet."));
        lines.push(Line::raw("  The campaign has not been given an order."));
        lines.push(Line::raw(""));
    } else {
        lines.push(Line::styled(
            "  The decisions you have made:",
            theme::title(),
        ));
        for (index, (instant, kind)) in decisions.iter().enumerate() {
            let date = app.sim().calendar().date(*instant).to_string();
            // **The focus is a style and a marker**, so it is findable in a
            // monochrome terminal as well as a colourful one.
            let (marker, style) = if index == app.fork_index() {
                (theme::FOCUS, theme::selected())
            } else {
                (theme::NO_FOCUS, theme::historical())
            };
            lines.push(Line::styled(format!("  {marker} {date:<12} {kind}"), style));
        }
        lines.push(Line::raw(""));
    }

    // The comparison.
    match app.branch() {
        None => {
            lines.push(Line::styled(
                "  Enter forks the campaign at the selected decision.",
                theme::historical(),
            ));
        }
        Some(branch) => {
            let differences = sim::branch::divergence(app.sim().state(), branch.state());
            lines.push(Line::styled(
                format!(
                    "  The branch is at {} — the original is at {}.",
                    app.sim().calendar().date(branch.now()),
                    app.sim().calendar().date(app.sim().now()),
                ),
                theme::title(),
            ));
            lines.push(Line::raw(""));

            if differences.is_empty() {
                lines.push(Line::styled(
                    "  The two branches agree on everything measured.",
                    theme::Meaning::Good.style(),
                ));
            } else {
                // **Two columns, side by side.** The player's eye compares
                // down the rows, and the row is the thing that differs.
                lines.push(Line::from(vec![
                    Span::raw(format!("  {:<16}", "")),
                    Span::styled(format!("{:<12}", "original"), theme::title()),
                    Span::styled("branch", theme::title()),
                ]));
                for difference in &differences {
                    lines.push(Line::from(vec![
                        Span::raw(format!("  {:<16}", difference.subject)),
                        Span::styled(format!("{:<12}", difference.left), theme::historical()),
                        Span::styled(
                            difference.right.clone(),
                            theme::Meaning::Informational.style(),
                        ),
                    ]));
                }
            }

            // **What only one of them has.** A difference the player cannot
            // see is a difference the player cannot use.
            let chronicle = sim::branch::chronicle_differences(app.sim().state(), branch.state());
            if !chronicle.is_empty() {
                lines.push(Line::raw(""));
                lines.push(Line::styled("  Only one of them saw:", theme::title()));
                for (headline, in_original) in chronicle.iter().take(12) {
                    let (which, style) = if *in_original {
                        ("original", theme::historical())
                    } else {
                        ("branch", theme::Meaning::Informational.style())
                    };
                    lines.push(Line::from(vec![
                        Span::raw(format!("  {which:<9} ")),
                        Span::styled(headline.clone(), style),
                    ]));
                }
            }
        }
    }

    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Branches"))
        .wrap(Wrap { trim: false });
    frame.render_widget(body, area);
}
