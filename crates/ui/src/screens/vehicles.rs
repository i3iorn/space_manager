//! The vehicles and missions screen: sliders, derived stats, risk, authorisation.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme;

/// Draws the vehicles and missions screen.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let state = app.sim().state();
    let mut lines = Vec::new();

    if state.vehicles.is_empty() && state.missions.is_empty() {
        lines.push(Line::raw("  The agency has no vehicles."));
        lines.push(Line::raw(""));
        lines.push(Line::styled(
            "  Vehicles arrive with Slice 3 of the build plan.",
            theme::historical(),
        ));
    }

    for vehicle in state.vehicles.values() {
        lines.push(Line::styled(
            format!(
                "  {} — {} ({:?})",
                vehicle.name,
                domain::vehicle::archetype_name(vehicle.archetype),
                vehicle.build_state
            ),
            theme::emphasis(),
        ));
        for (label, value) in vehicle.sliders.as_list() {
            let bar: String = "▇".repeat(usize::from(value) * 10 / 100)
                + &"░".repeat(10 - usize::from(value) * 10 / 100);
            lines.push(Line::from(vec![
                Span::raw(format!("    {label:<14}")),
                Span::styled(bar, theme::Meaning::Informational.style()),
                Span::raw(format!(" {value:>3}")),
            ]));
        }
        // **The numbers the design works out to.** They are derived from the
        // sliders every time they are shown, and they are what the player is
        // actually deciding between.
        let derived = &vehicle.derived;
        lines.push(Line::raw(format!(
            "    costs {}   reliability {}%   capacity {}   delta-v {}",
            derived.cost_estimate, derived.reliability, derived.payload_capacity, derived.delta_v
        )));
        if !derived.required_capabilities.is_empty() {
            lines.push(Line::styled(
                format!(
                    "    needs {}",
                    derived
                        .required_capabilities
                        .iter()
                        .map(|capability| domain::research::capability_name(*capability))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                theme::Meaning::Attention.style(),
            ));
        }
        lines.push(Line::raw(""));
    }

    for mission in state.missions.values() {
        lines.push(Line::styled(
            format!(
                "  Mission to {} — {}",
                mission.destination,
                mission.phase.label()
            ),
            theme::emphasis(),
        ));
        lines.push(Line::raw(format!(
            "    window {}   risk {}%   crew {}",
            mission.launch_window,
            mission.probability / 100,
            mission.crew.len()
        )));
        lines.push(Line::raw(""));
    }

    let body = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Vehicles and missions"),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(body, area);
}
