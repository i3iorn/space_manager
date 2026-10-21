//! The application state and the render pass.
//!
//! The app owns the simulation, the current screen, and the key handling. It
//! holds a **snapshot** of the campaign, never a live reference into it, so the
//! interface can never hold a borrow across a mutation.

use crossterm::event::KeyEvent;
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
    /// The fork points the player can choose from.
    fork_points: Vec<domain::time::Instant>,
    /// Which fork point is selected.
    fork_index: usize,
    clock: Clock,
    screen: Screen,
    should_quit: bool,
    notice: Option<String>,
    selected: usize,
    report: Option<MonthReport>,
}

impl App {
    /// Creates an application for a campaign.
    #[must_use]
    pub fn new(sim: Sim) -> Self {
        Self {
            sim,
            branch: None,
            palette: None,
            tutorial: tutorial::Tutorial::new(),
            manual: manual::Manual::new(),
            manual_open: false,
            controls: std::collections::BTreeMap::new(),
            show_help: false,
            fork_points: Vec::new(),
            fork_index: 0,
            clock: Clock::default(),
            screen: Screen::Dashboard,
            should_quit: false,
            notice: None,
            selected: 0,
            report: None,
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

    /// Which fork point is selected.
    #[must_use]
    pub const fn fork_index(&self) -> usize {
        self.fork_index
    }

    /// Forks the campaign at the selected instant.
    ///
    /// The fork replays the order log, so the branch is the same campaign up
    /// to the fork point.
    pub fn fork(&mut self) {
        let Some(point) = self.fork_points.get(self.fork_index).copied() else {
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

        // **The player has acted, so the hint on this screen has done its
        // job.** It is dismissed here rather than when it was first drawn, so
        // that it stays on the screen until it has been read.
        self.tutorial.dismiss(self.screen);

        // The month report is a modal: it takes every key until it is dismissed.
        //
        // **The clock controls are the exception.** They are global, because the
        // clock belongs to the host and not to any screen — and a player who
        // wants to stop and read the report must be able to stop the clock
        // without dismissing the thing they were reading.
        if self.show_help {
            self.show_help = false;
            return Action::Accept;
        }

        // **The manual is a modal**, and it takes the keys until it is closed.
        //
        // It is the one modal that takes *character* keys, because typing is
        // how it is narrowed. The clock controls are still global, because the
        // clock belongs to the host and not to any screen.
        if self.manual_open {
            let action = action_for(key, self.screen);
            if is_clock_control(action) {
                self.clock_action(action);
                return action;
            }
            self.manual_key(key, action);
            return action;
        }

        if self.report.is_some() {
            let action = action_for(key, self.screen);
            if is_clock_control(action) {
                self.clock_action(action);
                return action;
            }
            self.report = None;
            return Action::Accept;
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
            Action::Quit => self.should_quit = true,
            Action::Help => self.show_help = true,
            Action::OpenManual => self.open_manual(None),
            Action::OpenInfo => self.open_manual(Some(self.screen)),
            Action::Filter => {
                self.list_control_mut().start_typing();
                self.selected = 0;
            }
            Action::Sort => {
                let count = screens::sort_keys(self.screen).len();
                self.list_control_mut().next_sort(count);
                self.selected = 0;
            }
            Action::Up => self.selected = self.selected.saturating_sub(1),
            Action::Down => self.selected += 1,
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

    /// The shift a person is due to be put on, if an order says so.
    ///
    /// **A shift change lands at the next hour**, so the player needs to see
    /// that the change is in flight rather than watching the number and
    /// wondering why nothing happened.
    #[must_use]
    pub fn pending_shift(&self, person: domain::ids::PersonId) -> Option<u16> {
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
        let current = self.pending_shift(row.id).unwrap_or(person.state.shift);
        let next = sim::shift::stepped(current, up);
        if next == current {
            return;
        }
        let name = row.name.clone();
        self.issue(domain::order::OrderKind::SetShift {
            person: row.id,
            shift: next,
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

    /// The report on screen, if there is one.
    ///
    /// **The report is a prompt, not a record.** It appears when a month
    /// boundary is crossed, and it stays until the player dismisses it.
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
            Action::Left => palette.previous_parameter(),
            Action::Right => palette.next_parameter(),
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

    /// Issues an order, and reports a refusal.
    /// Gives an order.
    ///
    /// **This is the interface's own order path**, and it is public because
    /// the interface is the thing that issues orders. It also refreshes the
    /// fork points, because a decision the player has made is a decision they
    /// can fork from.
    pub fn issue(&mut self, kind: domain::order::OrderKind) {
        match self.sim.issue(kind) {
            Ok(id) => {
                self.refresh_fork_points();
                self.notice = Some(format!("Order {id:?} given."));
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
        // **The target is measured from the campaign's effective time**, not
        // from the grid point its clock lands on. The campaign's clock is on
        // the integration grid, and the fraction beyond it is carried; asking
        // from the grid point would let the carried fraction push the campaign
        // past the target, and two campaigns at different dilations would end
        // in different months.
        let target = self.sim.effective_now().saturating_add(delta);
        match self.sim.advance_to(target) {
            Ok(reports) => {
                // **The report is the month's summary, and it is a prompt, not
                // a record.** An undismissed report is not replaced by a newer
                // one: the player reads one, and the chronicle keeps the rest.
                if self.report.is_none() {
                    if let Some(last) = reports.last() {
                        self.report = Some(last.clone());
                    }
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
        if points.len() > self.fork_points.len() {
            // A new decision: the player probably wants to fork at the latest.
            self.fork_index = points.len() - 1;
        }
        self.fork_index = self.fork_index.min(points.len().saturating_sub(1));
        self.fork_points = points;
    }

    /// Draws the interface.
    pub fn render(&mut self, frame: &mut Frame) {
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

        if let Some(report) = &self.report {
            screens::report::render(frame, frame.area(), report);
        }

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

        // **And the manual is over the keymap**, because the manual is the
        // bigger question and the keymap is the answer to the smaller one.
        if self.manual_open {
            let area = centred(90, 90, frame.area());
            screens::manual::render_over(&mut self.manual, frame, area);
        }
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
                theme::title(),
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
