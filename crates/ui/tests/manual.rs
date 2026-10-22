//! The manual.
//!
//! **The information the game can give, and the three ways to reach it.**
//!
//! These tests are about *reachability*. A topic that exists in the store but
//! cannot be found from the interface is a topic the player does not have, and
//! that is the failure these are looking for.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use sim::Scenario;
use ui::App;
use ui::manual;
use ui::screen::Screen;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn app() -> App {
    App::started(sim::Sim::new(&Scenario::first_year()).expect("the campaign starts"))
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
fn the_manual_opens_and_closes() {
    // **The manual is reachable from anywhere**, and it is a modal over
    // everything else.
    let mut app = app();
    app.on_key(press(KeyCode::F(2)));
    assert!(app.manual_open(), "F2 opens the manual");
    app.on_key(press(KeyCode::Esc));
    assert!(!app.manual_open(), "and escape closes it");
}

#[test]
fn the_manual_is_actually_drawn() {
    // **A flag is not a modal.** The keymap once set a flag and nothing drew
    // it, and the only test checked the flag.
    let mut app = app();
    app.on_key(press(KeyCode::F(2)));
    let text = rendered(&mut app);
    assert!(
        text.contains("The binding factor") && text.contains("Morale"),
        "the manual must be on screen, and it was not"
    );
}

#[test]
fn the_information_view_is_the_manual_restricted_to_the_screen() {
    // **The player's own idea**: every screen has an information view, and
    // you can navigate around to all the different elements on it.
    let mut app = app();
    app.on_key(press(KeyCode::Char('i')));
    assert!(app.manual_open(), "`I` opens the information view");
    assert_eq!(
        app.manual().restricted_to(),
        Some(Screen::Dashboard),
        "and it is restricted to the screen the player is on"
    );

    let text = rendered(&mut app);
    assert!(
        text.contains("Due soon") && text.contains("Standing") && text.contains("Budget"),
        "the dashboard's own panels must be explained, and they were not:\n{text}"
    );
    assert!(
        !text.contains("Attrition"),
        "and the manual restricted to a screen must not range over everything"
    );
}

#[test]
fn every_screen_has_its_own_information() {
    // **Every screen names its own panels**, and every one of them is a topic
    // that exists.
    for screen in Screen::ALL {
        let topics = manual::screens::topics_for(screen);
        assert!(!topics.is_empty(), "{screen:?} explains nothing");
        for id in topics {
            assert!(
                manual::find(id).is_some(),
                "{screen:?} names a topic that does not exist: {id}"
            );
        }
    }
}

#[test]
fn every_order_is_explained() {
    // **An option the player does not understand is an option they will not
    // use.**
    for recipe in ui::order_palette::Recipe::ALL {
        assert!(
            recipe.summary().is_some(),
            "{} has no explanation",
            recipe.label()
        );
    }
}

#[test]
fn the_info_line_explains_the_screen() {
    // **The line above the ledger always says something useful**, so the
    // player does not have to open the manual to know what they are looking
    // at.
    let mut app = app();
    // The programme screen has no tutorial hint, so the info line shows.
    app.on_key(press(KeyCode::Char('3')));
    let text = rendered(&mut app);
    let summary = manual::summary(ui::screens::focus_topic(Screen::Programme))
        .expect("the programme screen has a focus topic");
    assert!(
        text.contains(summary),
        "the info line must show the focus topic's summary"
    );
}

#[test]
fn the_filter_narrows_the_manual() {
    // **Typing is how the manual is searched**, and it is the only way a
    // player can find a topic they half remember.
    let mut app = app();
    app.on_key(press(KeyCode::F(2)));
    let before = app.manual().visible().len();
    for character in "slack".chars() {
        app.on_key(press(KeyCode::Char(character)));
    }
    assert!(
        app.manual().visible().len() < before,
        "typing must narrow the manual"
    );
    let text = rendered(&mut app);
    assert!(text.contains("Slack"), "and it must find the right topic");
}

#[test]
fn typing_in_the_manual_does_not_quit_the_game() {
    // **`q` is a letter in a filter, not the quit key**, and a player typing
    // "quarantine" must not lose the campaign.
    let mut app = app();
    app.on_key(press(KeyCode::F(2)));
    app.on_key(press(KeyCode::Char('q')));
    assert!(app.manual_open(), "the manual is still open");
    assert!(!app.should_quit(), "and the game is still running");
}

#[test]
fn the_manual_remembers_its_filter() {
    // **A filter typed once is still there next time**, so the player does
    // not have to find the same topic twice.
    let mut app = app();
    app.on_key(press(KeyCode::F(2)));
    app.on_key(press(KeyCode::Char('s')));
    app.on_key(press(KeyCode::Esc));
    app.on_key(press(KeyCode::F(2)));
    assert_eq!(app.manual().filter(), "s", "the filter survives");
}

#[test]
fn a_topic_named_by_the_search_comes_first() {
    // **A player who types "binding" wants the binding factor**, not the
    // first topic that happens to mention it in its body.
    let mut manual = ui::manual::Manual::new();
    manual.filter_by("binding");
    let selected = manual.selected().expect("something matches");
    assert_eq!(
        selected.id, "agency.binding",
        "the topic named by the search must come first"
    );
}

#[test]
fn a_long_explanation_can_be_scrolled() {
    // **The longest explanation is over a thousand characters**, and a detail
    // pane in a small terminal cannot show it all. The player scrolls.
    let mut app = app();
    app.on_key(press(KeyCode::F(2)));
    for character in "binding".chars() {
        app.on_key(press(KeyCode::Char(character)));
    }
    let top = app.manual().scroll();
    app.on_key(press(KeyCode::PageDown));
    assert!(
        app.manual().scroll() > top,
        "page down must scroll the explanation"
    );
    app.on_key(press(KeyCode::PageUp));
    assert_eq!(app.manual().scroll(), top, "and page up must scroll back");
}

#[test]
fn a_new_topic_starts_at_the_top() {
    // **A player who scrolls to the bottom of one explanation and moves to
    // the next should not land in the middle of it.**
    let mut app = app();
    app.on_key(press(KeyCode::F(2)));
    for character in "binding".chars() {
        app.on_key(press(KeyCode::Char(character)));
    }
    app.on_key(press(KeyCode::PageDown));
    assert!(app.manual().scroll() > 0, "it is scrolled");
    app.on_key(press(KeyCode::Down));
    assert_eq!(
        app.manual().scroll(),
        0,
        "and the next topic starts at the top"
    );
}
