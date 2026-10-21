//! The research and facilities screen: two panes side by side.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme;

/// Draws the research and facilities screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.sim().state();
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Research.
    let mut lines = Vec::new();
    for project in state.projects.values() {
        lines.push(Line::styled(
            format!("  {} — {}%", project.title, project.progress.percent()),
            theme::emphasis(),
        ));
        lines.push(Line::raw(format!(
            "    {}   team {}",
            domain::research::discipline_name(project.discipline),
            project.team.len()
        )));
        lines.push(Line::raw(format!(
            "    cost {}   spent {}",
            sim::balance::research_cost_per_month(project.team.len()),
            project.funding.spent
        )));
        lines.push(Line::raw(""));
    }
    for campaign in state.test_campaigns.values() {
        let name = domain::research::capability_name(campaign.target_capability);
        lines.push(Line::styled(
            format!("  Test: {name} — {}%", campaign.progress.percent()),
            theme::emphasis(),
        ));
        lines.push(Line::raw(format!(
            "    team {}   risk reduction {} points",
            campaign.team.len(),
            campaign.risk_reduction
        )));
        lines.push(Line::raw(""));
    }
    if lines.is_empty() {
        lines.push(Line::raw("  No research is running."));
    }

    let research = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Research"))
        .wrap(Wrap { trim: false });
    frame.render_widget(research, columns[0]);

    // Facilities.
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
    frame.render_widget(facilities, columns[1]);
}
