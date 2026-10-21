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
//! | [`app`] | The application state and the render pass |
//! | [`screen`] | The screens and their fixed number keys |
//! | [`input`] | Key handling, global and per-screen |
//! | [`list_control`] | Filtering and sorting the lists |
//! | [`theme`] | Colour, glyphs, and the meaning of each |
//! | [`manual`] | Everything the game can explain, and the browser over it |
//! | [`screens`] | One module per screen |
//! | [`widgets`] | The reusable widgets |
//!
//! The interface design is `docs/UI.md`.

#![deny(missing_docs)]

pub mod app;
pub mod clock;
pub mod input;
pub mod list_control;
pub mod manual;
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

pub use app::App;
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
#[derive(Debug)]
struct TerminalGuard;

impl TerminalGuard {
    /// Puts the terminal into raw mode on the alternate screen.
    fn enter() -> Result<Self, UiError> {
        enable_raw_mode()?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = execute!(out, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
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
/// # Errors
///
/// Returns a [`UiError`] if the terminal cannot be prepared or drawn to. The
/// terminal is restored either way.
pub fn run(mut app: App) -> Result<(), UiError> {
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
