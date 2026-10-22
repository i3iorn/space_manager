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
//!
//! # The filter says more than a name
//!
//! **A name is not the question a player is asking.** "Who is in trouble" is
//! not a name, and a filter that can only search for names makes the player
//! read three hundred rows to answer it. So the filter takes **conditions** as
//! well as words:
//!
//! | Typed | Means |
//! |---|---|
//! | `okonjo` | anybody whose name or role contains "okonjo" |
//! | `fatigue>80` | anybody over eighty percent fatigue |
//! | `morale<-30` | anybody miserable enough to leave |
//! | `role:scientist` | anybody who is a scientist |
//! | `unassigned` | anybody on nothing |
//!
//! **The words and the conditions are combined.** A filter of `fatigue>80
//! unassigned` means "tired and on nothing", and every term has to hold.
//!
//! **A list says which fields it has.** The vocabulary belongs to the screen:
//! the staff list has fatigue and morale, the research list has progress and
//! team size, and a term that names a field the list does not have is a word
//! to search for rather than an error — so a filter typed on one screen does
//! not become a wall of nothing on another.

/// How a condition compares.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    /// `field:value` — the field is exactly this.
    Is,
    /// `field=value` — the field is exactly this.
    Equals,
    /// `field>value` — the field is more than this.
    Greater,
    /// `field>=value` — the field is at least this.
    AtLeast,
    /// `field<value` — the field is less than this.
    Less,
    /// `field<=value` — the field is at most this.
    AtMost,
}

impl Op {
    /// The operator as the player types it.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Is => ":",
            Self::Equals => "=",
            Self::Greater => ">",
            Self::AtLeast => ">=",
            Self::Less => "<",
            Self::AtMost => "<=",
        }
    }
}

/// What a field holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A number, compared with `>`, `>=`, `<`, `<=` or `=`.
    Number,
    /// A word, compared with `:` or `=`.
    Text,
    /// A flag: the term is the name on its own, and it holds when the row has
    /// the thing.
    Flag,
}

/// A field a list can be filtered by.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Field {
    /// The name the player types.
    pub name: &'static str,
    /// What the field holds.
    pub kind: Kind,
}

impl Field {
    /// A field holding a number.
    #[must_use]
    pub const fn number(name: &'static str) -> Self {
        Self {
            name,
            kind: Kind::Number,
        }
    }

    /// A field holding a word.
    #[must_use]
    pub const fn text(name: &'static str) -> Self {
        Self {
            name,
            kind: Kind::Text,
        }
    }

    /// A field that is either there or not.
    #[must_use]
    pub const fn flag(name: &'static str) -> Self {
        Self {
            name,
            kind: Kind::Flag,
        }
    }
}

/// What a row holds for a field.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FieldValue<'a> {
    /// A number.
    Number(i64),
    /// A word.
    Text(&'a str),
    /// A flag: the row has the thing, or it does not.
    Flag(bool),
}

/// One term of a filter.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Term {
    /// A word to look for.
    Text(String),
    /// A condition on one of the row's fields.
    Condition {
        /// The field's name.
        field: String,
        /// How it compares.
        op: Op,
        /// What it compares against.
        value: String,
    },
}

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

    /// The terms of the filter, read against the list's fields.
    ///
    /// **A term that is not a condition is a word to look for.** The list says
    /// which fields it has, and a term that names a field the list does not
    /// have is a word — so a filter typed on one screen does not become a wall
    /// of nothing on another.
    #[must_use]
    pub fn terms(&self, fields: &[Field]) -> Vec<Term> {
        self.filter
            .split_whitespace()
            .map(|word| Term::parse(word, fields))
            .collect()
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

impl Term {
    /// The field a condition names, if it is a condition.
    #[must_use]
    pub fn field(&self) -> Option<&str> {
        match self {
            Self::Text(_) => None,
            Self::Condition { field, .. } => Some(field),
        }
    }

    /// Whether the term is a word to look for.
    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Reads one word of a filter.
    fn parse(word: &str, fields: &[Field]) -> Self {
        let lower = word.to_lowercase();

        // **A flag is the name on its own.** "unassigned" means the row has
        // the thing, and there is nothing to compare it against.
        if let Some(field) = fields
            .iter()
            .find(|field| field.kind == Kind::Flag && field.name == lower)
        {
            return Self::Condition {
                field: field.name.to_owned(),
                op: Op::Equals,
                value: String::new(),
            };
        }

        // **Otherwise the word may be a condition.** The operator is the first
        // character that is not part of a field name, and the field is what
        // comes before it.
        for (index, character) in lower.char_indices() {
            let op = match character {
                ':' => Op::Is,
                '=' => Op::Equals,
                '>' => Op::Greater,
                '<' => Op::Less,
                _ => continue,
            };
            let name = &lower[..index];
            let field = fields
                .iter()
                .find(|field| field.name == name && field.kind != Kind::Flag);
            let Some(field) = field else {
                break;
            };
            // **"fatigue>=" is a number compared with "at least".** The
            // operator is two characters when it is one of the two-character
            // ones, and the value is what is left.
            let (op, rest) = if lower[index..].starts_with(">=") || lower[index..].starts_with("<=")
            {
                let op = if lower[index..].starts_with(">=") {
                    Op::AtLeast
                } else {
                    Op::AtMost
                };
                (op, &lower[index + 2..])
            } else {
                (op, &lower[index + 1..])
            };
            return Self::Condition {
                field: field.name.to_owned(),
                op,
                value: rest.to_owned(),
            };
        }

        Self::Text(lower)
    }

    /// Whether the term holds for a row.
    ///
    /// **A condition needs the row's value, and a word needs the row's text.**
    /// The texts are every word the row can be found by, and the value is what
    /// the row holds for the field the condition names.
    #[must_use]
    pub fn holds(&self, texts: &[&str], value: Option<&FieldValue<'_>>) -> bool {
        match self {
            Self::Text(word) => texts
                .iter()
                .any(|text| text.to_lowercase().contains(word.as_str())),
            Self::Condition {
                op, value: wanted, ..
            } => {
                let Some(value) = value else {
                    // **A condition on a field this row does not have does not
                    // hold.** A person on a programme has no fatigue, and a
                    // filter that says "fatigue over eighty" should not show
                    // them.
                    return false;
                };
                holds(*op, value, wanted)
            }
        }
    }
}

/// Whether a value satisfies a condition.
fn holds(op: Op, value: &FieldValue<'_>, wanted: &str) -> bool {
    match value {
        FieldValue::Number(number) => {
            let Some(wanted) = wanted.trim().parse::<i64>().ok() else {
                // **A number compared with a word is not a condition.** It does
                // not hold, and the player sees a shorter list rather than an
                // error they have to decode.
                return false;
            };
            match op {
                Op::Is | Op::Equals => *number == wanted,
                Op::Greater => *number > wanted,
                Op::AtLeast => *number >= wanted,
                Op::Less => *number < wanted,
                Op::AtMost => *number <= wanted,
            }
        }
        FieldValue::Text(text) => match op {
            Op::Is | Op::Equals => text.to_lowercase() == wanted.to_lowercase(),
            Op::Greater | Op::AtLeast => text.to_lowercase() > wanted.to_lowercase(),
            Op::Less | Op::AtMost => text.to_lowercase() < wanted.to_lowercase(),
        },
        FieldValue::Flag(has) => match op {
            // **A flag is either there or not.** "unassigned" is the flag with
            // nothing to compare against, and it holds when the row has the
            // thing. "unassigned:no" is the other way round.
            Op::Is | Op::Equals => {
                if wanted.is_empty() {
                    *has
                } else {
                    let wanted = wanted.to_lowercase();
                    let wanted = matches!(wanted.as_str(), "yes" | "true" | "1");
                    *has == wanted
                }
            }
            Op::Greater | Op::AtLeast => *has,
            Op::Less | Op::AtMost => !*has,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fields the staff list has.
    const STAFF: &[Field] = &[
        Field::number("fatigue"),
        Field::number("morale"),
        Field::text("role"),
        Field::flag("unassigned"),
    ];

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
        assert_eq!(control.sort(), 2);
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

    #[test]
    fn a_bare_word_is_a_text_search() {
        let mut control = ListControl::new();
        control.type_char('o');
        control.type_char('k');
        let terms = control.terms(STAFF);
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0], Term::Text("ok".to_owned()));
    }

    #[test]
    fn a_field_and_a_number_is_a_condition() {
        let mut control = ListControl::new();
        for character in "fatigue>80".chars() {
            control.type_char(character);
        }
        let terms = control.terms(STAFF);
        assert_eq!(
            terms[0],
            Term::Condition {
                field: "fatigue".to_owned(),
                op: Op::Greater,
                value: "80".to_owned(),
            }
        );
        assert!(terms[0].holds(&["Marsha Okonjo"], Some(&FieldValue::Number(85)),));
        assert!(!terms[0].holds(&["Marsha Okonjo"], Some(&FieldValue::Number(70)),));
    }

    #[test]
    fn a_field_and_a_word_is_a_condition() {
        let mut control = ListControl::new();
        for character in "role:scientist".chars() {
            control.type_char(character);
        }
        let terms = control.terms(STAFF);
        assert_eq!(
            terms[0],
            Term::Condition {
                field: "role".to_owned(),
                op: Op::Is,
                value: "scientist".to_owned(),
            }
        );
        assert!(terms[0].holds(&["Okonjo"], Some(&FieldValue::Text("Scientist"))));
        assert!(!terms[0].holds(&["Okonjo"], Some(&FieldValue::Text("Engineer"))));
    }

    #[test]
    fn a_flag_is_the_name_on_its_own() {
        let mut control = ListControl::new();
        for character in "unassigned".chars() {
            control.type_char(character);
        }
        let terms = control.terms(STAFF);
        assert!(terms[0].holds(&["Okonjo"], Some(&FieldValue::Flag(true))));
        assert!(!terms[0].holds(&["Okonjo"], Some(&FieldValue::Flag(false))));
    }

    #[test]
    fn a_field_the_list_does_not_have_is_a_word() {
        // **A filter typed on one screen must not become a wall of nothing on
        // another.** The programme list has no fatigue, so "fatigue>80" there
        // is a search for that text and not a condition.
        let mut control = ListControl::new();
        for character in "fatigue>80".chars() {
            control.type_char(character);
        }
        let terms = control.terms(&[]);
        assert_eq!(terms[0], Term::Text("fatigue>80".to_owned()));
    }

    #[test]
    fn a_condition_on_a_field_a_row_does_not_have_does_not_hold() {
        let mut control = ListControl::new();
        for character in "fatigue>80".chars() {
            control.type_char(character);
        }
        let terms = control.terms(STAFF);
        assert!(!terms[0].holds(&["A programme"], None));
    }

    #[test]
    fn a_number_compared_with_a_word_does_not_hold() {
        let mut control = ListControl::new();
        for character in "fatigue>high".chars() {
            control.type_char(character);
        }
        let terms = control.terms(STAFF);
        assert!(!terms[0].holds(&["Okonjo"], Some(&FieldValue::Number(90))));
    }

    #[test]
    fn every_term_has_to_hold() {
        let mut control = ListControl::new();
        for character in "fatigue>80 unassigned".chars() {
            control.type_char(character);
        }
        let terms = control.terms(STAFF);
        assert_eq!(terms.len(), 2);
    }
}
