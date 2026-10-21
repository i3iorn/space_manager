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

    testing_panel(state, &mut lines);

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
            app.date(mission.launch_window),
            mission.probability / 100,
            mission.crew.len()
        )));
        // **The phase and the flight time**, so the player can see what the
        // mission is doing and how long it has left.
        lines.push(Line::raw(format!(
            "    {}   {} of flight ahead",
            mission.phase.label(),
            flight_time_remaining(mission),
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

/// What testing would buy.
///
/// **This is the decision the slice exists for**, and it is the decision the
/// player makes *instead* of building a better vehicle: the money and the months
/// go on the test, and the launch gets safer.
fn testing_panel(state: &domain::state::CampaignState, lines: &mut Vec<Line<'static>>) {
    if state.unlocked.is_empty() {
        return;
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled("  What testing would buy:", theme::title()));
    for capability in &state.unlocked {
        let name = domain::research::capability_name(*capability);
        let tested = state.tested.get(capability).copied().unwrap_or(0);
        let cost = sim::balance::test_cost_per_month(1);
        let (what, style) = if tested > 0 {
            (
                format!("{tested} points of risk already taken out"),
                theme::Meaning::Good.style(),
            )
        } else {
            (
                format!("untested - a campaign would cost {cost} a month"),
                theme::Meaning::Attention.style(),
            )
        };
        lines.push(Line::from(vec![
            Span::raw(format!("    {name:<28}")),
            Span::styled(what, style),
        ]));
    }
}
/// How much flight is left in a mission.
///
/// **The phases have real durations** and the mission is in one of them, so the
/// time left is the rest of this phase and the phases after it.
fn flight_time_remaining(mission: &domain::mission::Mission) -> String {
    let mut left = mission.phase.duration();
    let mut phase = mission.phase.clone();
    while let Some(next) = phase.next() {
        left = domain::time::Duration::from_millis(
            left.millis().saturating_add(next.duration().millis()),
        );
        phase = next;
    }
    if left == domain::time::Duration::ZERO {
        "none".to_owned()
    } else {
        format!("{left}")
    }
}
