//! The screens.
//!
//! **Every screen renders, and every screen says something.** A screen that
//! panics loses the campaign; a screen that renders nothing is a screen the
//! player cannot use, and it is just as bad as a crash because nothing says it
//! is broken.
//!
//! The empty-campaign cases matter most. **A screen is written once, when the
//! campaign is full, and it is read for the first time when the campaign is
//! empty** — and a screen that assumes there is a programme, or a person, or a
//! facility, is a screen that crashes on the player's second game.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use domain::time::Duration;
use sim::Scenario;
use ui::App;
use ui::screen::Screen;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// A campaign with the scenario's people, projects, programme and facilities.
fn full() -> App {
    let mut app =
        App::started(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"));
    app.advance(Duration::from_days(400));
    app.on_key(press(KeyCode::Char('x')));
    app
}

/// A campaign with nothing in it at all.
fn empty() -> App {
    let mut scenario = Scenario::first_year();
    scenario.start_state.people.clear();
    scenario.start_state.projects.clear();
    scenario.start_state.programmes.clear();
    scenario.start_state.facilities.clear();
    scenario.start_state.political.promises.clear();
    App::started(sim::Sim::new(&scenario).expect("the campaign starts"))
}

/// The whole interface, as text.
fn rendered(app: &mut App) -> String {
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

/// The text with the box-drawing characters stripped, so a test can look at
/// what the screen says rather than at how it is drawn.
fn words(text: &str) -> String {
    text.chars()
        .filter(|character| {
            !character.is_ascii_whitespace() && *character != '│' && *character != '─'
        })
        .collect()
}

#[test]
fn every_screen_renders() {
    // **The positive case.** Every screen draws, on a campaign with something
    // in it.
    for screen in Screen::ALL {
        let mut app = full();
        app.on_key(press(KeyCode::Char(
            char::from_digit(u32::from(screen.key()), 10).expect("a digit"),
        )));
        let text = rendered(&mut app);
        assert!(
            text.contains("Stellar Expansion Authority"),
            "the {screen:?} screen draws",
        );
    }
}

#[test]
fn every_screen_renders_on_an_empty_campaign() {
    // **The negative case, and the one that matters.** A screen that assumes
    // there is something to show is a screen that crashes on the player's
    // second game, when the agency is a week old and has nothing.
    for screen in Screen::ALL {
        let mut app = empty();
        app.on_key(press(KeyCode::Char(
            char::from_digit(u32::from(screen.key()), 10).expect("a digit"),
        )));
        let text = rendered(&mut app);
        assert!(
            text.contains("Stellar Expansion Authority"),
            "the {screen:?} screen draws on an empty campaign",
        );
    }
}

#[test]
fn every_screen_says_something() {
    // **A screen that renders nothing is as broken as one that crashes.** The
    // screen has to carry the frame and the clock whatever else it has.
    for screen in Screen::ALL {
        let mut app = empty();
        app.on_key(press(KeyCode::Char(
            char::from_digit(u32::from(screen.key()), 10).expect("a digit"),
        )));
        let text = words(&rendered(&mut app));
        assert!(
            text.len() > 40,
            "the {screen:?} screen says something, and it said {text:?}",
        );
    }
}

#[test]
fn every_screen_carries_the_key_ledger() {
    // **The ledger is on every screen**, because a player who has to find the
    // keys on one screen has to find them on all of them.
    for screen in Screen::ALL {
        let mut app = full();
        app.on_key(press(KeyCode::Char(
            char::from_digit(u32::from(screen.key()), 10).expect("a digit"),
        )));
        let text = words(&rendered(&mut app));
        assert!(
            text.contains("order") && text.contains("pause") && text.contains("quit"),
            "the {screen:?} screen carries the ledger",
        );
    }
}

#[test]
fn the_empty_campaign_says_that_it_is_empty() {
    // **The negative case, said out loud.** "The agency has no programmes"
    // is a sentence; an empty panel is a puzzle.
    let mut app = empty();
    app.on_key(press(KeyCode::Char('3')));
    let text = words(&rendered(&mut app));
    assert!(
        text.contains("noprogrammes") || text.contains("nofacilities"),
        "and it says so: {text:?}"
    );
}

#[test]
fn the_report_can_be_dismissed_and_does_not_come_back() {
    // **The report is a prompt.** It waits until it is read, and it is not
    // replaced while it is undismissed.
    let mut app =
        App::started(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"));
    app.advance(Duration::from_days(400));
    assert!(app.report().is_some(), "the month has reported");
    app.on_key(press(KeyCode::Char('x')));
    assert!(app.report().is_none(), "and one key dismisses it");
    app.on_key(press(KeyCode::Char('x')));
    assert!(app.report().is_none(), "and it does not come back");
}

#[test]
fn the_order_palette_opens_and_closes() {
    // **The positive case of the modal**, and the negative of the key: the
    // palette takes every key until it is closed.
    let mut app = full();
    app.on_key(press(KeyCode::Char('o')));
    assert!(app.palette().is_some(), "the palette opens");
    app.on_key(press(KeyCode::Esc));
    assert!(app.palette().is_none(), "and escape closes it");
}

#[test]
fn the_palette_refuses_an_order_it_cannot_build() {
    // **The negative case of the palette.** A recipe with no parameters
    // cannot be built, and the palette must not offer it.
    let app = full();
    for recipe in ui::order_palette::Recipe::ALL {
        let parameters = recipe.parameters(app.sim().state());
        if !recipe.available(app.sim().state()) {
            continue;
        }
        assert!(
            recipe.build(&parameters).is_some(),
            "{} can be built from its own parameters",
            recipe.label()
        );
    }
}

#[test]
fn the_manual_opens_from_every_screen() {
    // **The manual is the answer to "what is this", and it has to be
    // reachable from everywhere.**
    for screen in Screen::ALL {
        let mut app = full();
        app.on_key(press(KeyCode::Char(
            char::from_digit(u32::from(screen.key()), 10).expect("a digit"),
        )));
        app.on_key(press(KeyCode::F(2)));
        assert!(
            app.manual_open(),
            "the manual opens from the {screen:?} screen",
        );
    }
}
