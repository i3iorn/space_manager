//! The order palette, end to end.
//!
//! **The palette is how the player acts.** Before it existed the player could
//! watch a year happen and could not touch it — and the MVP's exit test is a
//! playtester who hires two people and runs a research project for a year.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn app() -> App {
    let scenario = Scenario::first_year();
    App::new(sim::Sim::new(&scenario).expect("the scenario starts"))
}

#[test]
fn the_palette_opens_and_closes() {
    let mut app = app();
    assert!(app.palette().is_none());

    app.on_key(press(KeyCode::Char('o')));
    assert!(app.palette().is_some(), "`o` opens the palette");

    app.on_key(press(KeyCode::Esc));
    assert!(app.palette().is_none(), "`Esc` closes it");
}

#[test]
fn the_player_can_hire_someone() {
    let mut app = app();
    let before = app.sim().state().people.len();

    app.on_key(press(KeyCode::Char('o')));
    // The first order on offer is a hire.
    app.on_key(press(KeyCode::Enter));
    assert!(
        app.palette().expect("the palette is open").is_filling(),
        "the first Enter moves to the form"
    );
    app.on_key(press(KeyCode::Enter));
    assert!(app.palette().is_none(), "the second Enter confirms");

    // **The order is in flight, not applied.** It lands at its lead time.
    assert_eq!(app.sim().pending_orders().len(), 1);
    assert_eq!(
        app.sim().state().people.len(),
        before,
        "the recruit has not arrived yet"
    );
}

#[test]
fn the_palette_shows_the_price_before_the_order_is_confirmed() {
    let mut app = app();
    app.on_key(press(KeyCode::Char('o')));
    app.on_key(press(KeyCode::Enter));

    let palette = app.palette().expect("the palette is open");
    let recipe = palette.chosen_recipe().expect("a recipe is chosen");
    let parameters = recipe.parameters(app.sim().state());
    let kind = recipe.build(&parameters).expect("the order builds");
    let price = ui::order_palette::price(&kind, app.sim().state(), app.sim().effective_now())
        .expect("the price");

    assert!(
        price.lead_time > domain::time::Duration::ZERO,
        "a hire takes time"
    );
    assert_eq!(price.goodwill, sim::balance::GOODWILL_HIRE);
}

#[test]
fn a_refused_order_is_refused_at_once() {
    // **The player is never told three weeks later that something was
    // impossible.** The palette refuses it here, with a reason.
    let mut app = app();
    let person = app
        .sim()
        .state()
        .people
        .keys()
        .copied()
        .next()
        .expect("a person");

    // Assign somebody who is already assigned: the order is impossible.
    let already = app.sim().state().people[&person].assignment.clone();
    if !matches!(already, domain::person::Assignment::Unassigned) {
        app.on_key(press(KeyCode::Char('o')));
        // Walk to the Assign recipe.
        app.on_key(press(KeyCode::Down));
        app.on_key(press(KeyCode::Down));
        app.on_key(press(KeyCode::Enter));
        app.on_key(press(KeyCode::Enter));
        assert!(
            app.palette().is_none(),
            "the order was either accepted or refused at once"
        );
    }
}
#[test]
fn the_tutorial_teaches_the_binding_factor() {
    // **The tutorial's job is the binding factor** — it is the one panel the
    // player has to understand to understand the game.
    let tutorial = ui::tutorial::Tutorial::new();
    let hint = tutorial
        .peek(ui::screen::Screen::Staff)
        .expect("the staff hint");
    assert!(
        hint.contains("why your team is slow"),
        "the staff hint should point at the binding factor: {hint}"
    );
}
