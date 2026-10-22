//! The opening briefing.
//!
//! **The game opens with the framing and a clear first goal**, and the
//! briefing is a modal over the dashboard: any key dismisses it, and the
//! world waits while the player reads it.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use domain::time::Duration;
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn app() -> App {
    let scenario = Scenario::first_orbit();
    App::new(sim::Sim::new(&scenario).expect("the campaign starts"))
}

/// The screen, as the player sees it.
fn screen(app: &mut App) -> String {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).expect("the terminal starts");
    terminal
        .draw(|frame| app.render(frame))
        .expect("the interface draws");
    let buffer = terminal.backend().buffer().clone();
    buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>()
}

#[test]
fn the_game_opens_with_the_briefing() {
    let mut app = app();
    assert!(app.briefing(), "the briefing is the first thing on screen");
    let text = screen(&mut app);
    assert!(
        text.contains("Welcome, Director"),
        "the briefing says who the player is:\n{text}"
    );
    assert!(
        text.contains("First Orbit"),
        "and which campaign they are in:\n{text}"
    );
}

#[test]
fn the_briefing_says_what_the_first_goal_is() {
    // **A clear first goal.** The campaign opens with a promise to the
    // government, and the briefing says what it is and when it is due.
    let mut app = app();
    let text = screen(&mut app);
    assert!(
        text.contains("reach orbit"),
        "the briefing says what the player is trying to do:\n{text}"
    );
}

#[test]
fn any_key_dismisses_the_briefing() {
    let mut app = app();
    app.on_key(press(KeyCode::Enter));
    assert!(!app.briefing(), "and the first key dismisses it");
    let text = screen(&mut app);
    assert!(
        !text.contains("Welcome, Director"),
        "and it is not on screen afterwards"
    );
}

#[test]
fn the_world_waits_while_the_briefing_is_read() {
    // **The campaign does not move while the player is reading the opening.**
    // The real time is still consumed, so there is no backlog afterwards.
    let mut app = app();
    let before = app.sim().now();
    app.advance(Duration::from_hours(3));
    assert_eq!(
        app.sim().now(),
        before,
        "the campaign waits for the player to begin"
    );

    app.on_key(press(KeyCode::Enter));
    app.advance(Duration::from_hours(3));
    assert!(
        app.sim().now() > before,
        "and it moves once the briefing is dismissed"
    );
}

#[test]
fn a_key_release_does_not_dismiss_the_briefing() {
    // **Only a press is an action**, and the briefing is an action.
    let mut app = app();
    app.on_key(KeyEvent::new_with_kind(
        KeyCode::Enter,
        KeyModifiers::NONE,
        KeyEventKind::Release,
    ));
    assert!(app.briefing(), "a release does not dismiss it");
}
