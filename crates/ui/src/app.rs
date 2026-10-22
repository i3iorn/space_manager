//! The application state and the render pass.
//!
//! The app owns the simulation, the current screen, and the key handling. It
//! holds a **snapshot** of the campaign, never a live reference into it, so the
//! interface can never hold a borrow across a mutation.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use domain::report::MonthReport;
use domain::time::Duration;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use sim::Sim;

use crate::clock::Clock;
use crate::input::{Action, action_for, is_clock_control};
use crate::list_control;
use crate::manual;
use crate::order_palette;
use crate::screen::Screen;
use crate::screens;
use crate::theme;
use crate::tutorial;

/// Where the campaign is saved when the player has not said otherwise.
///
/// **A save the player cannot find is a save they will not trust**, so the
/// file is named and the notice says where it went.
pub const DEFAULT_SAVE_PATH: &str = "campaign.json";

/// How long the game waits between automatic saves.
///
/// **Ten seconds of real time.** At the base rate a month boundary arrives
/// every half hour, so the throttle never bites; at the top dilation it
/// arrives several times a second, and the throttle is what keeps the game
/// from writing the whole campaign five times a second.
const AUTOSAVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);

/// Writes a campaign to a file, making the directory if it is not there.
///
/// **The game's files live in the platform's configuration directory**, and
/// that directory does not exist until the game has been run once — so the
/// first save has to make it.
fn write_campaign(path: &str, text: &str) -> std::io::Result<()> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, text)
}

/// The application state.
#[derive(Debug)]
pub struct App {
    sim: Sim,
    /// The forked branch, if the player has made one.
    ///
    /// **The branch is a second campaign**, and it advances with the clock like
    /// the original does — otherwise the comparison would be between two
    /// campaigns at different instants, and every difference would be noise.
    branch: Option<Sim>,
    /// The open order palette, if the player has opened it.
    palette: Option<order_palette::Palette>,
    /// The tutorial: what the player has been shown, and what is next.
    tutorial: tutorial::Tutorial,
    /// The manual, and the player's place in it.
    ///
    /// **The manual's state is kept even when it is closed**, so a filter
    /// typed once is still there next time, and the player does not have to
    /// find the same topic twice.
    manual: manual::Manual,
    /// Whether the manual is on screen.
    manual_open: bool,
    /// The player's control over each screen's list.
    ///
    /// **One per screen, so a filter survives a trip elsewhere.** A player who
    /// has filtered the staff list down to the people who are in trouble, and
    /// who goes to look at a project and comes back, should find the filter
    /// still in force.
    controls: std::collections::BTreeMap<Screen, list_control::ListControl>,
    /// Whether the keymap is on screen.
    show_help: bool,
    /// The chronicle's filter: the year, the category, and the text.
    ///
    /// **It is kept per screen like every other list control**, so a filter
    /// the player set survives a trip elsewhere.
    chronicle_filter: screens::chronicle::ChronicleFilter,
    /// The fork points the player can choose from.
    fork_points: Vec<domain::time::Instant>,

    clock: Clock,
    screen: Screen,
    should_quit: bool,
    notice: Option<String>,
    selected: usize,
    report: Option<MonthReport>,
    /// The confirmation before the game quits.
    ///
    /// **The campaign is not saved**, so leaving is not a thing to do by
    /// accident. The confirmation is a modal over the screen behind it, and
    /// the safe answer is the one that starts selected.
    confirm: Option<crate::confirm::Confirmation>,
    /// Where the campaign is saved and loaded from.
    ///
    /// **The player must be able to save without having planned to.** It is
    /// the file named on the command line, or the one in the platform's
    /// configuration directory, and the notice says where it went.
    save_path: String,
    /// Where the game saves the campaign by itself, if anywhere.
    ///
    /// **`None` means it does not**, which is what a test wants: a game that
    /// saves itself in the middle of a test suite leaves files in the source
    /// tree, and a file nobody asked for is a file nobody cleans up.
    autosave_path: Option<String>,
    /// When the game last saved the campaign by itself.
    ///
    /// **This is the throttle on the automatic save**, and it is real time
    /// rather than simulated time: the point is how much disk the game writes,
    /// and the disk does not care what the campaign thinks a month is.
    last_autosave: Option<std::time::Instant>,
    /// Which piece of advice the banner is showing.
    ///
    /// **The advice is read from the campaign and is never stored**, so this
    /// is only a position in the cycle: the banner shows the advice at this
    /// position among the advice that applies.
    advice_index: usize,
    /// Whether the opening briefing is on screen.
    ///
    /// **The game opens with the framing and a clear first goal.** The
    /// briefing is a modal over the dashboard, and any key dismisses it.
    briefing: bool,
}

impl App {
    /// Creates an application for a campaign.
    ///
    /// **This is the game as the player meets it**, and it opens with the
    /// briefing: the framing, and the first goal.
    #[must_use]
    pub fn new(sim: Sim) -> Self {
        let mut app = Self::started(sim);
        app.briefing = true;
        app
    }

    /// Creates an application with the briefing already dismissed.
    ///
    /// **Most of the interface is about what happens after the briefing**, and
    /// a caller that means to press a key wants that key to reach the screen
    /// behind it.
    #[must_use]
    pub fn started(sim: Sim) -> Self {
        Self {
            sim,
            branch: None,
            palette: None,
            tutorial: tutorial::Tutorial::new(),
            manual: manual::Manual::new(),
            manual_open: false,
            controls: std::collections::BTreeMap::new(),
            chronicle_filter: screens::chronicle::ChronicleFilter::default(),
            show_help: false,
            fork_points: Vec::new(),

            clock: Clock::default(),
            screen: Screen::Dashboard,
            should_quit: false,
            notice: None,
            selected: 0,
            report: None,
            confirm: None,
            save_path: DEFAULT_SAVE_PATH.to_owned(),
            autosave_path: None,
            last_autosave: None,
            advice_index: 0,
            briefing: false,
        }
    }

    /// The simulation the app is showing.
    #[must_use]
    pub const fn sim(&self) -> &Sim {
        &self.sim
    }

    /// An instant, as the player reads it.
    ///
    /// **An instant is not a date.** It is a count of milliseconds since the
    /// campaign began, and the interface has to project it onto the civil
    /// calendar to show it. Every screen that shows a date goes through here,
    /// because the alternative is a screen full of millisecond counts.
    #[must_use]
    pub fn date(&self, instant: domain::time::Instant) -> String {
        self.sim.calendar().date(instant).to_string()
    }

    /// A person's name, by their id.
    ///
    /// **A person is a name, not a number.** A screen that shows `Okonjo` is
    /// telling the player who they are looking at; a screen that shows `0` is
    /// telling them nothing.
    #[must_use]
    pub fn person_name(&self, id: domain::ids::PersonId) -> String {
        self.sim.state().people.get(&id).map_or_else(
            || "someone who has left".to_owned(),
            |person| person.name.clone(),
        )
    }

    /// The simulation the app is showing, mutably.
    ///
    /// The interface issues orders through this, and the order palette
    /// (S1-37) will use it.
    pub const fn sim_mut(&mut self) -> &mut Sim {
        &mut self.sim
    }

    /// Where the campaign is saved and loaded from.
    ///
    /// **The command line's `--save` file is the same file the key saves
    /// to**, so a player who started the game with `--save` and a player who
    /// did not are both saving where they think they are.
    pub fn set_save_path(&mut self, path: String) {
        self.save_path = path;
    }

    /// The file the campaign is saved to and loaded from.
    #[must_use]
    pub fn save_path(&self) -> &str {
        &self.save_path
    }

    /// Saves the campaign.
    ///
    /// **The player must be able to save without having planned to.** The
    /// campaign they have spent an evening on is not saved unless they say so,
    /// and they should not have to restart the game to say it.
    pub fn save(&mut self) {
        let path = self.save_path.clone();
        let text = match self.sim.save() {
            Ok(text) => text,
            Err(error) => {
                self.notice = Some(format!("The campaign could not be written: {error}"));
                return;
            }
        };
        match write_campaign(&path, &text) {
            Ok(()) => self.notice = Some(format!("Saved to {path}.")),
            Err(error) => {
                self.notice = Some(format!(
                    "The campaign could not be written to {path}: {error}"
                ));
            }
        }
    }

    /// Where the game saves the campaign by itself, if anywhere.
    ///
    /// **This is the same file the player saves to by hand**, because a game
    /// that keeps two campaigns is a game that has to explain which is which.
    pub fn set_autosave_path(&mut self, path: String) {
        self.autosave_path = Some(path);
    }

    /// Saves the campaign without being asked.
    ///
    /// **The player must not have to think about saving.** The game writes the
    /// campaign at every month boundary, so the worst a player can lose by
    /// closing the terminal is a month, and the file it writes is the one the
    /// player loads with `F4` when they come back.
    ///
    /// **It is quiet.** An automatic save that says something every month
    /// would be a message the player learns to ignore, and the one time it
    /// matters — the save that failed — is the one that would be lost in the
    /// noise. A failure is worth saying out loud, though, because a save that
    /// is not happening is worse than one that is.
    fn autosave(&mut self) {
        let Some(path) = self.autosave_path.clone() else {
            return;
        };
        // **The throttle.** At the top dilation a month boundary arrives
        // several times a second, and writing the whole campaign each time is
        // a lot of disk for no gain — the player would lose the same minute
        // either way.
        if let Some(last) = self.last_autosave {
            if last.elapsed() < AUTOSAVE_INTERVAL {
                return;
            }
        }
        let Ok(text) = self.sim.save() else {
            return;
        };
        match write_campaign(&path, &text) {
            Ok(()) => self.last_autosave = Some(std::time::Instant::now()),
            Err(error) => {
                self.notice = Some(format!(
                    "The campaign could not be saved to {path}: {error}"
                ));
            }
        }
    }

    /// Loads the campaign from the file.
    ///
    /// **A file that cannot be read changes nothing.** The player is told why,
    /// and the campaign they are playing is still there.
    pub fn load(&mut self) {
        let path = self.save_path.clone();
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                self.notice = Some(format!("The campaign at {path} could not be read: {error}"));
                return;
            }
        };
        let sim = match Sim::load(&text) {
            Ok(sim) => sim,
            Err(error) => {
                self.notice = Some(format!(
                    "The campaign at {path} could not be loaded: {error}"
                ));
                return;
            }
        };
        self.sim = sim;
        // **The branch belongs to the campaign that was replaced**, and so do
        // the fork points.
        self.branch = None;
        self.fork_points.clear();
        self.selected = 0;
        self.report = None;
        self.refresh_fork_points();
        self.notice = Some(format!("Loaded {path}."));
    }

    /// The advice the banner is showing.
    ///
    /// **The advice is read from the campaign every time it is asked for**, so
    /// it always matches the campaign the player is actually in. A player who
    /// has done the thing the advice said now sees the next thing.
    #[must_use]
    pub fn advice(&self) -> Option<String> {
        crate::advice::text_at(&self.sim, self.advice_index)
    }

    /// How many pieces of advice there are for the campaign as it stands.
    #[must_use]
    pub fn advice_count(&self) -> usize {
        crate::advice::count(&self.sim)
    }

    /// The advice at a position in the cycle.
    ///
    /// **The player can look at every piece of advice**, and this is how the
    /// banner reaches the ones it is not showing.
    #[must_use]
    pub fn advice_at(&self, index: usize) -> Option<String> {
        crate::advice::text_at(&self.sim, index)
    }

    /// Shows the next piece of advice.
    ///
    /// **The cycle wraps**, so a player who keeps asking sees every piece of
    /// advice there is and then comes back to the first one.
    pub fn next_advice(&mut self) {
        self.advice_index = self.advice_index.saturating_add(1);
    }

    /// The host's clock.
    #[must_use]
    pub const fn clock(&self) -> &Clock {
        &self.clock
    }

    /// The host's clock, mutably.
    ///
    /// The clock is the host's, and the simulation never sees it.
    pub const fn clock_mut(&mut self) -> &mut Clock {
        &mut self.clock
    }

    /// The screen the player is on.
    #[must_use]
    pub const fn screen(&self) -> Screen {
        self.screen
    }

    /// Whether the player has asked to leave.
    #[must_use]
    pub const fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// The forked branch, if the player has made one.
    #[must_use]
    pub const fn branch(&self) -> Option<&Sim> {
        self.branch.as_ref()
    }

    /// The instants the player can fork at.
    ///
    /// **These are the instants at which the player gave an order** — the
    /// decisions worth asking "what if I had not?".
    #[must_use]
    pub fn fork_points(&self) -> &[domain::time::Instant] {
        &self.fork_points
    }

    /// The decisions the player has made, and when.
    ///
    /// **The timeline of a campaign is its decisions**, and a branch is a
    /// different timeline. The name is the order's, because "2039-02-01" on
    /// its own says nothing and "2039-02-01 — Hire a scientist" says exactly
    /// what the player did.
    #[must_use]
    pub fn decisions(&self) -> Vec<(domain::time::Instant, String)> {
        self.sim
            .log()
            .iter()
            .map(|entry| (entry.issued_at, entry.kind.clone()))
            .collect()
    }

    /// Which fork point is selected.
    ///
    /// **The selection is the player's position on the screen**, and the
    /// branches screen's list is the decisions — so the same index moves them.
    #[must_use]
    pub const fn fork_index(&self) -> usize {
        self.selected
    }

    /// Forks the campaign at the selected instant.
    ///
    /// The fork replays the order log, so the branch is the same campaign up
    /// to the fork point.
    pub fn fork(&mut self) {
        let Some(point) = self.fork_points.get(self.selected).copied() else {
            self.notice = Some("There is nothing to fork yet.".to_owned());
            return;
        };
        match self.sim.fork(point) {
            Ok(branch) => {
                self.branch = Some(branch);
                self.notice = Some(format!("Forked at {}.", self.date(point)));
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }

    /// The keys of an open modal.
    ///
    /// **A modal takes the keys until it is dismissed**, and the clock
    /// controls are the exception, because the clock belongs to the host and
    /// not to any screen — a player who wants to stop and read the report
    /// must be able to stop the clock without dismissing the thing they were
    /// reading.
    ///
    /// The manual is the one modal that takes *character* keys, because typing
    /// is how it is narrowed.
    fn modal_key(&mut self, key: KeyEvent) -> Option<Action> {
        if self.show_help {
            self.show_help = false;
            return Some(Action::Accept);
        }
        if self.manual_open {
            let action = action_for(key, self.screen);
            if is_clock_control(action) {
                self.clock_action(action);
                return Some(action);
            }
            self.manual_key(key, action);
            return Some(action);
        }
        None
    }

    /// Handles a key press and returns what it did.
    pub fn on_key(&mut self, key: KeyEvent) -> Action {
        // **Only a press is an action.**
        //
        // On Windows the terminal reports *both* the press and the release of
        // every key, and acting on both makes every key fire twice - which is
        // what a player sees as "it jumps two steps". It is worse than that for
        // the keys that open something: the keymap opens on the press and
        // closes on the release, and the player sees it flicker and do nothing.
        //
        // It is worse still for an order: the first press issues it and the
        // second is refused, because the first already did it.
        if !key.kind.is_press() {
            return Action::None;
        }

        // **The confirmation is a modal over everything**, so it takes the
        // keys before any of the other modals do. It is drawn last, and a
        // dialog that is drawn last and read first is a dialog that works.
        if self.confirm.is_some() {
            return self.confirm_key(key);
        }

        // **Ctrl+C always reaches the confirmation, whatever is on the screen.**
        //
        // Every modal takes the keys while it is open — the report, the
        // manual, the filter, the order palette — and each of them would
        // otherwise swallow the one key that means "stop". A player who
        // presses Ctrl+C in a modal and finds the game still running has to
        // work out how to leave, and a game that will not let a player leave
        // is a game that feels broken.
        //
        // This is checked before everything else for that reason, and the
        // same key is handled globally in `input.rs`, so the two agree.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.open_confirmation();
            return Action::Quit;
        }

        // **The briefing is a modal too**, and it takes the keys until it is
        // dismissed. It is the first thing the player sees, and it says what
        // the game is and what the first goal is.
        //
        // **It comes after the Ctrl+C check**, because that key has to reach
        // the confirmation from anywhere on the screen.
        if self.briefing {
            self.briefing = false;
            return Action::Accept;
        }

        // **The player has acted, so the hint on this screen has done its
        // job.** It is dismissed here rather than when it was first drawn, so
        // that it stays on the screen until it has been read.
        self.tutorial.dismiss(self.screen);

        // **The open modal takes the keys.** The keymap, the manual, and the
        // month report are all modals, and the clock controls are the
        // exception, because the clock belongs to the host and not to any
        // screen.
        if let Some(action) = self.modal_key(key) {
            return action;
        }

        // **Typing into a filter is a mode.** While the player is typing, the
        // keys go into the filter and nowhere else, so a filter can contain a
        // `q` without the game quitting and a `1` without the screen changing.
        if self.list_control().is_typing() {
            self.filter_key(key);
            return Action::None;
        }

        // **The order palette is a modal too**, and it takes every key until it
        // is confirmed or cancelled — the same rule, for the same reason: a
        // modal that let the keys through to the screen behind it would issue
        // orders the player never meant to issue.
        if self.palette.is_some() {
            let action = action_for(key, self.screen);
            self.palette_key(action);
            return action;
        }

        let action = action_for(key, self.screen);
        match action {
            Action::GoTo(screen) => {
                self.screen = screen;
                self.selected = 0;
            }
            Action::NextScreen => {
                self.screen = self.screen.next();
                self.selected = 0;
            }
            Action::PreviousScreen => {
                self.screen = self.screen.previous();
                self.selected = 0;
            }
            Action::Quit => self.open_confirmation(),
            // **Saving is one key, and loading asks first**, because it throws
            // the campaign away.
            Action::Save => self.save(),
            Action::Load => self.open_load_confirmation(),
            // **The advice is browsed, not taken.** The player reads what the
            // banner says and presses `n` when they want to know what else
            // the campaign needs.
            Action::NextAdvice => self.next_advice(),
            Action::Help => self.show_help = true,
            Action::OpenManual => self.open_manual(None),
            Action::OpenInfo => self.open_manual(Some(self.screen)),
            Action::Filter => {
                self.list_control_mut().start_typing();
                self.selected = 0;
            }
            // **The chronicle's filter axes are its own**, and they only act
            // on the chronicle.
            Action::FilterYear if self.screen == Screen::Chronicle => {
                self.cycle_chronicle_year();
            }
            Action::FilterCategory if self.screen == Screen::Chronicle => {
                self.cycle_chronicle_category();
            }
            Action::Sort => {
                let count = screens::sort_keys(self.screen).len();
                self.list_control_mut().next_sort(count);
                self.selected = 0;
            }
            Action::Up => self.selected = self.selected.saturating_sub(1),
            Action::Down => self.move_selection_down(),
            Action::Accept => {
                // **Enter does not resolve a month.** The clock runs the
                // campaign; there is no month to end. The one thing Enter does
                // is fork at the selected instant, on the branches screen.
                if self.screen == Screen::Branches {
                    self.fork();
                }
            }
            Action::OpenPalette => self.open_palette(),
            Action::TogglePause => self.clock.toggle_pause(),
            Action::Faster => self.clock.faster(),
            Action::Slower => self.clock.slower(),
            Action::SetScale(scale) => self.clock.set_scale(scale),
            Action::FilterYear | Action::FilterCategory => {}
            Action::Left | Action::Right | Action::Back | Action::None => {
                // **The staff screen's arrows are the shift.** The throttle on
                // a person is a control the player reaches for while they are
                // looking at the person, and it is an order like every other
                // thing the player does.
                if self.screen == Screen::Staff {
                    self.adjust_shift(action == Action::Right);
                }
            }
        }
        action
    }

    /// Moves the selection down, and stops at the end of the list.
    ///
    /// **A list the player can walk off the end of is a list that loses the
    /// focus**, and the focus is how the player says which person they mean.
    /// The down arrow stops on the last row rather than stepping past it.
    fn move_selection_down(&mut self) {
        let count = screens::item_count(self, self.screen);
        self.selected = self.selected.saturating_add(1).min(count.saturating_sub(1));
    }

    /// Keeps the selection on a row that is there.
    ///
    /// **The list changes under the player** — a person leaves, a filter
    /// narrows it, a hire arrives — so the selection is clamped every time
    /// the screen is drawn, and the focus can never point at a row that is
    /// not there.
    fn clamp_selection(&mut self) {
        let count = screens::item_count(self, self.screen);
        self.selected = self.selected.min(count.saturating_sub(1));
    }

    /// Applies a clock control.
    fn clock_action(&mut self, action: Action) {
        match action {
            Action::TogglePause => self.clock.toggle_pause(),
            Action::Faster => self.clock.faster(),
            Action::Slower => self.clock.slower(),
            Action::SetScale(scale) => self.clock.set_scale(scale),
            _ => {}
        }
    }

    /// Opens the order palette.
    pub fn open_palette(&mut self) {
        self.palette = Some(order_palette::Palette::new(self.sim.state()));
    }

    /// Opens the manual.
    ///
    /// **Restricted to a screen, or over everything.** The screen's own
    /// information is the same browser with the list filtered to that screen,
    /// so there is one answer to every question the player can ask.
    pub fn open_manual(&mut self, screen: Option<Screen>) {
        self.manual.restrict(screen);
        self.manual_open = true;
    }

    /// Whether the manual is on screen.
    #[must_use]
    pub const fn manual_open(&self) -> bool {
        self.manual_open
    }

    /// The manual.
    #[must_use]
    pub const fn manual(&self) -> &manual::Manual {
        &self.manual
    }

    /// The chronicle's filter.
    #[must_use]
    pub const fn chronicle_filter(&self) -> &screens::chronicle::ChronicleFilter {
        &self.chronicle_filter
    }

    /// The chronicle's filter, to change it.
    pub fn chronicle_filter_mut(&mut self) -> &mut screens::chronicle::ChronicleFilter {
        &mut self.chronicle_filter
    }

    /// Moves the chronicle's filter on to the next year, or back to every year.
    pub fn cycle_chronicle_year(&mut self) {
        let years: Vec<u32> = self
            .sim
            .state()
            .chronicle
            .iter()
            .map(|entry| self.sim.calendar().date(entry.date).year)
            .filter_map(|year| u32::try_from(year).ok())
            .collect();
        let mut years = years;
        years.sort_unstable();
        years.dedup();
        self.chronicle_filter.cycle_year(&years);
    }

    /// Moves the chronicle's filter on to the next category.
    pub fn cycle_chronicle_category(&mut self) {
        self.chronicle_filter.cycle_category();
    }
    /// The player's control over the current screen's list.
    #[must_use]
    pub fn list_control(&self) -> &list_control::ListControl {
        self.controls
            .get(&self.screen)
            .unwrap_or_else(|| empty_control())
    }

    /// The player's control over the current screen's list, to change it.
    pub fn list_control_mut(&mut self) -> &mut list_control::ListControl {
        self.controls.entry(self.screen).or_default()
    }

    /// The shift change in flight for a person, if there is one.
    ///
    /// **A shift change lands at the next hour**, so the player needs to see
    /// that the change is in flight rather than watching the number and
    /// wondering why nothing happened.
    ///
    /// **The inner `None` is a change back onto the policy**, which is a
    /// change the player needs to see too.
    #[must_use]
    pub fn pending_shift(&self, person: domain::ids::PersonId) -> Option<Option<u16>> {
        // **The last order wins**, because the orders land in the order they
        // were given and the last one is the one that ends up true.
        self.sim
            .state()
            .pending_orders
            .values()
            .filter_map(|order| match &order.kind {
                domain::order::OrderKind::SetShift { person: who, shift } if *who == person => {
                    Some(*shift)
                }
                _ => None,
            })
            .next_back()
    }

    /// A key press, while the player is typing into a filter.
    ///
    /// **The keys go into the filter and nowhere else.** The filter is a mode,
    /// and that is the whole point of it: the player can type a `q` without
    /// the game quitting.
    fn filter_key(&mut self, key: KeyEvent) {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Esc => {
                // **Escape abandons the filter** rather than keeping it, so a
                // filter the player did not mean to type is one keystroke
                // away from gone.
                let control = self.list_control_mut();
                control.clear();
                control.stop_typing();
            }
            KeyCode::Enter => self.list_control_mut().stop_typing(),
            KeyCode::Backspace => self.list_control_mut().backspace(),
            KeyCode::Char(character) if character.is_alphanumeric() => {
                self.list_control_mut().type_char(character);
                self.selected = 0;
            }
            _ => {}
        }
    }

    /// Moves the selected person's shift one step.
    ///
    /// **The shift is an order**, and it lands at the next hour. The player
    /// sees the order's price - which is nothing - and the change is real the
    /// moment it lands.
    fn adjust_shift(&mut self, up: bool) {
        let Some(row) = screens::staff::rows(self).get(self.selected).cloned() else {
            return;
        };
        let state = self.sim.state();
        let Some(person) = state.people.get(&row.id) else {
            return;
        };
        // **The step is on the shift that is already in flight**, not on the
        // one that has landed, so pressing the key twice moves two steps and
        // not one.
        // **The shift that is in force**, which is the policy's unless this
        // person has been moved off it.
        let current = self
            .pending_shift(row.id)
            .flatten()
            .unwrap_or_else(|| sim::derived::effective_shift(state, person));
        let next = sim::shift::stepped(current, up);
        if next == current {
            return;
        }
        let name = row.name.clone();
        self.issue(domain::order::OrderKind::SetShift {
            person: row.id,
            shift: Some(next),
        });
        // **The notice says what the player did**, because "Order 3 given" is
        // not a thing anybody can act on.
        self.notice = Some(format!(
            "{name} will be on a {}% shift.",
            u32::from(next) * 100 / domain::scale::FULL_PERCENT as u32
        ));
    }

    /// A key press, while the manual is open.
    ///
    /// **Typing narrows it**, and the arrow keys move through what is left.
    fn manual_key(&mut self, key: KeyEvent, action: Action) {
        use crossterm::event::KeyCode;
        match action {
            Action::Back => {
                self.manual_open = false;
                return;
            }
            Action::Help => {
                // The keymap goes on top of the manual, and the manual stays
                // where it was underneath it.
                self.show_help = true;
                return;
            }
            Action::Up => {
                self.manual.up();
                return;
            }
            Action::Down => {
                self.manual.down();
                return;
            }
            _ => {}
        }

        match key.code {
            KeyCode::Backspace => self.manual.backspace(),
            // **The explanation scrolls**, because the longest one does not fit
            // in a small terminal.
            KeyCode::PageDown => self.manual.scroll_down(8),
            KeyCode::PageUp => self.manual.scroll_up(8),
            // **The filter takes letters and numbers, and nothing else.** A key
            // that is a control character - a space, for instance - is not a
            // word the player is searching for.
            KeyCode::Char(character) if character.is_alphanumeric() => {
                self.manual.type_char(character);
            }
            _ => {}
        }
    }

    /// The open order palette, if there is one.
    #[must_use]
    pub const fn palette(&self) -> Option<&order_palette::Palette> {
        self.palette.as_ref()
    }

    /// Whether the keymap is on screen.
    #[must_use]
    pub const fn show_help(&self) -> bool {
        self.show_help
    }

    /// The report of the last month, if there is one.
    ///
    /// **The report is a panel and not a prompt.** It appears on the dashboard
    /// when a month boundary is crossed, and it stays there until the next
    /// one replaces it — because a player who has to dismiss a modal to see
    /// their agency is a player who has stopped playing.
    #[must_use]
    pub const fn report(&self) -> Option<&MonthReport> {
        self.report.as_ref()
    }

    /// A key press, while the palette is open.
    fn palette_key(&mut self, action: Action) {
        // **The manual opens over the palette.** The palette is a modal over
        // the agency, and the manual is a modal over the palette, and a player
        // who wants to know what an order is while they are filling one in
        // should not have to cancel the order to find out.
        match action {
            Action::OpenManual => {
                self.open_manual(None);
                return;
            }
            Action::OpenInfo => {
                self.open_manual(Some(self.screen));
                return;
            }
            _ => {}
        }

        let Some(mut palette) = self.palette.take() else {
            return;
        };
        match action {
            Action::Up => palette.up(),
            Action::Down => palette.down(),
            // **Left and right change the value, and up and down navigate.**
            // The form is read downwards, one parameter per line, and the
            // value is the thing the player is choosing.
            Action::Left => palette.previous_option(),
            Action::Right => palette.next_option(),
            Action::Accept => {
                // **The first Enter moves to the form; the second confirms.**
                if let Some(kind) = palette.accept(self.sim.state()) {
                    self.issue(kind);
                    return;
                }
            }
            // `back` returns whether the palette is still open.
            Action::Back if !palette.back() => return,
            _ => {}
        }
        self.palette = Some(palette);
    }

    /// Opens the confirmation before the game quits.
    ///
    /// **The campaign is not saved, so leaving is not a thing to do by
    /// accident.** The confirmation is a modal over the screen behind it, so
    /// the player can see the agency they are about to leave while they are
    /// asked about it.
    fn open_confirmation(&mut self) {
        if self.confirm.is_none() {
            self.confirm = Some(crate::confirm::Confirmation::new());
        }
    }

    /// Opens the confirmation before the campaign is replaced.
    ///
    /// **Loading throws away the campaign the player is playing**, which is the
    /// same kind of decision as quitting, so it is the same dialog.
    fn open_load_confirmation(&mut self) {
        if self.confirm.is_none() {
            self.confirm = Some(crate::confirm::Confirmation::load(self.save_path.clone()));
        }
    }

    /// The keys of the confirmation.
    ///
    /// **Ctrl+C leaves at once.** The player has been asked already, so a
    /// second press is a decision rather than a mistake. Everything else is
    /// explicit: the arrows move the selection, Enter chooses, and Esc stays.
    fn confirm_key(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.confirm = None;
            self.should_quit = true;
            return Action::Quit;
        }
        let action = action_for(key, self.screen);
        match action {
            Action::Up | Action::Down | Action::Left | Action::Right => {
                if let Some(confirmation) = self.confirm.as_mut() {
                    confirmation.move_selection();
                }
            }
            Action::Accept => {
                let purpose = self
                    .confirm
                    .as_ref()
                    .map(|confirmation| confirmation.purpose().clone());
                let going = self
                    .confirm
                    .as_ref()
                    .is_some_and(crate::confirm::Confirmation::is_go);
                self.confirm = None;
                if going {
                    match purpose {
                        Some(crate::confirm::Purpose::Quit) => {
                            self.should_quit = true;
                            return Action::Quit;
                        }
                        Some(crate::confirm::Purpose::Load { .. }) => {
                            self.load();
                        }
                        None => {}
                    }
                }
            }
            Action::Back => {
                self.confirm = None;
            }
            _ => {}
        }
        action
    }
    /// Issues an order, and reports a refusal.
    /// Gives an order.
    ///
    /// **This is the interface's own order path**, and it is public because
    /// the interface is the thing that issues orders. It also refreshes the
    /// fork points, because a decision the player has made is a decision they
    /// can fork from.
    pub fn issue(&mut self, kind: domain::order::OrderKind) {
        match self.sim.issue(kind) {
            Ok(_id) => {
                self.refresh_fork_points();
                // **No identifier in the notice.** The player issued the order
                // they can see; the number the engine gave it is an
                // implementation detail, and a confirmation that shows one
                // teaches the player to read numbers instead of names.
                self.notice = Some("Order given.".to_owned());
            }
            // **A refused order is refused here, at the moment it is given**,
            // with a reason. The player is never told three weeks later that
            // something was impossible.
            Err(reason) => self.notice = Some(reason.to_string()),
        }
    }

    /// Advances the simulation by a span of simulated time.
    ///
    /// This is the host's half of the continuous model: the clock decides how
    /// much simulated time has passed, and the simulation is told only that
    /// much.
    ///
    /// **The campaign runs whether the player is touching anything or not** —
    /// that is the whole of the continuous model. The one thing the player can
    /// do about it is pause the clock.
    ///
    /// **The branch advances too.** It is a second campaign, and a comparison
    /// between two campaigns at different instants says nothing.
    pub fn advance(&mut self, delta: Duration) {
        if delta.is_zero() {
            return;
        }
        // **The world waits while the player decides.** The dialog is a
        // question about whether to keep playing, and the campaign moving on
        // underneath it would be the game changing the subject. The real time
        // is still consumed, so there is no backlog when the player returns.
        //
        // **The briefing waits too.** It is the first thing the player reads,
        // and a campaign that moves on while it is being read is a campaign
        // that has already spent the time the player needed.
        if self.confirm.is_some() || self.briefing {
            return;
        }
        // **The target is measured from the campaign's effective time**, not
        // from the grid point its clock lands on. The campaign's clock is on
        // the integration grid, and the fraction beyond it is carried; asking
        // from the grid point would let the carried fraction push the campaign
        // past the target, and two campaigns at different dilations would end
        // in different months.
        let target = self.sim.effective_now().saturating_add(delta);
        match self.sim.advance_to(target) {
            Ok(reports) => {
                // **The report is the month's summary, and it is a panel and
                // not a prompt.** The newest month replaces the one before it:
                // the panel says what the last month was like, and a month
                // that has been replaced is history.
                if let Some(last) = reports.last() {
                    self.report = Some(last.clone());
                }
                // **A month boundary is the game's cue to save itself.** The
                // campaign the player has been playing for an hour is written
                // to disk before anything else happens, so that closing the
                // terminal costs a month rather than an evening.
                if !reports.is_empty() {
                    self.autosave();
                }
            }
            Err(error) => self.notice = Some(error.to_string()),
        }

        if let Some(branch) = self.branch.as_mut() {
            let target = branch.effective_now().saturating_add(delta);
            if let Err(error) = branch.advance_to(target) {
                self.notice = Some(error.to_string());
            }
        }
    }

    /// Refreshes the fork points from the order log.
    ///
    /// **The fork points are the instants at which the player gave an order** —
    /// the decisions worth asking "what if I had not?".
    pub fn refresh_fork_points(&mut self) {
        let points: Vec<domain::time::Instant> =
            self.sim.log().iter().map(|entry| entry.issued_at).collect();
        if points.len() > self.fork_points.len() && self.screen == Screen::Branches {
            // A new decision: the player probably wants to fork at the latest.
            // **This is the branches screen's own selection**, so it only
            // moves when the player is looking at it.
            self.selected = points.len() - 1;
        }
        self.fork_points = points;
    }

    /// Draws the interface.
    pub fn render(&mut self, frame: &mut Frame) {
        // **The selection is a position in a list**, and a list can change
        // under the player. So it is clamped before anything is drawn, and the
        // focus is always on a row that is there.
        self.clamp_selection();

        // **The tutorial's hint belongs to the screen the player is on**, and
        // it stays there until the player does something. A hint that is
        // marked as seen when it is first drawn is on the screen for one
        // frame, and a hint nobody read is not a hint.
        let hint = self.tutorial.peek(self.screen);

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

        self.render_top_bar(frame, areas[0]);
        self.render_body(frame, areas[1]);
        self.render_hint(frame, areas[2], hint);
        // **The key ledger is on every screen.** The most common keys are
        // always on screen, so a player never has to remember them.
        screens::keymap::render_ledger(frame, areas[3]);

        // **The order palette is a modal over the screen behind it**, so the
        // player can see the agency they are ordering against while they order.
        if let Some(palette) = &self.palette {
            let area = centred(70, 70, frame.area());
            order_palette::render(
                palette,
                self.sim.state(),
                self.sim.effective_now(),
                frame,
                area,
            );
        }

        // **The keymap is the last thing drawn**, because it is a modal over
        // everything else — including the report and the palette. A player who
        // asks what the keys are should get the keys, not a keymap underneath a
        // month report.
        if self.show_help {
            screens::keymap::render(frame, frame.area());
        }

        // **And the confirmation is over everything**, because it is the last
        // question the game asks.
        if let Some(confirmation) = &self.confirm {
            crate::confirm::render(frame, frame.area(), confirmation);
        }

        // **And the manual is over the keymap**, because the manual is the
        // bigger question and the keymap is the answer to the smaller one.
        if self.manual_open {
            let area = centred(90, 90, frame.area());
            screens::manual::render_over(&mut self.manual, frame, area);
        }

        // **And the briefing is over everything**, because it is the first
        // thing the player sees and nothing else should compete with it.
        if self.briefing {
            let area = centred(80, 80, frame.area());
            crate::briefing::render(frame, area, &self.sim.scenario().name, &self.first_goal());
        }
    }

    /// The first goal, in the player's language.
    ///
    /// **The briefing says what the player is trying to do**, and the goal is
    /// the deadline the campaign opens with. A campaign with no deadline has
    /// no goal, and the briefing says so.
    #[must_use]
    pub fn first_goal(&self) -> String {
        let state = self.sim.state();
        for programme in state.programmes.values() {
            let Some(objective) = programme.primary_objective() else {
                continue;
            };
            let Some(due) = objective.commitment.committed else {
                continue;
            };
            return format!(
                "reach orbit before {}. The state has promised the public that you will.",
                self.date(due)
            );
        }
        "keep the agency alive, and learn what it can do.".to_owned()
    }

    /// Whether the opening briefing is on screen.
    #[must_use]
    pub const fn briefing(&self) -> bool {
        self.briefing
    }

    /// The hint line, and the key hints.
    ///
    /// **The footer never changes height**, whatever is in it, so the body never
    /// jumps.
    fn render_hint(&self, frame: &mut Frame, area: Rect, hint: Option<&'static str>) {
        let mut spans = Vec::new();
        if let Some(hint) = hint {
            spans.push(Span::styled(
                format!("  {hint}"),
                theme::Meaning::Informational.style(),
            ));
        }
        // **The info line always says something useful.** When there is no
        // tutorial hint waiting, it explains what the player is looking at.
        if hint.is_none() {
            if let Some(summary) = self.info_line() {
                spans.push(Span::styled(format!("  {summary}"), theme::historical()));
            }
        }
        if let Some(notice) = &self.notice {
            spans.push(Span::styled(
                format!("  {notice}"),
                theme::Meaning::Attention.style(),
            ));
        }
        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
    }

    /// The one-line explanation of what the player is looking at.
    ///
    /// **The line is the manual's front door.** It is a summary of the topic
    /// the screen is about, and the manual is one key away.
    #[must_use]
    pub fn info_line(&self) -> Option<&'static str> {
        manual::summary(screens::focus_topic(self.screen))
    }

    /// The top bar: the date, the money, and the agenda.
    ///
    /// Its position is the anchor the rest of the interface is built around, and
    /// it never scrolls away and never changes shape.
    fn render_top_bar(&self, frame: &mut Frame, area: Rect) {
        let state = self.sim.state();
        let treasury = &state.treasury;
        let runway = treasury.runway_months();
        // **The clock shows the campaign's effective time**, which is the grid
        // point plus the fraction the campaign has reached. The grid is an
        // implementation detail; the player's clock moves smoothly, and the
        // minute is not always zero.
        let date = self.sim.calendar().date(self.sim.effective_now());

        // The clock is always visible. A player who does not know how fast time
        // is passing does not know what a decision costs.
        let clock = self.clock.label();
        let clock = if self.clock.is_paused() {
            format!("[{clock}]")
        } else {
            format!("{clock:>8}")
        };

        // The goodwill is on the top bar because it is a price, not a mood.
        // A player who does not know what the state's patience costs cannot
        // budget for it.
        let goodwill = state.political.goodwill.percent();
        let goodwill_style = if goodwill < sim::balance::GOODWILL_OBSTRUCTING {
            theme::Meaning::Bad.style()
        } else {
            theme::title()
        };
        let approval_style = if state.political.approval.value < 0 {
            theme::Meaning::Bad.style()
        } else {
            theme::title()
        };

        // **The net is a daily rate, not a monthly one.** The game runs in
        // real time, so the player thinks in "per day" — and the number is
        // small enough to read at a glance.
        let net = treasury.daily_net();
        let net_style = if net.is_negative() {
            theme::Meaning::Bad.style()
        } else {
            theme::title()
        };

        // What needs the player's attention, and how much of it there is.
        let attention = sim::derived::needs_attention(state, self.sim.now());
        let attention_style = if attention > 0 {
            theme::Meaning::Attention.style()
        } else {
            theme::title()
        };

        // **The runway is coloured by how much trouble it is in.** The bar has
        // no room for another field — it is already as long as a narrow
        // terminal allows — so the crisis shows as a colour on the number the
        // player is already reading.
        let crisis = state.treasury.crisis_stage;
        let runway_style = if crisis != domain::finance::CrisisStage::None {
            theme::Meaning::Bad.style()
        } else if runway < 3 {
            theme::Meaning::Attention.style()
        } else {
            theme::title()
        };

        // The bar is a set of spans rather than one line, so that a field can
        // be red without the whole bar shouting.
        let spans = vec![
            Span::styled(format!(" {}  {} ", state.agency.name, date), theme::title()),
            Span::styled(format!("│ {clock} │ "), theme::title()),
            Span::styled(
                format!("treasury {} │ ", format_money(treasury.balance)),
                theme::title(),
            ),
            Span::styled(format!("net {} │ ", format_money(net)), net_style),
            Span::styled(
                format!("approval {:+} │ ", state.political.approval.percent()),
                approval_style,
            ),
            Span::styled(format!("goodwill {goodwill:+} │ "), goodwill_style),
            Span::styled(
                if runway < u32::MAX {
                    format!("runway {runway} mo │ ")
                } else {
                    "│ ".to_owned()
                },
                runway_style,
            ),
            Span::styled(format!("{attention} due"), attention_style),
        ];

        let bar = Paragraph::new(Line::from(spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("space_manager"),
        );

        frame.render_widget(bar, area);
    }

    /// The body of the current screen.
    fn render_body(&self, frame: &mut Frame, area: Rect) {
        match self.screen {
            Screen::Dashboard => screens::dashboard::render(self, frame, area),
            Screen::Staff => screens::staff::render(self, frame, area),
            Screen::Programme => screens::programme::render(self, frame, area),
            Screen::Research => screens::research::render(self, frame, area),
            Screen::Vehicles => screens::vehicles::render(self, frame, area),
            Screen::Chronicle => screens::chronicle::render(self, frame, area),
            Screen::Branches => screens::branches::render(self, frame, area),
        }
    }

    /// The index of the selected item on the current screen.
    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }

    /// A notice to show, if there is one.
    #[must_use]
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }
}

/// The list control for a screen that has no list of its own.
fn empty_control() -> &'static list_control::ListControl {
    static EMPTY: std::sync::OnceLock<list_control::ListControl> = std::sync::OnceLock::new();
    EMPTY.get_or_init(list_control::ListControl::new)
}

/// A rectangle of the given percentage of the area, centred in it.
fn centred(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let vertical = area.height * height_percent / 100;
    let horizontal = area.width * width_percent / 100;
    Rect {
        x: area.x + (area.width.saturating_sub(horizontal)) / 2,
        y: area.y + (area.height.saturating_sub(vertical)) / 2,
        width: horizontal.max(1),
        height: vertical.max(1),
    }
}

/// Formats money compactly, for the top bar.
fn format_money(money: domain::money::Money) -> String {
    let credits = money.credits();
    let sign = if credits < 0 { "-" } else { "" };
    let magnitude = credits.unsigned_abs();
    if magnitude >= 1_000_000 {
        format!("{sign}{:.1}M", magnitude as f64 / 1_000_000.0)
    } else if magnitude >= 1_000 {
        format!("{sign}{}k", magnitude / 1_000)
    } else {
        format!("{sign}{magnitude}")
    }
}

/// The style for a line of body text.
#[must_use]
pub fn body_style() -> Style {
    Style::default()
}

/// The style for a selected row.
#[must_use]
pub fn selected_style() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// A left-aligned paragraph.
#[must_use]
pub fn paragraph(text: Vec<Line<'_>>) -> Paragraph<'_> {
    Paragraph::new(text)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left)
}
