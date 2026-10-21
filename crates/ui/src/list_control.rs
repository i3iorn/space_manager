//! The player's control over a list.
//!
//! **Every list in the game is filterable and sortable.** An agency with three
//! hundred people has a staff list that cannot be read top to bottom, and a
//! player who has to scroll to find the one person who is about to resign has
//! a screen that does not do its job.
//!
//! The filter is entered explicitly, by pressing `/`, and it is a **mode
//! rather than a side effect**: while the player is typing, the keys go into
//! the filter and nowhere else, so a filter can contain a `q` without the
//! game quitting.

/// The player's control over a list: what it is filtered by, and how it is
/// sorted.
#[derive(Clone, Debug, Default)]
pub struct ListControl {
    /// The text the player has typed to narrow the list.
    filter: String,
    /// Whether the player is typing into the filter.
    typing: bool,
    /// The index of the sort key.
    sort: usize,
}

impl ListControl {
    /// A control with no filter and the first sort key.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            filter: String::new(),
            typing: false,
            sort: 0,
        }
    }

    /// The filter text.
    #[must_use]
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Whether the player is typing into the filter.
    #[must_use]
    pub const fn is_typing(&self) -> bool {
        self.typing
    }

    /// Starts typing into the filter.
    pub fn start_typing(&mut self) {
        self.typing = true;
    }

    /// Stops typing, keeping the filter.
    pub fn stop_typing(&mut self) {
        self.typing = false;
    }

    /// Types a character into the filter.
    pub fn type_char(&mut self, character: char) {
        self.filter
            .push(character.to_lowercase().next().unwrap_or(character));
    }

    /// Takes the last character off the filter.
    pub fn backspace(&mut self) {
        self.filter.pop();
    }

    /// Empties the filter.
    pub fn clear(&mut self) {
        self.filter.clear();
    }

    /// Whether a piece of text passes the filter.
    ///
    /// **The filter matches anywhere in the text**, because a player looking
    /// for "Okonjo" should find her whatever else is on the line.
    #[must_use]
    pub fn matches(&self, text: &str) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        text.to_lowercase().contains(&self.filter)
    }

    /// The index of the sort key.
    #[must_use]
    pub const fn sort(&self) -> usize {
        self.sort
    }

    /// Moves to the next sort key.
    pub fn next_sort(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.sort = (self.sort + 1) % count;
    }

    /// The name of the current sort key.
    #[must_use]
    pub fn sort_name(&self, names: &[&'static str]) -> &'static str {
        names.get(self.sort).copied().unwrap_or("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_filter_matches_everything() {
        let control = ListControl::new();
        assert!(control.matches("Marsha Okonjo"));
        assert!(control.matches(""));
    }

    #[test]
    fn the_filter_matches_anywhere_in_the_text() {
        let mut control = ListControl::new();
        control.type_char('o');
        control.type_char('k');
        assert!(control.matches("Marsha Okonjo"));
        assert!(!control.matches("Viktor Sørensen"));
    }

    #[test]
    fn the_filter_is_case_insensitive() {
        let mut control = ListControl::new();
        control.type_char('O');
        control.type_char('K');
        assert!(control.matches("Marsha Okonjo"));
    }

    #[test]
    fn the_sort_key_cycles() {
        let mut control = ListControl::new();
        assert_eq!(control.sort(), 0);
        control.next_sort(3);
        assert_eq!(control.sort(), 1);
        control.next_sort(3);
        control.next_sort(3);
        assert_eq!(control.sort(), 0, "and it wraps");
    }

    #[test]
    fn typing_is_a_mode() {
        let mut control = ListControl::new();
        assert!(!control.is_typing());
        control.start_typing();
        assert!(control.is_typing());
        control.stop_typing();
        assert!(!control.is_typing());
    }
}
