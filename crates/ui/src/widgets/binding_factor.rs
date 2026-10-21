//! The performance explanation panel.
//!
//! **The most important file in the interface.** It shows the effective
//! performance, the modifier breakdown, and the *binding factor* — the single
//! constraint costing the most, named in words. If the performance model cannot
//! be explained here, the fix is to the model, not to the widget.
//!
//! # What it must do
//!
//! 1. Show the effective performance as a percentage.
//! 2. Show the modifier breakdown, each as a bar and a signed number.
//! 3. **Name the binding factor in words** — the single largest cost.
//! 4. Never show a number without a name.
//!
//! # What it must never do
//!
//! * Show the raw formula. A player should not be able to reconstruct the
//!   arithmetic.
//! * Show one factor alone. "Held back by fatigue" without the rest of the
//!   breakdown is not an explanation; the player needs to see what else is
//!   contributing, or they cannot weigh whether to fix fatigue *or* tools.
//!
//! See `docs/UI.md` §6.

use domain::performance::{Modifier, ModifierKind, PerformanceResult};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier as StyleModifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::theme::{self, Meaning};

/// The width of the modifier bars, in cells.
const BAR_WIDTH: usize = 10;

/// The performance explanation panel.
pub struct BindingFactor<'a> {
    /// The result to explain.
    pub result: &'a PerformanceResult,
}

impl<'a> BindingFactor<'a> {
    /// A panel explaining a performance result.
    #[must_use]
    pub const fn new(result: &'a PerformanceResult) -> Self {
        Self { result }
    }

    /// Draws the panel.
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        frame.render_widget(self.headline(), rows[0]);
        frame.render_widget(self.breakdown(), rows[1]);
    }

    /// The performance figure and the constraint, named.
    fn headline(&self) -> Paragraph<'a> {
        let percent = self.result.percent();
        let constraint = self.result.constraint_sentence();

        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled(
                format!("{percent:>3}% of standard"),
                Style::default().add_modifier(StyleModifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled(constraint.clone(), theme::Meaning::style_for(&constraint)),
        ]));
        lines.push(Line::from(""));

        Paragraph::new(lines).wrap(Wrap { trim: false })
    }

    /// The modifier breakdown.
    fn breakdown(&self) -> Paragraph<'a> {
        let mut lines = Vec::new();
        for modifier in &self.result.modifiers {
            lines.push(modifier_line(*modifier));
        }

        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Performance"))
            .wrap(Wrap { trim: false })
    }
}

/// One modifier, as a label, a bar, and a signed number.
fn modifier_line(modifier: Modifier) -> Line<'static> {
    let value = modifier.value();
    let magnitude = usize::from(value.unsigned_abs());
    let filled = magnitude * BAR_WIDTH / 100;
    let bar: String = "▇".repeat(filled) + &"░".repeat(BAR_WIDTH - filled);

    let meaning = match value.cmp(&0) {
        std::cmp::Ordering::Less => Meaning::Bad,
        std::cmp::Ordering::Greater => Meaning::Good,
        std::cmp::Ordering::Equal => Meaning::Historical,
    };

    Line::from(vec![
        Span::raw(format!("{:<12}", modifier.kind().label())),
        Span::styled(bar, meaning.style()),
        Span::raw("  "),
        Span::styled(format!("{value:>+4}"), meaning.style()),
    ])
}

/// The meaning of a constraint sentence, for colouring.
trait ConstraintMeaning {
    /// The meaning that fits the sentence.
    fn style_for(sentence: &str) -> Style;
}

impl ConstraintMeaning for Meaning {
    fn style_for(sentence: &str) -> Style {
        if sentence.starts_with("No significant") {
            Meaning::Good.style()
        } else {
            Meaning::Attention.style()
        }
    }
}

/// The name of the constraint, for the interface.
#[must_use]
pub fn constraint_name(kind: Option<ModifierKind>) -> String {
    match kind {
        Some(kind) => kind.label().to_owned(),
        None => "NONE".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::performance::{Modifier, PerformanceInput, PerformanceResult};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn result() -> PerformanceResult {
        let mut input = PerformanceInput::neutral();
        input.skill = Modifier::new(ModifierKind::Skill, 18);
        input.fatigue = Modifier::new(ModifierKind::Fatigue, -31);
        sim::performance::compute(&input)
    }

    #[test]
    fn the_panel_renders_without_panicking() {
        let result = result();
        let backend = TestBackend::new(40, 14);
        let mut terminal = Terminal::new(backend).expect("the terminal starts");
        terminal
            .draw(|frame| BindingFactor::new(&result).render(frame, frame.area()))
            .expect("the panel draws");
    }

    #[test]
    fn the_constraint_is_named_in_words() {
        let result = result();
        assert!(result.constraint_sentence().contains("FATIGUE"));
    }

    #[test]
    fn a_healthy_result_says_there_is_no_constraint() {
        let input = PerformanceInput::neutral();
        let result = sim::performance::compute(&input);
        assert_eq!(result.constraint_sentence(), "No significant constraint");
    }
}
