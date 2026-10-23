//! Key handling.
//!
//! Keys that mean the same thing everywhere are global. Every other key belongs
//! to the screen that is showing, and a key that means something different on
//! two screens is a bug.
//!
//! The **clock controls are global**, because the clock belongs to the host and
//! not to any screen:
//!
//! | Key | Action |
//! |---|---|
//! | `Space` | Pause and resume the clock |
//! | `+` / `-` | One dilation step faster or slower |
//! | `Ctrl+1` … `Ctrl+9` | Set the dilation directly |
//! | `F1` | The keymap — the same key on every layout |
//! | `F2` | The manual |
//! | `F3` | Save the campaign |
//! | `F4` | Load the campaign |
//! | `?` or `/` | The keymap, if your layout has them |
//!
//! `Ctrl+digit` is used rather than `Shift+digit` because the shifted
//! character is layout-dependent, while the control modifier is not.
//!
//! See `docs/UI.md` §3.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use domain::time::TimeScale;

use crate::screen::Screen;

/// What the player asked the interface to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Do nothing; the key was not one we handle.
    None,
    /// Move to a screen.
    GoTo(Screen),
    /// Move to the next screen in the cycle.
    NextScreen,
    /// Move to the previous screen in the cycle.
    PreviousScreen,
    /// Move the selection up.
    Up,
    /// Move the selection down.
    Down,
    /// Move to the previous field.
    Left,
    /// Move to the next field.
    Right,
    /// Activate the selection.
    Accept,
    /// Go back, or close whatever is open.
    Back,
    /// Open the order palette.
    OpenPalette,
    /// Show the keymap for the current screen.
    Help,
    /// Open the manual, over everything.
    OpenManual,
    /// Open the manual restricted to the current screen.
    OpenInfo,
    /// Start filtering the list on the current screen.
    Filter,
    /// Move to the next sort key for the list on the current screen.
    Sort,
    /// Move the chronicle's filter on to the next year.
    FilterYear,
    /// Move the chronicle's filter on to the next category.
    FilterCategory,
    /// Leave the game.
    Quit,
    /// Show the next piece of advice.
    NextAdvice,
    /// Save the campaign.
    Save,
    /// Load the campaign.
    Load,
    /// Open the main menu.
    OpenMenu,
    /// Answer the decision the world has put on the table.
    OpenDecision,
    /// Stop or start the clock.
    TogglePause,
    /// One dilation step faster.
    Faster,
    /// One dilation step slower.
    Slower,
    /// Set the dilation directly.
    SetScale(TimeScale),
}

/// Interprets a key press.
///
/// The global keys are checked first, so a screen never has to reimplement them.
#[must_use]
pub fn action_for(key: KeyEvent, _screen: Screen) -> Action {
    // The clock controls, which are global.
    if let Some(action) = clock_action(key) {
        return action;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Action::Quit;
    }

    match key.code {
        KeyCode::Char('1') => screen_by_key(1),
        KeyCode::Char('2') => screen_by_key(2),
        KeyCode::Char('3') => screen_by_key(3),
        KeyCode::Char('4') => screen_by_key(4),
        KeyCode::Char('5') => screen_by_key(5),
        KeyCode::Char('6') => screen_by_key(6),
        KeyCode::Char('7') => screen_by_key(7),
        KeyCode::Tab => Action::NextScreen,
        KeyCode::BackTab => Action::PreviousScreen,
        KeyCode::Up => Action::Up,
        KeyCode::Down => Action::Down,
        KeyCode::Left => Action::Left,
        KeyCode::Right => Action::Right,
        KeyCode::Enter => Action::Accept,
        KeyCode::Esc => Action::Back,
        // **The keymap is on `F1` first.** A function key is the same physical
        // key on every keyboard layout — the terminal reports it from the
        // hardware, not from a character table — so it is the one key that
        // cannot be in a different place on someone else's keyboard.
        //
        // `?` is a shifted `/` on a Nordic layout, and a terminal may report
        // either the shifted character or the unshifted one with the shift
        // modifier. **A shifted `/` opens the keymap; an unshifted one filters
        // the list**, so the two never collide.
        KeyCode::F(1) | KeyCode::Char('?') => Action::Help,
        KeyCode::Char('/') if key.modifiers.contains(KeyModifiers::SHIFT) => Action::Help,
        // **The manual is on `F2`, and the screen's own information is on
        // `I`.** Both are function keys or plain letters, so both are the
        // same on every keyboard layout.
        KeyCode::F(2) => Action::OpenManual,
        KeyCode::Char('i' | 'I') => Action::OpenInfo,
        // **Saving and loading are on `F3` and `F4`,** so they are the same
        // physical key on every keyboard layout. **A player must be able to
        // save without having planned to** — the campaign they have spent an
        // evening on is not saved unless they say so, and they should not have
        // to restart the game to say it.
        KeyCode::F(3) => Action::Save,
        KeyCode::F(4) => Action::Load,
        // **The filter is on an unshifted `/`**, which is the key every editor
        // and every browser uses for it, and which is unshifted on the Nordic
        // layouts as well as the American ones.
        KeyCode::Char('/') => Action::Filter,
        // **The sort key is on `s`**, and it cycles rather than opening a
        // menu, because a menu to choose a sort is a menu to dismiss.
        KeyCode::Char('s' | 'S') => Action::Sort,
        // **The chronicle's filter axes.** They are on the chronicle screen
        // and nowhere else, so they are handled by the screen rather than
        // globally.
        KeyCode::Char('y' | 'Y') => Action::FilterYear,
        KeyCode::Char('c' | 'C') => Action::FilterCategory,
        // **The next piece of advice is on `n`,** for "next". It is a plain
        // letter, so it is on every layout, and it is a key the player reaches
        // for when they have read the banner and want another.
        KeyCode::Char('n' | 'N') => Action::NextAdvice,
        KeyCode::Char('o') => Action::OpenPalette,
        // **The main menu is on `m`,** for "menu". It is a plain letter, so it
        // is in the same place on every keyboard layout, and it is the key a
        // player reaches for when they want the things that are about the
        // campaign rather than about the screen they are on.
        KeyCode::Char('m' | 'M') => Action::OpenMenu,
        // **A decision is on `d`,** for "decide". It is a plain letter, so it
        // is the same key on every layout, and the world does not ask its
        // questions often enough to spend a function key on them.
        KeyCode::Char('d' | 'D') => Action::OpenDecision,
        KeyCode::Char('q') => Action::Quit,
        _ => Action::None,
    }
}

/// The clock controls, if the key is one.
fn clock_action(key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(character) = key.code {
            let digit = u8::try_from(character.to_digit(10)?).ok()?;
            return TimeScale::from_key(digit).map(Action::SetScale);
        }
        return None;
    }

    match key.code {
        KeyCode::Char(' ') => Some(Action::TogglePause),
        KeyCode::Char('+' | '=') => Some(Action::Faster),
        KeyCode::Char('-' | '_') => Some(Action::Slower),
        _ => None,
    }
}

/// The screen a number key selects.
fn screen_by_key(key: u8) -> Action {
    match Screen::from_key(key) {
        Some(screen) => Action::GoTo(screen),
        None => Action::None,
    }
}

/// Whether an action is one of the clock controls.
///
/// **The clock controls are global**, because the clock belongs to the host and
/// not to any screen — and a modal that swallowed them would take the player's
/// controls away exactly when they wanted to stop and think.
#[must_use]
pub fn is_clock_control(action: Action) -> bool {
    matches!(
        action,
        Action::TogglePause | Action::Faster | Action::Slower | Action::SetScale(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(character: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(character), KeyModifiers::CONTROL)
    }

    #[test]
    fn number_keys_jump_to_screens() {
        assert_eq!(
            action_for(press(KeyCode::Char('2')), Screen::Dashboard),
            Action::GoTo(Screen::Staff)
        );
    }

    #[test]
    fn tab_cycles_screens() {
        assert_eq!(
            action_for(press(KeyCode::Tab), Screen::Dashboard),
            Action::NextScreen
        );
        assert_eq!(
            action_for(press(KeyCode::BackTab), Screen::Dashboard),
            Action::PreviousScreen
        );
    }

    #[test]
    fn control_c_always_quits() {
        assert_eq!(action_for(ctrl('c'), Screen::Staff), Action::Quit);
    }

    #[test]
    fn space_toggles_the_pause() {
        assert_eq!(
            action_for(press(KeyCode::Char(' ')), Screen::Dashboard),
            Action::TogglePause
        );
    }

    #[test]
    fn plus_and_minus_step_the_dilation() {
        assert_eq!(
            action_for(press(KeyCode::Char('+')), Screen::Dashboard),
            Action::Faster
        );
        assert_eq!(
            action_for(press(KeyCode::Char('-')), Screen::Dashboard),
            Action::Slower
        );
    }

    #[test]
    fn control_digits_set_the_dilation() {
        assert_eq!(
            action_for(ctrl('1'), Screen::Dashboard),
            Action::SetScale(TimeScale::Sixty)
        );
        assert_eq!(
            action_for(ctrl('9'), Screen::Dashboard),
            Action::SetScale(TimeScale::SixHundredThousand)
        );
    }

    #[test]
    fn help_and_quit_are_global() {
        for screen in Screen::ALL {
            assert_eq!(action_for(press(KeyCode::Char('?')), screen), Action::Help);
            assert_eq!(action_for(press(KeyCode::Char('q')), screen), Action::Quit);
        }
    }

    #[test]
    fn the_clock_controls_work_on_every_screen() {
        for screen in Screen::ALL {
            assert_eq!(
                action_for(press(KeyCode::Char(' ')), screen),
                Action::TogglePause
            );
        }
    }
}
