//! The main menu.
//!
//! **The seven screens are the game; the menu is the system.** Restarting,
//! saving, loading, and changing the clock are decisions about the campaign as a
//! whole, and they do not belong on a screen about staff or money. These tests
//! are the guard on that: the menu opens, it takes the keys, and each thing it
//! offers does what it says.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn fresh_app() -> App {
    App::started(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"))
}

/// A file name of the app's own, so two tests do not fight over one file.
fn scratch(name: &str) -> String {
    let path = std::env::temp_dir().join(format!("space_manager_ui_menu_{name}.json"));
    path.to_string_lossy().into_owned()
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

/// Opens the menu and walks up to an item.
///
/// The menu starts on "Back", so every item is reached by walking up from there.
fn select(app: &mut App, steps_up: usize) {
    app.on_key(press(KeyCode::Char('m')));
    for _ in 0..steps_up {
        app.on_key(press(KeyCode::Up));
    }
}

#[test]
fn the_m_key_opens_the_menu() {
    // **The menu is one key away**, and it is there when the player wants it.
    let mut app = fresh_app();
    assert!(!app.menu_open());
    app.on_key(press(KeyCode::Char('m')));
    assert!(app.menu_open());
    let screen = rendered(&mut app);
    assert!(
        screen.contains("Restart the campaign"),
        "the menu is on the screen: {screen}"
    );
}

#[test]
fn escape_closes_the_menu_and_stays_in_the_campaign() {
    // **The menu is a modal, not a screen.** A player who changes their mind is
    // one key away from the game they were playing.
    let mut app = fresh_app();
    app.on_key(press(KeyCode::Char('m')));
    app.on_key(press(KeyCode::Esc));
    assert!(!app.menu_open());
    assert!(!app.should_quit(), "the game did not quit");
}

#[test]
fn the_menu_starts_on_back() {
    // **The safe answer is the one that starts selected**, the same rule as the
    // confirmation dialog. Enter without reading does not restart anything.
    let mut app = fresh_app();
    let before = app.sim().state_hash();
    app.on_key(press(KeyCode::Char('m')));
    app.on_key(press(KeyCode::Enter));
    assert!(!app.menu_open(), "the menu closed");
    assert_eq!(app.sim().state_hash(), before, "the campaign is unchanged");
}

#[test]
fn save_from_the_menu_writes_the_campaign() {
    let mut app = fresh_app();
    let path = scratch("save");
    app.set_save_path(path.clone());
    app.advance(domain::time::Duration::from_months(2));
    let expected = app.sim().state_hash();

    // "Save" is four steps up from "Back": Back, Quit, Settings, Load, Save.
    select(&mut app, 4);
    app.on_key(press(KeyCode::Enter));
    let written = std::path::Path::new(&path).exists();
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let _ = std::fs::remove_file(&path);

    assert!(written, "the campaign was written to {path}");
    let loaded = sim::Sim::load(&text).expect("the campaign loads");
    assert_eq!(loaded.state_hash(), expected, "and it is the campaign");
}

#[test]
fn load_from_the_menu_asks_first() {
    // **Loading throws the campaign away**, so the menu does not do it without
    // asking — the same dialog as `F4`.
    let mut app = fresh_app();
    // "Load" is three steps up from "Back": Back, Quit, Settings, Load.
    select(&mut app, 3);
    app.on_key(press(KeyCode::Enter));
    assert!(!app.menu_open(), "the menu closed");
    let screen = rendered(&mut app);
    assert!(
        screen.contains("Load the saved campaign?"),
        "the confirmation is on the screen: {screen}"
    );
}

#[test]
fn quit_from_the_menu_asks_first() {
    // **Quitting is not the same as losing**, and the menu asks the same
    // question as `q`.
    let mut app = fresh_app();
    // "Quit" is one step up from "Back".
    select(&mut app, 1);
    app.on_key(press(KeyCode::Enter));
    assert!(!app.menu_open(), "the menu closed");
    assert!(!app.should_quit(), "the game did not quit");
    let screen = rendered(&mut app);
    assert!(
        screen.contains("Leave the campaign?"),
        "the confirmation is on the screen: {screen}"
    );
}

#[test]
fn restart_starts_a_new_campaign() {
    // **The player asked for the beginning**, and the beginning is what they
    // get: the same scenario, from its first instant.
    let mut app = fresh_app();
    app.advance(domain::time::Duration::from_months(6));
    let scenario = app.sim().scenario().id.clone();
    let progressed = app.sim().now();
    assert!(progressed > domain::time::Instant::ZERO);

    // "Restart" is five steps up from "Back": Back, Quit, Settings, Load, Save,
    // Restart.
    select(&mut app, 5);
    app.on_key(press(KeyCode::Enter));

    assert!(!app.menu_open(), "the menu closed");
    assert_eq!(
        app.sim().now(),
        domain::time::Instant::ZERO,
        "back to the start"
    );
    assert_eq!(app.sim().scenario().id, scenario, "the same scenario");
}

#[test]
fn settings_show_the_clock() {
    // **The settings are the host's, not the campaign's.** The one the player
    // can change is how fast the clock runs, and the menu shows it.
    let mut app = fresh_app();
    // "Settings" is two steps up from "Back": Back, Quit, Settings.
    select(&mut app, 2);
    app.on_key(press(KeyCode::Enter));
    let screen = rendered(&mut app);
    assert!(
        screen.contains("Dilation"),
        "the settings are on the screen: {screen}"
    );
    assert!(app.menu_open(), "the menu is still open");
}
