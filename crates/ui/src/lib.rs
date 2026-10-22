//! The terminal interface for `space_manager`.
//!
//! The interface is a **view** over the simulation. It reads a snapshot of the
//! campaign and issues orders; it never mutates simulation state.
//!
//! # Rules for this crate
//!
//! * **The simulation never imports this crate.** The dependency points one way:
//!   `ui` → `sim` → `domain`.
//! * **No `unwrap`, no `expect`, no `panic!`** outside tests. A panic here loses
//!   the campaign.
//! * **The terminal must always be restored**, even on panic. The [`TerminalGuard`]
//!   does this on drop.
//!
//! # Where to look
//!
//! | Module | Holds |
//! |---|---|
//! | [`advice`] | What to do next, read from the campaign |
//! | [`app`] | The application state and the render pass |
//! | [`briefing`] | The opening briefing: the framing, and the first goal |
//! | [`screen`] | The screens and their fixed number keys |
//! | [`input`] | Key handling, global and per-screen |
//! | [`list_control`] | Filtering and sorting the lists |
//! | [`menu`] | The main menu: restart, save, load, settings |
//! | [`theme`] | Colour, glyphs, and the meaning of each |
//! | [`manual`] | Everything the game can explain, and the browser over it |
//! | [`screens`] | One module per screen |
//! | [`widgets`] | The reusable widgets |
//!
//! The interface design is `docs/UI.md`.

#![deny(missing_docs)]

pub mod advice;
pub mod app;
pub mod briefing;
pub mod clock;
pub mod confirm;
pub mod input;
pub mod list_control;
pub mod manual;
pub mod menu;
pub mod order_palette;
pub mod screen;
pub mod screens;
pub mod theme;
pub mod tutorial;
pub mod widgets;

use std::io::{self, Stdout};

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use thiserror::Error;

pub use app::{App, DEFAULT_SAVE_PATH};
pub use screen::Screen;

/// Something went wrong in the interface.
#[derive(Debug, Error)]
pub enum UiError {
    /// The terminal could not be put into the mode the interface needs.
    #[error("The terminal could not be prepared: {0}")]
    Terminal(#[from] io::Error),
}

/// Restores the terminal when it goes out of scope.
///
/// The guard is the reason a panic in a widget cannot leave the player with a
/// broken terminal. The `Drop` implementation runs on unwind, so the alternate
/// screen is always left and raw mode is always disabled.
///
/// It also holds the panic hook the process had before the game started, and
/// puts it back when the game ends. **A library must not change the host
/// process's panic hook permanently**, and the game is a library first and a
/// program second.
struct TerminalGuard {
    /// The hook that was in place before the game started.
    previous_hook: Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send + 'static>,
}

impl TerminalGuard {
    /// Puts the terminal into raw mode on the alternate screen.
    ///
    /// **The two steps are undone together.** If the alternate screen cannot
    /// be entered, the raw mode that was already enabled is turned off again
    /// before the error goes anywhere else, because a terminal left in raw
    /// mode is a terminal the player cannot type in — and the player has no
    /// way to know that the game did it.
    fn enter() -> Result<Self, UiError> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        if let Err(error) = execute!(out, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(UiError::Terminal(error));
        }
        Ok(Self {
            previous_hook: install_panic_hook(),
        })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = execute!(out, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        std::panic::set_hook(std::mem::replace(&mut self.previous_hook, Box::new(|_| {})));
    }
}

/// Installs a panic hook that leaves the terminal usable and says what happened.
///
/// **A panic message written to the alternate screen is a message the player
/// never sees.** The alternate screen is thrown away when it is left, so a
/// panic that unwinds out of the interface takes its own explanation with it:
/// the terminal comes back, the game is gone, and nothing says why. That is
/// the "it just shut down" the player is describing.
///
/// The hook restores the terminal *first* and writes the message *after*, so
/// the player gets a working terminal and the reason in one piece.
///
/// The hook that was in place is returned rather than chained: the default
/// hook's output is reproduced here, and a library that chains hooks has to
/// get the lifetimes of `PanicHookInfo` exactly right, which is not worth the
/// trouble for a message the player reads once.
fn install_panic_hook() -> Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send + 'static> {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut out = io::stdout();
        let _ = execute!(out, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        eprintln!();
        eprintln!("The game stopped unexpectedly. It may have been a bug.");
        if let Some(location) = info.location() {
            eprintln!("  at {}:{}", location.file(), location.line());
        }
        eprintln!("  {info}");
        eprintln!();
    }));
    previous
}

/// The type of the terminal the interface draws to.
type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Runs the interface until the player quits.
///
/// The loop is **not** a turn loop. It polls for events with a short timeout so
/// that the clock keeps running whether or not the player is touching anything,
/// and it hands the simulation a span of *simulated* time each frame. The
/// simulation is never told how fast it is running.
///
/// **The application is borrowed and not consumed**, so the caller can save
/// the campaign when the game ends.
///
/// # Errors
///
/// Returns a [`UiError`] if the terminal cannot be prepared or drawn to. The
/// terminal is restored either way.
pub fn run(app: &mut App) -> Result<(), UiError> {
    let _guard = TerminalGuard::enter()?;
    let mut terminal: Tui = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    while !app.should_quit() {
        // How much simulated time has passed since the last frame.
        let delta = app.clock_mut().tick();
        app.advance(delta);

        terminal.draw(|frame| app.render(frame))?;

        // Poll rather than block, so the frame rate stays bounded and the clock
        // keeps running while nobody is pressing anything.
        if event::poll(std::time::Duration::from_millis(clock::FRAME_MILLIS))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panic_does_not_take_the_terminal_with_it() {
        // **The hook runs before the unwind**, so it can put the terminal
        // back while the stack is still coming down. A hook that panicked
        // would abort the process and leave the alternate screen up, which
        // is the very thing this mechanism exists to prevent — so the test
        // is that the hook survives being fired.
        let previous = install_panic_hook();
        let outcome = std::panic::catch_unwind(|| {
            panic!("the test's own panic");
        });
        assert!(outcome.is_err(), "the panic happened");
        // The hook we installed is the one in place, and putting the old one
        // back is how the game leaves the process as it found it.
        std::panic::set_hook(previous);
    }
}
