//! The tutorial.
//!
//! **The tutorial teaches the binding factor**, and it does it by pointing at
//! the thing that is already on the screen. The binding-factor widget is the
//! highest-risk component in the interface — it is the one panel the player has
//! to understand to understand the game — and a panel nobody looks at teaches
//! nothing.
//!
//! The hints are **contextual, not a sequence**. Each one belongs to a screen
//! and appears the first time the player is on that screen. A player who goes
//! straight to the research screen sees the research hint first, and the
//! tutorial does not insist on an order.
//!
//! Every hint is dismissed by seeing it, and the player can never see the same
//! one twice.

use crate::screen::Screen;

/// A line of guidance.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hint {
    /// The screen it belongs to. It appears when the player is on that screen.
    pub screen: Screen,
    /// What it says.
    pub text: &'static str,
}

/// The hints, in the order they are offered.
pub const HINTS: [Hint; 7] = [
    Hint {
        screen: Screen::Dashboard,
        text: "Three scientists, one project, and a budget that will not last. Press 2 to look at them.",
    },
    Hint {
        screen: Screen::Staff,
        text: "This is why your team is slow. The panel names the thing that is holding them back — and it is not always the same thing.",
    },
    Hint {
        screen: Screen::Dashboard,
        text: "Press o to give an order. You will see what it costs before you commit to it.",
    },
    Hint {
        screen: Screen::Research,
        text: "Your project advances at the rate your team can sustain. A tired team finishes late.",
    },
    Hint {
        screen: Screen::Dashboard,
        text: "Time passes whether you are ready or not. Press Space to pause the clock, and Ctrl+1 to Ctrl+9 to set how fast it runs.",
    },
    Hint {
        screen: Screen::Chronicle,
        text: "Everything that happens is recorded here. The agency keeps a history, and so does the state.",
    },
    Hint {
        screen: Screen::Branches,
        text: "You can fork the campaign at any decision you have made, and watch the two branches play out. Press Enter to fork at the selected instant.",
    },
];

/// The player's progress through the tutorial.
#[derive(Clone, Debug, Default)]
pub struct Tutorial {
    /// Which hints have been seen.
    seen: Vec<bool>,
}

impl Tutorial {
    /// A tutorial the player has not seen.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seen: vec![false; HINTS.len()],
        }
    }

    /// The hint to show on a screen, if there is one.
    ///
    /// **The hint stays until the player acts.** A hint that is marked as seen
    /// the first time it is asked for is on the screen for one frame - sixteen
    /// milliseconds - and a hint nobody read is not a hint. So the hint is
    /// marked as seen when the player *does* something, not when the screen is
    /// first drawn.
    #[must_use]
    pub fn peek(&self, screen: Screen) -> Option<&'static str> {
        for (index, hint) in HINTS.iter().enumerate() {
            if hint.screen == screen && !self.seen[index] {
                return Some(hint.text);
            }
        }
        None
    }

    /// Marks the hint on a screen as read.
    ///
    /// **The player has acted, so the hint has done its job.** Only the hint
    /// that is being shown is marked - a screen can have more than one, and
    /// the next one is shown on the next visit.
    pub fn dismiss(&mut self, screen: Screen) {
        for (index, hint) in HINTS.iter().enumerate() {
            if hint.screen == screen && !self.seen[index] {
                self.seen[index] = true;
                return;
            }
        }
    }

    /// Whether every hint has been seen.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.seen.iter().all(|seen| *seen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hint_stays_until_the_player_acts() {
        let mut tutorial = Tutorial::new();
        // **The hint is on the screen for as long as the player is looking at
        // it**, and not for a single frame.
        assert!(tutorial.peek(Screen::Staff).is_some(), "it is there");
        assert!(tutorial.peek(Screen::Staff).is_some(), "and it stays");
        tutorial.dismiss(Screen::Staff);
        assert!(tutorial.peek(Screen::Staff).is_none(), "until it is read");
    }

    #[test]
    fn a_hint_belongs_to_its_screen() {
        let tutorial = Tutorial::new();
        assert!(tutorial.peek(Screen::Staff).is_some());
        // The dashboard's hint is a different one, and is still waiting.
        assert!(tutorial.peek(Screen::Dashboard).is_some());
    }

    #[test]
    fn the_tutorial_finishes() {
        let mut tutorial = Tutorial::new();
        for hint in &HINTS {
            assert!(tutorial.peek(hint.screen).is_some());
            tutorial.dismiss(hint.screen);
        }
        assert!(tutorial.is_finished());
    }
}
