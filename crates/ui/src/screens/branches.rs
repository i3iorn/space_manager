//! The branches screen: fork the campaign, and compare.
//!
//! **Forking is the answer to "what if I had not cancelled that programme?"**
//! The player looks back at a decision, forks the campaign at that instant, and
//! watches the two campaigns play out side by side.
//!
//! The branch is the *same campaign* up to the fork point — the same state, the
//! same chronicle, the same recruits — because otherwise every difference
//! would be noise.

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

    // The fork points: the instants at which the player gave an order.
    let points = app.fork_points();
    if points.is_empty() {
        lines.push(Line::raw("  There is nothing to fork yet."));
        lines.push(Line::raw("  The campaign has not been given an order."));
        lines.push(Line::raw(""));
    } else {
        lines.push(Line::styled("  Fork at:", theme::title()));
        for (index, instant) in points.iter().enumerate() {
            let date = app.sim().calendar().date(*instant).to_string();
            // **The focus is a style and a marker**, so it is findable in a
            // monochrome terminal as well as a colourful one.
            let (marker, style) = if index == app.fork_index() {
                (theme::FOCUS, theme::selected())
            } else {
                (theme::NO_FOCUS, theme::historical())
            };
            lines.push(Line::styled(format!("  {marker} {date}"), style));
        }
        lines.push(Line::raw(""));
    }

    // The comparison.
    match app.branch() {
        None => {
            lines.push(Line::styled(
                "  Enter forks the campaign at the selected instant.",
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
                lines.push(Line::from(vec![
                    Span::styled("  Diverged".to_owned(), theme::title()),
                    Span::styled("original".to_owned(), theme::title()),
                    Span::styled("branch".to_owned(), theme::title()),
                ]));
                for difference in &differences {
                    lines.push(Line::from(vec![
                        Span::raw(format!("  {:<14}", difference.subject)),
                        Span::styled(difference.left.clone(), theme::Meaning::Attention.style()),
                        Span::styled(
                            difference.right.clone(),
                            theme::Meaning::Informational.style(),
                        ),
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
