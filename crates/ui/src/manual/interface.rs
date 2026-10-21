//! Using the agency.
//!
//! **How the interface works**, and the handful of rules that are not obvious
//! from watching it.
//!
//! This section is the one the player reads first, and it is the one that has
//! to answer the question a new player actually has: *what do I do first?*

use crate::manual::{Section, Topic, topic};

/// The topics about the interface.
pub const TOPICS: &[Topic] = &[
    topic!(
        "interface.firsthour",
        Section::Interface,
        "The first hour",
        "What to do, in what order, and what each of those things is for.",
        "**The opening campaign is small on purpose.** A new agency has a handful of \
         named people, a treasury of 190,000, and no obligations. Everything below is \
         the order that works, and the reason for it.\n\n\
         **1. Look at the staff screen.** Press `2`. The panel on the right names the \
         **binding factor** - the one thing holding each person back. Every decision \
         after this one is easier because the game has already told you what the \
         problem is.\n\n\
         **2. Put somebody on something.** Press `o` and assign a scientist to a \
         research project. An unassigned person is a salary and nothing else, and the \
         project's cost of 12,000 a head is only worth paying when somebody is \
         contributing.\n\n\
         **3. Start the project.** It takes a week or two to land, and then it runs at \
         the rate its team can sustain. **A project with a full team of four takes six \
         months**, so the sooner it starts the sooner it finishes, and every week it sits \
         idle is a week of slack spent for nothing.\n\n\
         **4. Keep an eye on the top bar.** The net is a daily figure. When it goes \
         red, the runway is what it is worth paying attention to - two months of runway \
         is one of the things the world reads, and the crisis stages begin to move.\n\n\
         **5. Set a deadline before you need one.** Press `3`, give the programme an \
         objective, and give it a horizon. The target is half the horizon and the \
         commitment is all of it, so the player is given a buffer rather than being \
         asked to invent one. A promise made is worth four points of approval; a promise \
         broken costs fourteen, and a deadline missed costs eighteen.\n\n\
         **6. Do not build anything yet.** A laboratory costs 350,000 and takes fifteen \
         months to build. The campaign is thirty years long and the treasury is 190,000. \
         The first building should be the first one the campaign can pay for, not the \
         first one that sounds right.\n\n\
         **7. Read the chronicle now and then.** The forecast on the right is the next \
         twelve months of obligations, with the risk on each. It is the only place the \
         player can see the future, and it is a future they have already committed to."
    ),
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
         the screen always shows the common keys.\n\n\
         **The manual is one store, reached three ways**, and that is deliberate: if the \
         binding factor is explained in the staff screen's information and in the manual, \
         the two will disagree within a month. There is one binding-factor topic and both \
         routes reach it."
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
         another are the same campaign.\n\n\
         **And stopping does not release a backlog.** The clock consumes the real time it \
         passed whether or not it was paused, so an hour spent away is not an hour of \
         campaign. The player who pauses for lunch comes back to the campaign they left, \
         not to one an hour further on."
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
         hire in the air restores it.\n\n\
         **Every order is a claim about the future.** The order palette shows the lead \
         time because the player is being asked to say what the world will look like when \
         it lands - and the game will hold the agency to it, and the world will read it as \
         a pressure on the programme it lands in."
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
         controls work while it is up.\n\n\
         **The report is where the refusals are reported.** An order that was accepted \
         when it was given and refused when it landed - because the person had left, or \
         the programme had been cancelled - is reported here, and not charged. The player \
         who reads the report every month knows exactly what the agency did and what it \
         cost; the player who does not will find out from the treasury.\n\n\
         The report is also where the warnings are: the state's patience running out, a \
         deadline coming due with no slack, a facility that has nobody working in it."
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
         replaying the order log, so the two branches can be compared.\n\n\
         **The history is filtered three ways** - by year, by category, and by text - and \
         the line above it always says which of them is in force. The filter matters \
         because the history is long: a campaign of thirty years has hundreds of entries, \
         and the player is looking for one of them.\n\n\
         **The forecast is the next twelve months and no further.** The player is planning \
         *this* year, and a schedule that runs to the end of the campaign is not a plan."
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
         The branches screen lists the decisions, and `Enter` forks at the selected one.\n\n\
         **This is the answer to the question the player cannot otherwise ask**: *what \
         would have happened if I had done the other thing?* The branch is the same world, \
         the same people, the same seed - the only difference is the decision - so the \
         comparison is the player's own, and not the engine's."
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
         shift on the staff screen.\n\n\
         **`Ctrl+C` always quits, whatever is on the screen.** The report, the manual, \
         the filter, and the order palette each take the keys while they are open, and a \
         player who presses the key that means *stop* in one of them and finds the game \
         still running has to work out how to leave. It is handled before everything else, \
         so it works from anywhere."
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
         is that the situation can be read.\n\n\
         The same rule applies to the messages: a refusal says *there is no programme \
         called Orbital Survey*, not *no programme 5*. **A message is a sentence about \
         something the player knows by name.**"
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
         stop, and the player has to be able to tell the difference.\n\n\
         **`Ctrl+C` is the way out, and it always works.** A game that will not let a \
         player leave is a game that feels broken."
    ),
];
