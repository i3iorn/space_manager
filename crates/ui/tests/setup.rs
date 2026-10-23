//! The setup: the game as the player meets it.
//!
//! **The campaign is a running agency, not a monthly tick.** These tests check
//! the things a player would notice in the first minute: that the clock moves
//! the campaign on its own, that there is no month to end, and that the
//! interface shows what is happening.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

use domain::time::{Duration, Instant, TimeScale};
use sim::Scenario;
use ui::App;

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// A key being let go.
///
/// **Windows reports the release of every key**, and the interface has to
/// ignore it.
fn release(code: KeyCode) -> KeyEvent {
    KeyEvent::new_with_kind(code, KeyModifiers::NONE, KeyEventKind::Release)
}

/// A key pressed with shift held.
fn shifted(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::SHIFT)
}

fn app() -> App {
    let scenario = Scenario::first_year();
    App::started(sim::Sim::new(&scenario).expect("the campaign starts"))
}

#[test]
fn the_campaign_advances_without_any_input() {
    // **This is the whole of the continuous model.** The agency runs whether or
    // not the player is touching anything.
    let mut app = app();
    let before = app.sim().now();
    app.advance(Duration::from_hours(3));
    assert!(
        app.sim().now() > before,
        "the campaign should have moved on by itself"
    );
}

#[test]
fn there_is_no_month_to_end() {
    // **Enter does not resolve a month.** The clock runs the campaign, and the
    // one thing the player can do about it is pause the clock.
    let mut app = app();
    let before = app.sim().now();
    app.on_key(press(KeyCode::Enter));
    assert_eq!(
        app.sim().now(),
        before,
        "Enter must not move the campaign forward"
    );
}

#[test]
fn the_clock_can_be_paused() {
    let mut app = app();
    app.on_key(press(KeyCode::Char(' ')));
    assert!(app.clock().is_paused());
    app.on_key(press(KeyCode::Char(' ')));
    assert!(!app.clock().is_paused());
}

#[test]
fn the_dilation_moves_through_the_nine_steps() {
    let mut app = app();
    let mut seen = Vec::new();
    for _ in 0..8 {
        app.on_key(press(KeyCode::Char('+')));
        seen.push(app.clock().scale());
    }
    // The scale only ever goes one way, and it ends at the top.
    assert_eq!(seen.last().copied(), Some(TimeScale::SixHundredThousand));
    app.on_key(press(KeyCode::Char('-')));
    assert_eq!(app.clock().scale(), TimeScale::ThreeHundredThousand);
}

#[test]
fn the_campaign_is_the_same_at_every_dilation() {
    // **The property the whole time model rests on.** The same campaign, run
    // at two very different tempi, is the same campaign.
    //
    // The two runs cover **the same span of simulated time** and chop it up
    // very differently: at the base rate the frame is a second, and at the top
    // it is nearly three hours.
    let run = |scale: TimeScale| {
        let scenario = Scenario::first_year();
        let mut app = App::started(sim::Sim::new(&scenario).expect("the campaign starts"));
        app.clock_mut().set_scale(scale);
        // The host's frame is a fixed amount of *real* time, and the clock
        // turns it into simulated time.
        let frame = Duration::from_millis(16 * scale.multiplier());
        // The same span of simulated time, in very different pieces.
        let target = Instant::from_hours(100);
        while app.sim().effective_now() < target {
            let now = app.sim().effective_now();
            let next = now.saturating_add(frame).min(target);
            let delta = next.duration_since(now);
            app.advance(delta);
            // **A question on the table is put aside, not answered.** The
            // world stops while the dialog is up, and this loop waits for the
            // campaign to move — so a question nobody answers would stop it
            // for ever. Putting it aside is what a player who is not watching
            // does, and it keeps the clock running.
            if app.decision().is_some() {
                app.on_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
            }
        }
        app.sim().state_hash()
    };
    assert_eq!(run(TimeScale::Sixty), run(TimeScale::SixHundredThousand));
}

#[test]
fn the_scenario_starts_with_something_to_do() {
    // **A small agency with a budget that will not last, three scientists, and
    // a research project already running** — so the player's first decision is
    // about a project that exists.
    let scenario = Scenario::first_year();
    assert_eq!(scenario.start_state.people.len(), 3);
    assert_eq!(scenario.start_state.projects.len(), 1);
    assert_eq!(scenario.start_state.facilities.len(), 1);
}

#[test]
fn the_clock_shows_the_campaign_is_running() {
    // The top bar shows the dilation and the date, and the date moves.
    let mut app = app();
    let before = app
        .sim()
        .calendar()
        .date(app.sim().effective_now())
        .to_string();
    app.advance(Duration::from_days(3));
    let after = app
        .sim()
        .calendar()
        .date(app.sim().effective_now())
        .to_string();
    assert_ne!(before, after, "the date should have moved");
}

#[test]
fn the_report_is_a_panel_and_not_a_prompt() {
    // **The report is the month's summary, and it is a panel.** A newer month
    // replaces the one before it, because the panel says what the last month
    // was like — and it is never dismissed, because the dashboard is never
    // blocked.
    let mut app = app();
    // Past the opening month, so the next boundary is a real one.
    app.advance(Duration::from_days(40));
    assert!(app.report().is_some(), "a month has passed");
    let first = app.report().expect("a report").date.clone();
    app.advance(Duration::from_months(2));
    assert_ne!(
        app.report().expect("a report").date,
        first,
        "a newer month replaces the one before it"
    );
    // **And no key dismisses it.**
    app.on_key(press(KeyCode::Char('x')));
    assert!(app.report().is_some(), "the panel is still there");
}

#[test]
fn the_report_appears_when_a_month_passes_and_not_otherwise() {
    // **The report appears when a month boundary is crossed**, not because the
    // player asked for one.
    let mut app = app();
    // Past the opening month.
    app.advance(Duration::from_days(40));
    assert!(app.report().is_some(), "a month has passed");

    app.advance(Duration::from_hours(1));
    assert!(
        app.report().is_some(),
        "an hour is not a month, and the panel still shows the last one"
    );
    app.advance(Duration::from_days(40));
    assert!(app.report().is_some(), "a month has passed");
}

#[test]
fn the_orders_in_flight_are_visible() {
    // **The agenda.** The player can see what the agency is about to do.
    let mut app = app();
    app.on_key(press(KeyCode::Char('o')));
    app.on_key(press(KeyCode::Enter));
    app.on_key(press(KeyCode::Enter));
    assert_eq!(
        app.sim().pending_orders().len(),
        1,
        "the order is in flight"
    );
}
#[test]
fn the_clock_controls_work_while_the_report_is_up() {
    // **The clock controls are global**, and a modal that swallowed them would
    // take the player's controls away exactly when they wanted to stop and
    // think.
    let mut app = app();
    app.advance(Duration::from_days(40));
    assert!(app.report().is_some(), "the month has reported");

    let before = app.clock().scale();
    app.on_key(press(KeyCode::Char('+')));
    assert_eq!(
        app.clock().scale(),
        before.faster(),
        "the clock must respond while the report is up"
    );
    app.on_key(press(KeyCode::Char(' ')));
    assert!(app.clock().is_paused(), "and the clock must be pausable");
    assert!(
        app.report().is_some(),
        "and the report must still be there to read"
    );
}

#[test]
fn the_keymap_can_be_opened_and_closed() {
    // **The player should not have to leave the game to find the keys.**
    let mut app = app();
    app.on_key(press(KeyCode::Char('?')));
    assert!(app.show_help(), "? opens the keymap");
    app.on_key(press(KeyCode::Char('x')));
    assert!(!app.show_help(), "and any key closes it");
}

#[test]
fn the_keymap_is_actually_drawn() {
    // **A flag is not a modal.**
    //
    // The keymap once set a flag and nothing drew it, and because the test
    // only checked the flag, the keymap was "working" and invisible at the
    // same time. The player's key did nothing, and no test noticed.
    let mut app = app();
    app.on_key(press(KeyCode::Char('?')));
    let text = rendered(&mut app);
    assert!(
        text.contains("jump to a screen") && text.contains("this keymap"),
        "the keymap must be on screen, and it was not"
    );
}

#[test]
fn the_keymap_fits_in_a_small_terminal() {
    // **A keymap that is clipped is a keymap the player cannot read.**
    let mut app = app();
    app.on_key(press(KeyCode::Char('?')));
    let text = rendered_in(&mut app, 60, 24);
    assert!(
        text.contains("jump to a screen") && text.contains("quit"),
        "the whole keymap must fit in a small terminal, and it did not"
    );
}

#[test]
fn the_keymap_opens_on_the_function_key() {
    // **`F1` is the same physical key on every layout.** The terminal reports
    // it from the hardware rather than from a character table, so it cannot be
    // in a different place on someone else's keyboard.
    let mut app = app();
    app.on_key(press(KeyCode::F(1)));
    assert!(app.show_help(), "F1 opens the keymap");
    let text = rendered(&mut app);
    assert!(
        text.contains("jump to a screen"),
        "and the keymap must actually be drawn"
    );
}

#[test]
fn the_clock_starts_at_a_minute_a_second() {
    // **One second in life is one minute in game**, and there is nothing
    // slower: a campaign is thirty to sixty years, and a rate below that would
    // be a lifetime.
    let app = app();
    assert_eq!(app.clock().scale(), TimeScale::Sixty);
    assert_eq!(app.clock().scale().slower(), TimeScale::Sixty);
}

#[test]
fn a_key_is_acted_on_once_and_not_twice() {
    // **On Windows the terminal reports both the press and the release of every
    // key.** Acting on both makes every key fire twice, which is what a player
    // sees as "it jumps two steps" — and it makes the keymap flicker open and
    // shut again.
    let mut app = app();
    let before = app.clock().scale();
    app.on_key(press(KeyCode::Char('+')));
    assert_eq!(
        app.clock().scale(),
        before.faster(),
        "the press moves the clock one step"
    );
    app.on_key(release(KeyCode::Char('+')));
    assert_eq!(
        app.clock().scale(),
        before.faster(),
        "the release does nothing at all"
    );
}

#[test]
fn a_release_does_not_open_the_keymap_and_close_it_again() {
    let mut app = app();
    app.on_key(press(KeyCode::Char('?')));
    assert!(app.show_help(), "the press opens the keymap");
    app.on_key(release(KeyCode::Char('?')));
    assert!(app.show_help(), "the release must not close it again");
}

#[test]
fn a_slash_filters_and_a_shifted_slash_opens_the_keymap() {
    // **The keyboard layout is not the player's fault.** On a Nordic layout the
    // `?` is a shifted `/`, and a terminal that reports the unshifted
    // character with the shift modifier sends `/` with shift held.
    //
    // **The unshifted `/` filters the list and the shifted one opens the
    // keymap**, so the two never collide.
    let mut filtering = app();
    filtering.on_key(press(KeyCode::Char('/')));
    assert!(
        !filtering.show_help(),
        "an unshifted slash does not open the keymap"
    );
    assert!(
        filtering.list_control().is_typing(),
        "it filters the list instead"
    );

    let mut keymap = app();
    keymap.on_key(shifted(KeyCode::Char('/')));
    assert!(keymap.show_help(), "a shifted slash opens the keymap");
}

#[test]
fn the_key_ledger_is_on_screen_on_every_screen() {
    // **The most common keys are always on screen.** A player should not have
    // to remember the keymap, and should not have to open it to find the one
    // key they were told about.
    let mut app = app();
    for screen in 1..=7u32 {
        let digit = char::from_digit(screen, 10).expect("a digit");
        app.on_key(press(KeyCode::Char(digit)));
        let text = rendered(&mut app);
        assert!(
            text.contains("order")
                && text.contains("pause")
                && text.contains("speed")
                && text.contains("quit"),
            "the ledger must be on screen {screen}, and it was not"
        );
    }
}

/// The whole interface, as text.
fn rendered(app: &mut App) -> String {
    rendered_in(app, 100, 40)
}

/// The whole interface, as text, in a terminal of the given size.
fn rendered_in(app: &mut App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
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
fn a_release_does_not_issue_the_order_twice() {
    // The first press issues the order and the second is refused, because the
    // first already did it.
    let mut app = app();
    app.on_key(press(KeyCode::Char('o')));
    app.on_key(press(KeyCode::Enter));
    app.on_key(press(KeyCode::Enter));
    assert_eq!(
        app.sim().pending_orders().len(),
        1,
        "the order is in flight"
    );
    app.on_key(release(KeyCode::Enter));
    assert_eq!(
        app.sim().pending_orders().len(),
        1,
        "the release must not issue a second order"
    );
}
