//! Using the agency.
//!
//! **How the interface works**, and the handful of rules that are not obvious
//! from watching it.

use crate::manual::{Section, Topic, topic};

/// The topics about the interface.
pub const TOPICS: &[Topic] = &[
    topic!(
        "interface.manual",
        Section::Interface,
        "The manual",
        "This is the manual. F2 opens it; type to narrow it; I shows the screen you are on.",
        "**Everything the game can explain is in here.** The manual is opened with `F2` \
         and narrowed by typing - type a word and the list narrows to the topics that \
         mention it, in their title, their summary, or their body.\n\n\
         `I` opens the manual restricted to the screen you are on, which is the quickest \
         way to find out what a panel is for. The line above the key ledger always shows \
         a one-line explanation of whatever is selected, so most of the time the manual \
         is not needed at all - it is needed when the one line is not enough.\n\n\
         The keymap is on `F1` as well as `?` and `/`, and the key ledger at the bottom of \
         the screen always shows the common keys."
    ),
    topic!(
        "interface.clock",
        Section::Interface,
        "The clock and the dilation",
        "Time passes whether or not the player is ready. Space stops it; + and - change how fast.",
        "**The game is not turn-based.** There is no month to resolve and no 'end month' \
         button. Time runs continuously in real time at a dilation the player chooses, and \
         the top bar shows where the campaign has reached.\n\n\
         The base rate is sixty times - one second in life is one minute in game - and \
         there is nothing slower, because a campaign is thirty to sixty years and a slower \
         rate would be a lifetime rather than a game. The steps go up to six hundred \
         thousand times, at which a thirty-year campaign is about half an hour.\n\n\
         `Space` pauses the clock, `+` and `-` step the dilation, and `Ctrl+1` to `Ctrl+9` \
         set it outright. **The clock controls work everywhere**, including on top of a \
         modal - a player who wants to stop and read the report must be able to stop the \
         clock without dismissing the thing they were reading.\n\n\
         **The dilation never changes the campaign.** It is a pacing control, and the game \
         guarantees that a campaign played at one rate and the same campaign played at \
         another are the same campaign."
    ),
    topic!(
        "interface.order",
        Section::Interface,
        "Orders, and the order palette",
        "Press o. Every order has a price, a delay, and a consequence, and all three are shown before it is confirmed.",
        "The player never changes the state directly. The player issues **orders**, and \
         each one is checked, scheduled, and applied when it lands.\n\n\
         `o` opens the order palette. The player picks a kind of order, fills in its \
         parameters, and the palette shows what it will cost, how long it will take, and \
         what it will do to goodwill - before anything is committed.\n\n\
         **The cost is charged when the order lands, not when it is issued**, and the order \
         is validated twice: once when it is issued, and again when it lands, because the \
         world may have moved underneath it. An order that cannot be accepted is refused \
         with a reason, never silently ignored and never half-applied.\n\n\
         The orders in flight are part of the campaign's state, so a campaign saved with a \
         hire in the air restores it."
    ),
    topic!(
        "interface.report",
        Section::Interface,
        "The month report",
        "What changed in the last month, and what it cost. It waits until it is read.",
        "Every month the agency produces a report: what happened, what it cost, and what \
         is now overdue.\n\n\
         **The report is a prompt, not a record.** It stays on the screen until it is \
         dismissed, and it is not replaced while it is undismissed - a report that vanished \
         on its own would be a report nobody read. Any key dismisses it, and the clock \
         controls work while it is up."
    ),
    topic!(
        "interface.chronicle",
        Section::Interface,
        "The chronicle",
        "Everything that has happened, and what is coming. The agency keeps a history.",
        "The chronicle is the agency's record. Everything that happens - a project \
         completing, a mission failing, a person leaving, a promise being kept - is \
         written down with the instant it happened.\n\n\
         It has two halves. The **history** is what has already happened. The **forecast** \
         is what is scheduled to happen next, which is the only place the player can see \
         the future at all - and it is the future the player has already committed to by \
         the orders in flight.\n\n\
         The chronicle is also the record the game replays. A campaign is forked by \
         replaying the order log, so the two branches can be compared."
    ),
    topic!(
        "interface.branches",
        Section::Interface,
        "Forking the campaign",
        "Replay the campaign from a decision and watch the two branches play out.",
        "The player can fork the campaign at any point they have made a decision, and run \
         both branches forward to compare them.\n\n\
         **A fork replays the order log.** The branch is the same campaign up to the fork \
         point, so the comparison is meaningful - the two branches differ because of the \
         decision, and not because of anything else. The order log therefore records the \
         campaign's effective time for each order, and the carried fraction has to be \
         reproduced exactly or the branch and the original drift apart.\n\n\
         The branches screen lists the decisions, and `Enter` forks at the selected one."
    ),
    topic!(
        "interface.keys",
        Section::Interface,
        "The keys",
        "F1 is the keymap. The ledger at the bottom of the screen always shows the common ones.",
        "The keymap is on `F1`, and on `?` and a shifted `/` for layouts that have them. \
         It is a modal listing every key and what it does.\n\n\
         The **key ledger** is the line at the bottom of every screen. It is always there \
         and it always shows the same things: order, pause, speed, screens, keys, the \
         manual, and quit. A player never has to remember them.\n\n\
         `F1` is the primary key for the keymap because a function key is reported by the \
         terminal from the hardware rather than from a character table, and is therefore \
         the same key on every keyboard layout. **A shifted `/` opens the keymap and an \
         unshifted one filters the list**, because on a Nordic layout the `?` is a shifted \
         `/` and the two must not collide.\n\n\
         The keys that are not in the ledger are `/` to filter a list, `s` to change how it \
         is sorted, and the left and right arrows, which change the selected person's \
         shift on the staff screen."
    ),
    topic!(
        "interface.names",
        Section::Interface,
        "Names, never numbers",
        "The interface never shows an identifier. It shows what things are called.",
        "**A number that means something to the engine and nothing to the player is a \
         bug.** The interface shows the programme's title, the person's name, the \
         facility's name, and the vehicle's name - never their numbers.\n\n\
         Where the interface has no name to show, because the thing has closed or been \
         demolished, it says what kind of thing it was: *a programme that has closed*, \
         not *programme 3*. A player who is shown an identifier has to learn to read \
         numbers instead of reading the situation, and the whole point of the interface \
         is that the situation can be read."
    ),
    topic!(
        "interface.stopping",
        Section::Interface,
        "When the game stops",
        "The terminal is always put back, and the game always says why it stopped.",
        "The game runs on the alternate screen and in raw mode, and both are undone when \
         it ends - whether the player quits, whether the engine reports an error, or \
         whether the game fails.\n\n\
         **A message written to the alternate screen is a message the player never \
         sees**, because the alternate screen is thrown away when it is left. So if the \
         game ever fails, it puts the terminal back *first* and says plainly that it \
         stopped unexpectedly, with the place it stopped written underneath. A game that \
         disappears without a word is indistinguishable from a game that decided to \
         stop, and the player has to be able to tell the difference."
    ),
];
