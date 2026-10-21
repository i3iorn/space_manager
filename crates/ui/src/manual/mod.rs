//! The manual.
//!
//! **Everything the game knows how to explain, in one place.** The player is
//! managing an agency that runs on a vocabulary — goodwill, slack, the binding
//! factor, appropriation — and none of it is obvious from looking at it. A
//! number the player does not understand is a number they cannot make a
//! decision about.
//!
//! **The content is a store, not a set of screens.** A topic is written once
//! and reached three ways:
//!
//! - the **manual** (`F2`), browsable and filterable, from anywhere;
//! - the **screen's own information** (`I`), which is the manual filtered to
//!   the screen the player is looking at;
//! - the **info line**, which shows the summary of whatever is selected.
//!
//! **One store means one answer.** If the binding factor is explained in the
//! staff screen's information and in the manual, the two will disagree within
//! a month. There is one binding-factor topic and both routes reach it.

use crate::screen::Screen;

pub mod agency;
pub mod interface;
pub mod orders;
pub mod programmes;
pub mod research;
pub mod screens;
pub mod state;
pub mod world;

/// Defines a topic.
///
/// The bodies are long, and a struct literal repeated seventy times is a wall
/// of punctuation that hides the prose.
macro_rules! topic {
    ($id:literal, $section:expr, $title:literal, $summary:literal, $body:literal) => {
        $crate::manual::Topic {
            id: $id,
            section: $section,
            title: $title,
            summary: $summary,
            body: $body,
        }
    };
}
pub(crate) use topic;

/// A section of the manual.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Section {
    /// How to use the game.
    Interface,
    /// The people.
    Agency,
    /// The money and the state.
    State,
    /// Research, testing, and facilities.
    Research,
    /// Programmes, objectives, and promises.
    Programmes,
    /// The world the agency works in.
    World,
    /// The screens, and what is on them.
    Screens,
    /// The orders, and what they do.
    Orders,
}

impl Section {
    /// Every section, in the order they are read.
    pub const ALL: [Self; 8] = [
        Self::Interface,
        Self::Agency,
        Self::State,
        Self::Research,
        Self::Programmes,
        Self::Orders,
        Self::World,
        Self::Screens,
    ];

    /// The section's heading.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Interface => "Using the agency",
            Self::Agency => "The people",
            Self::State => "Money and the state",
            Self::Research => "Research and facilities",
            Self::Programmes => "Programmes and promises",
            Self::Orders => "The orders",
            Self::World => "The world",
            Self::Screens => "The screens",
        }
    }
}

/// A topic: one thing the game can explain.
#[derive(Clone, Copy, Debug)]
pub struct Topic {
    /// A stable identifier. Screens refer to topics by this.
    pub id: &'static str,
    /// The section it belongs to.
    pub section: Section,
    /// What it is called.
    pub title: &'static str,
    /// One line, for the info line and for the list.
    pub summary: &'static str,
    /// The full explanation.
    pub body: &'static str,
}

/// Every topic, in the order they are read.
pub fn all() -> impl Iterator<Item = &'static Topic> {
    interface::TOPICS
        .iter()
        .chain(agency::TOPICS.iter())
        .chain(state::TOPICS.iter())
        .chain(research::TOPICS.iter())
        .chain(programmes::TOPICS.iter())
        .chain(orders::TOPICS.iter())
        .chain(world::TOPICS.iter())
        .chain(screens::TOPICS.iter())
}

/// The topic with the given id.
#[must_use]
pub fn find(id: &str) -> Option<&'static Topic> {
    all().find(|topic| topic.id == id)
}

/// The summary of the topic with the given id, if there is one.
///
/// **This is what the info line shows**, and it is asked for on every frame,
/// so it stays a lookup and not a copy.
#[must_use]
pub fn summary(id: &str) -> Option<&'static str> {
    find(id).map(|topic| topic.summary)
}

/// The player's place in the manual.
///
/// **The manual is a browser, not a stack of pages.** The player filters it,
/// moves through what is left, and reads. Everything the manual can say is
/// reachable from one place, and the filter is what makes that bearable.
/// A row in the manual's list.
///
/// **The list is grouped by section**, and the headings are rows of their own.
/// That makes the row count different from the topic count, so the scroll
/// position is counted in rows and not in topics - otherwise the scrolling
/// drifts every time a heading appears or disappears.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Row {
    /// The heading of a section.
    Heading(Section),
    /// A topic, by its index in the visible list.
    Topic(usize),
}

/// The player's place in the manual.
#[derive(Clone, Debug, Default)]
pub struct Manual {
    /// What the player has typed to narrow it.
    filter: String,
    /// The index of the selected topic, among the visible ones.
    selected: usize,
    /// The first row on show, so the list scrolls.
    offset: usize,
    /// The screen the manual is restricted to, if any.
    only: Option<Screen>,
    /// How far the explanation has been scrolled.
    ///
    /// **The longest explanation is over a thousand characters**, and a detail
    /// pane in a small terminal cannot show it all. The player scrolls.
    scroll: usize,
    /// The topics on show, cached because the list is rebuilt on every frame.
    visible: Vec<&'static Topic>,
    /// The rows on show, cached for the same reason.
    rows: Vec<Row>,
}

impl Manual {
    /// A manual with nothing typed and nothing selected.
    #[must_use]
    pub fn new() -> Self {
        let mut manual = Self::default();
        manual.rebuild();
        manual
    }

    /// The rows of the list, including the section headings.
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The first row on show.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// How far the explanation has been scrolled.
    #[must_use]
    pub const fn scroll(&self) -> usize {
        self.scroll
    }

    /// Sets how far the explanation has been scrolled.
    pub fn set_scroll(&mut self, scroll: usize) {
        self.scroll = scroll;
    }

    /// Scrolls the explanation down.
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_add(lines);
    }

    /// Scrolls the explanation up.
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_sub(lines);
    }

    /// Puts the explanation back at the top.
    ///
    /// **A new topic starts at the top**, because a player who scrolls to the
    /// bottom of one explanation and then moves to the next should not land in
    /// the middle of it.
    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    /// Scrolls the list so that the selection is on screen.
    ///
    /// **The list is longer than the screen**, so it has to scroll, and the
    /// selection must never be the one thing that is off it.
    pub fn ensure_visible(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        let Some(row) = self.row_of_selection() else {
            self.offset = 0;
            return;
        };
        if row < self.offset {
            self.offset = row;
        } else if row >= self.offset + height {
            self.offset = row + 1 - height;
        }
        let max_offset = self.rows.len().saturating_sub(height);
        self.offset = self.offset.min(max_offset);
    }

    /// The row the selected topic is on.
    fn row_of_selection(&self) -> Option<usize> {
        self.rows
            .iter()
            .position(|row| matches!(row, Row::Topic(index) if *index == self.selected))
    }

    /// The topics on show.
    #[must_use]
    pub fn visible(&self) -> &[&'static Topic] {
        &self.visible
    }

    /// The selected topic.
    #[must_use]
    pub fn selected(&self) -> Option<&'static Topic> {
        self.visible.get(self.selected).copied()
    }

    /// The index of the selected topic.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.selected
    }

    /// What the player has typed to narrow the manual.
    #[must_use]
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// The screen the manual is restricted to, if any.
    #[must_use]
    pub const fn restricted_to(&self) -> Option<Screen> {
        self.only
    }

    /// Restricts the manual to a screen, or lets it range over everything.
    pub fn restrict(&mut self, screen: Option<Screen>) {
        self.only = screen;
        self.selected = 0;
        self.reset_scroll();
        self.rebuild();
    }

    /// Narrows the manual by what the player has typed.
    ///
    /// **The filter matches the title, the summary, and the body**, so a player
    /// who remembers that a thing is called "slack" finds it, and a player who
    /// only remembers that it is about "time" finds it too.
    pub fn filter_by(&mut self, text: &str) {
        self.filter = text.to_lowercase();
        self.selected = 0;
        self.reset_scroll();
        self.rebuild();
    }

    /// Types a character into the filter.
    pub fn type_char(&mut self, character: char) {
        let lowered = character.to_lowercase().next().unwrap_or(character);
        self.filter.push(lowered);
        self.selected = 0;
        self.reset_scroll();
        self.rebuild();
    }

    /// Takes the last character off the filter.
    pub fn backspace(&mut self) {
        self.filter.pop();
        self.selected = 0;
        self.reset_scroll();
        self.rebuild();
    }

    /// Moves the selection down.
    pub fn down(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.visible.len();
        self.reset_scroll();
    }

    /// Moves the selection up.
    pub fn up(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        self.selected = (self.selected + self.visible.len() - 1) % self.visible.len();
        self.reset_scroll();
    }

    /// Rebuilds the visible list.
    ///
    /// **A filter that matches nothing leaves the list empty, and the manual
    /// says so.** An empty screen with no explanation is a bug the player
    /// cannot diagnose.
    fn rebuild(&mut self) {
        let needle = self.filter.trim();
        let mut matched: Vec<(&'static Topic, u8)> = all()
            .filter(|topic| {
                self.only
                    .is_none_or(|screen| screens::topics_for(screen).contains(&topic.id))
            })
            .filter_map(|topic| {
                if needle.is_empty() {
                    return Some((topic, 0));
                }
                // **A topic named by the search comes first.** A player who
                // types "binding" wants the binding factor, not the first
                // topic that happens to mention it in its body.
                let title = topic.title.to_lowercase();
                if title.contains(needle) {
                    Some((topic, 0))
                } else if topic.summary.to_lowercase().contains(needle) {
                    Some((topic, 1))
                } else if topic.body.to_lowercase().contains(needle) {
                    Some((topic, 2))
                } else {
                    None
                }
            })
            .collect();

        // **The sort is stable**, so the order inside a rank is the order the
        // topics are written in.
        matched.sort_by_key(|(_, rank)| *rank);

        self.visible = matched.iter().map(|(topic, _)| *topic).collect();
        if self.selected >= self.visible.len() {
            self.selected = self.visible.len().saturating_sub(1);
        }

        // **The rows are the topics with the section headings woven in**, and
        // they are rebuilt with them so the two can never disagree.
        self.rows.clear();
        let mut last: Option<Section> = None;
        for (index, topic) in self.visible.iter().enumerate() {
            if last != Some(topic.section) {
                self.rows.push(Row::Heading(topic.section));
                last = Some(topic.section);
            }
            self.rows.push(Row::Topic(index));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_topic_has_a_summary_and_a_body() {
        for topic in all() {
            assert!(!topic.summary.is_empty(), "{} has no summary", topic.id);
            assert!(!topic.body.is_empty(), "{} has no body", topic.id);
            assert!(
                topic.title.chars().count() <= 40,
                "{} has an unwieldy title",
                topic.id
            );
        }
    }

    #[test]
    fn the_topic_ids_are_unique() {
        let mut ids: Vec<&str> = all().map(|topic| topic.id).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "two topics share an id");
    }

    #[test]
    fn a_topic_can_be_found_by_its_id() {
        let topic = find("agency.morale").expect("morale is explained");
        assert_eq!(topic.title, "Morale");
    }

    #[test]
    fn the_filter_narrows_the_list() {
        let mut manual = Manual::new();
        let before = manual.visible().len();
        manual.filter_by("slack");
        assert!(manual.visible().len() < before, "the filter narrows it");
        assert!(
            manual
                .visible()
                .iter()
                .any(|topic| topic.id == "programme.slack"),
            "and it finds the right topic"
        );
    }

    #[test]
    fn a_filter_that_matches_nothing_leaves_the_list_empty() {
        let mut manual = Manual::new();
        manual.filter_by("zzzznotathing");
        assert!(manual.visible().is_empty());
        assert!(manual.selected().is_none());
    }

    #[test]
    fn the_selection_moves_and_wraps() {
        let mut manual = Manual::new();
        manual.down();
        assert_eq!(manual.index(), 1);
        manual.up();
        assert_eq!(manual.index(), 0);
        manual.up();
        assert_eq!(
            manual.index(),
            manual.visible().len() - 1,
            "up from the top wraps to the bottom"
        );
    }

    #[test]
    fn typing_and_deleting_narrows_and_widens() {
        let mut manual = Manual::new();
        let before = manual.visible().len();
        for character in "morale".chars() {
            manual.type_char(character);
        }
        assert!(manual.visible().len() < before);
        for _ in 0.."morale".chars().count() {
            manual.backspace();
        }
        assert_eq!(manual.visible().len(), before);
    }
}
