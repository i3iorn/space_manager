//! The confirmation before the game quits.
//!
//! **Quitting is not the same as losing.** The game saves itself, so leaving
//! is a decision about when to stop. These tests are the guard on that: a
//! stray `q` must not end a campaign, a player who means to leave must be
//! able to, and the dialog must say what is true about the save.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn control_c() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
}

/// The app, with a campaign that is worth losing.
fn campaign() -> App {
    let mut app =
        App::started(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"));
    app.advance(domain::time::Duration::from_days(400));
    app.on_key(press(KeyCode::Char('x')));
    app
}

/// The whole interface, as text.
fn rendered(app: &mut App) -> String {
    let backend = TestBackend::new(110, 40);
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
fn the_q_key_asks_before_it_quits() {
    // **The whole point.** A stray `q` must not end a campaign.
    let mut app = campaign();
    app.on_key(press(KeyCode::Char('q')));
    assert!(!app.should_quit(), "the game did not quit");
    let screen = rendered(&mut app);
    assert!(
        screen.contains("Leave the campaign?"),
        "the confirmation is on the screen: {screen}"
    );
}

#[test]
fn the_confirmation_says_the_campaign_is_saved() {
    // **The dialog says what is true**, because a dialog that hides the cost
    // of the choice is worse than no dialog — and a dialog that lies about the
    // cost is worse still. **The game saves itself**, so quitting is not
    // throwing the evening away, and the player has to be told that.
    let mut app = campaign();
    app.on_key(press(KeyCode::Char('q')));
    let screen = rendered(&mut app);
    assert!(
        screen.contains("saved when you quit"),
        "the warning is on the screen: {screen}"
    );
}

#[test]
fn the_safe_answer_is_the_one_that_starts_selected() {
    // **A player who opens the dialog and presses Enter without reading it has
    // not quit** - that is the mistake the dialog exists to catch.
    let mut app = campaign();
    app.on_key(press(KeyCode::Char('q')));
    app.on_key(press(KeyCode::Enter));
    assert!(!app.should_quit(), "the game did not quit");
    let screen = rendered(&mut app);
    assert!(
        !screen.contains("Leave the campaign?"),
        "the dialog closed: {screen}"
    );
}

#[test]
fn escape_stays_in_the_campaign() {
    let mut app = campaign();
    app.on_key(press(KeyCode::Char('q')));
    app.on_key(press(KeyCode::Esc));
    assert!(!app.should_quit(), "the game did not quit");
    let screen = rendered(&mut app);
    assert!(
        !screen.contains("Leave the campaign?"),
        "the dialog closed: {screen}"
    );
}

#[test]
fn choosing_quit_quits() {
    // **And a player who means to leave can.**
    let mut app = campaign();
    app.on_key(press(KeyCode::Char('q')));
    app.on_key(press(KeyCode::Right));
    app.on_key(press(KeyCode::Enter));
    assert!(app.should_quit(), "the game quit");
}

#[test]
fn control_c_in_the_dialog_leaves_at_once() {
    // **The escape hatch.** The player has been asked already, so a second
    // press is a decision rather than a mistake.
    let mut app = campaign();
    app.on_key(control_c());
    assert!(!app.should_quit(), "the first press asks");
    app.on_key(control_c());
    assert!(app.should_quit(), "the second press leaves");
}

#[test]
fn the_campaign_does_not_advance_while_the_player_decides() {
    // **The question costs no time.** The world waits, and the real time is
    // still consumed, so there is no backlog when the player carries on.
    let mut app = campaign();
    let before = app.sim().effective_now();
    app.on_key(press(KeyCode::Char('q')));
    app.advance(domain::time::Duration::from_days(30));
    assert_eq!(
        app.sim().effective_now(),
        before,
        "the campaign moved while the dialog was up"
    );
    app.on_key(press(KeyCode::Esc));
    app.advance(domain::time::Duration::from_days(30));
    assert!(
        app.sim().effective_now() > before,
        "and it moves again once the dialog is closed"
    );
}
