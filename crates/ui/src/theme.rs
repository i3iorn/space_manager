//! Colour, glyphs, and spacing.
//!
//! **Colour is the second signal, never the only one.** Every colour-coded state
//! also carries a glyph, so the interface is fully usable in a monochrome
//! terminal, over SSH, or by a colour-blind player.
//!
//! See `docs/UI.md` §7.

use ratatui::style::{Color, Modifier, Style};

/// The meaning of a colour in the interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Meaning {
    /// Good, safe, complete.
    Good,
    /// Attention needed; at risk.
    Attention,
    /// Bad: failed, over budget, negative.
    Bad,
    /// Informational, or in the future.
    Informational,
    /// Historical, inactive, or dimmed.
    Historical,
}

impl Meaning {
    /// The colour for this meaning.
    #[must_use]
    pub const fn colour(self) -> Color {
        match self {
            Self::Good => Color::Green,
            Self::Attention => Color::Yellow,
            Self::Bad => Color::Red,
            Self::Informational => Color::Cyan,
            Self::Historical => Color::DarkGray,
        }
    }

    /// The glyph for this meaning.
    ///
    /// The glyph is what carries the meaning when colour is unavailable.
    #[must_use]
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Good => "✓",
            Self::Attention => "⚠",
            Self::Bad => "✗",
            Self::Informational => "○",
            Self::Historical => "—",
        }
    }

    /// The style for this meaning.
    #[must_use]
    pub fn style(self) -> Style {
        Style::default().fg(self.colour())
    }
}

/// The style for a headline or section title.
#[must_use]
pub fn title() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

/// The style for a value the player must not miss.
#[must_use]
pub fn emphasis() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

/// The style for a value that is committed to somebody else.
///
/// A commitment is a promise; a target is a plan. The typography carries the
/// distinction, so the two are never confused on the programme screen.
#[must_use]
pub fn committed() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

/// The style for a value that is only the agency's own plan.
#[must_use]
pub fn target() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// The style for text that has faded into the past.
#[must_use]
pub fn historical() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// The style for the row that has the focus.
///
/// **The focus must be impossible to miss.** A player who cannot see where
/// the selection is cannot use the selection, and a management game with three
/// hundred rows on a screen is unusable without it.
///
/// The reverse video is the signal, and it is the one that survives a
/// monochrome terminal. The marker beside the row is the second signal, so
/// the focus is findable even where the colours are gone.
#[must_use]
pub fn selected() -> Style {
    Style::default()
        .add_modifier(Modifier::REVERSED)
        .add_modifier(Modifier::BOLD)
}

/// The marker put beside the row that has the focus.
///
/// **The marker is a second signal, not the only one.** The theme's rule is
/// that colour is never the only thing carrying a meaning, and a focus that
/// can only be found by colour is a focus a monochrome player cannot find.
pub const FOCUS: &str = "▸";

/// The marker put beside every row that does not have the focus.
///
/// **It is a space, not nothing**, so the text of every row starts in the
/// same column and the list does not jump when the selection moves.
pub const NO_FOCUS: &str = " ";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_meaning_has_a_distinct_glyph() {
        let glyphs = [
            Meaning::Good.glyph(),
            Meaning::Attention.glyph(),
            Meaning::Bad.glyph(),
            Meaning::Informational.glyph(),
            Meaning::Historical.glyph(),
        ];
        let mut unique = glyphs.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), glyphs.len());
    }
}
