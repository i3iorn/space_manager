//! Money and the state.
//!
//! **The agency is not self-sufficient.** It is funded by a state that has
//! other things to spend money on, and the agency's position with that state
//! is worth as much as its bank balance.
//!
//! The topics here are the numbers the player is judged on from outside: the
//! money, the funding, and the political position that pays for both.

use crate::manual::{Section, Topic, topic};

/// The topics about money and the state.
pub const TOPICS: &[Topic] = &[
    topic!(
        "state.treasury",
        Section::State,
        "The treasury",
        "What the agency has in the bank, and what is owed to it.",
        "The treasury is the agency's balance: the money it has, the money it is owed, \
         and the commitments it has made. It is the number at the top of the screen, and it \
         is the number every decision eventually comes back to.\n\n\
         Money in the game is a fixed-point integer of credits. There are no fractions of a \
         credit anywhere, and no floating point touches it - a ledger that disagreed with \
         itself by a rounding error would be a ledger nobody could trust.\n\n\
         **A new agency starts with 190,000 credits.** That is enough to begin and not \
         enough to be careless: a four-person research project costs 54,000 a month, and \
         six months of one is 324,000. The player who spends the starting treasury on a \
         facility has about seven months of runway, and a facility takes fifteen.\n\n\
         The treasury is also the number the crisis is measured against, and the number \
         the state's behaviour is measured by - an agency with no money is an agency the \
         state stops accommodating."
    ),
    topic!(
        "state.net",
        Section::State,
        "Income, outgoings, and net",
        "The daily burn. Net is the number that decides how long the agency lasts.",
        "The top bar shows the agency's income, its outgoings, and the net of the two - and \
         the net is a **daily** figure, not a monthly one, because the game runs in real \
         time and the player thinks in days.\n\n\
         Salaries are the largest outgoing and they are the hardest to change. Research costs \
         12,000 credits a head plus 6,000 of overhead; a test campaign costs \
         7,000 a head. Facilities cost what they cost, and the cost is charged when \
         the order lands rather than when it is issued.\n\n\
         **The arithmetic, for forty scientists:**\n\n\
         | | A month | A day |\n|---|---|---|\n\
         | Payroll | 48,000 | 1,600 |\n\
         | The appropriation | 76,000 | 2,533 |\n\
         | The surplus | 28,000 | 933 |\n\
         | One four-person project | 54,000 | 1,800 |\n\n\
         Which is to say that **one project eats the surplus twice over.** The agency is \
         funded in proportion to its size, and a project is funded out of the money that \
         size brings in. A player who wants to run several projects at once has to grow, \
         and growing means hiring, and hiring means the payroll.\n\n\
         A negative net is not immediately fatal, but it is never sustainable, and the crisis \
         stages below start to move."
    ),
    topic!(
        "state.runway",
        Section::State,
        "Runway",
        "How many months the money lasts at the current rate. The number to watch.",
        "Runway is the number of months the current balance lasts at the current net, and it \
         is the single most useful number on the screen for a player who is not watching the \
         clock.\n\n\
         It moves as fast as the money moves, which means it is a function of the net rather \
         than of the balance. A large balance with a bad net has a short runway; a small \
         balance in surplus has a long one. The top bar shows the runway in months, and it \
         turns red when it gets short.\n\n\
         **This is where the campaign's timescale becomes real.** A facility takes fifteen \
         months to build and a launch complex thirty; a runway of seven months means the \
         money runs out before the building does. The player who does not look at the \
         runway will discover this the expensive way.\n\n\
         Runway is also what the world reads. **Two months of runway or less is one of the \
         eight weighting contexts**, and it raises the odds of financial crises and of \
         political pressure - the world pushes back on an agency that is about to run out."
    ),
    topic!(
        "state.crisis",
        Section::State,
        "The crisis stages",
        "What happens when the money runs out, in order. The stages do not skip.",
        "The crisis moves through six stages, and each is worse than the last.\n\n\
         At **warning**, a shortfall is projected and the player can see it coming. At \
         **austerity**, the state has imposed a cut. At **borrowing**, emergency loans have \
         been taken out and the agency is paying for them. At **attrition**, unpaid staff are \
         leaving. At **cancellations**, programmes are being terminated. At **collapse**, the \
         campaign is over.\n\n\
         The stages are not a punishment for being poor. They are the consequence of being \
         poor, and the state does not wait politely for the player to notice.\n\n\
         **The stages move in one direction.** There is no way back up them by spending \
         more: a player who has reached cancellations has already lost the programmes, and \
         the money saved is not the programmes regained. The only way back is to get the \
         net positive again early, which is why the runway is the number to watch and not \
         the balance."
    ),
    topic!(
        "state.appropriation",
        Section::State,
        "The appropriation",
        "The state's funding. It scales with the size of the agency, not with its ambition.",
        "The state funds the agency with an appropriation, and the appropriation is not a \
         fixed sum. It is **1,900 credits a head, with a floor of 12,000** \
         - so a small agency gets the floor, and past that every extra member of \
         staff brings in another 1,900.\n\n\
         The figure is then scaled by the agency's approval, by up to 50% in either \
         direction. An agency the state trusts is funded more generously than one it does \
         not, and the same 12,000 of costs is a smaller share of a larger sum.\n\n\
         **The arithmetic, for forty scientists at neutral approval:** 40 x 1,900 = 76,000 \
         a month, which is 2,533 a day. At maximum approval it is 114,000; at minimum \
         approval, 38,000. **The state can halve the agency's funding by simply disliking \
         it**, and no amount of work changes that except more work.\n\n\
         This is why headcount is not simply a cost. Growing the agency brings in money as \
         well as costing it - but only if the new people are working, and only if the state \
         is in a good mood about it. A player who hires forty people to make one project \
         faster has also asked the state for 76,000 a month, and will be held to it."
    ),
    topic!(
        "state.approval",
        Section::State,
        "Approval",
        "The state's approval of the agency. It decides the appropriation.",
        "Approval is the state's opinion of the agency, and it is the number that scales \
         the appropriation. It moves when promises are made and kept, when deadlines are \
         met and missed, and when the agency does work the state is pleased with.\n\n\
         It is not goodwill and the two should not be confused. Approval is about the state's \
         view of the work; goodwill is about the state's patience with the director. A \
         state can approve of the programme and be sick of the director running it.\n\n\
         **The scale matters more than it looks.** Approval runs from -100 to +100 and it is \
         applied directly: at +100 the agency is funded at one and a half times its size, at \
         -100 at half. So a move from zero to +20 is worth about 7,600 a month on a \
         forty-person agency - which is more than a hire costs, and it is the cheapest money \
         in the game.\n\n\
         **The two things that move it most are the ones the player controls**: keeping a \
         deadline is worth +8, missing one is worth -18, and a promise kept is worth +4 \
         against -14 for a promise broken. The asymmetry is deliberate. A state would rather \
         have a director who delivers what they said than one who promises generously."
    ),
    topic!(
        "state.goodwill",
        Section::State,
        "Goodwill",
        "The state's patience. Every order spends some of it, and a hostile state makes every order slower and dearer.",
        "Goodwill runs from -100 to +100, and it is a **price rather than a cap**. Every \
         order that lands draws on it, and a refused order does not. It regenerates at 3 \
         % a month, so a player who is not spending it is slowly earning it back.\n\n\
         **The effect is zero between -40 and +40 and grows to its maximum at the \
         extremes.** A state that is indifferent costs nothing and delays nothing. A state \
         that is hostile makes every order take 50% longer and charges 30% more, \
         and pushes the morale target down by up to 20%. A state that is enthusiastic \
         does the opposite.\n\n\
         **The effect is computed when the order is issued, not when it lands.** The player \
         therefore sees the delay and the price before committing to it, and the same order \
         is worth a different amount depending on the day it is written. Goodwill is the \
         game's answer to a director who changes the plan every hour: the state stops \
         accommodating, and the price of being difficult is shown rather than felt later.\n\n\
         **Every order has a price in goodwill, and the prices are not small.** A facility \
         costs five to fifteen points depending on its size, a fire costs four, a promotion \
         three. An agency that hires every month is spending its patience on the one thing \
         it could have done slowly.\n\n\
         It is also the number that decides the agency's staff: a hostile state pushes the \
         morale target down, and low morale is one of the eight factors."
    ),
    topic!(
        "state.capital",
        Section::State,
        "Political capital",
        "The currency for favours and exceptions. It is spent on postponements and it comes back on its own.",
        "Political capital is what the agency spends when it wants something the state would \
         rather not give. It is a separate currency from goodwill, and it is a separate \
         currency from approval.\n\n\
         **Postponing a deadline is the main thing it buys**, and the price depends on who \
         the deadline was promised to: forty for a political, public or regulatory \
         deadline, twenty for a contractual one, and five for the agency's own. The player \
         who has capital can buy time; the player who does not has to either deliver or \
         break the promise.\n\n\
         It regenerates at three a month, up to a hundred, so it is a resource to be spent \
         carefully rather than hoarded - the interesting question is not *whether* to spend \
         it but *when*.\n\n\
         **Approval, goodwill, and capital are three different levers on the same state.** \
         Approval is what the state thinks of the agency and it scales the funding. \
         Goodwill is the state's patience and it prices every order. Capital is what the \
         agency can spend on an exception. A player who watches all three knows more about \
         their political position than one who watches any one of them."
    ),
    topic!(
        "state.mandate",
        Section::State,
        "The mandate",
        "The point at which the state stops being annoyed and starts asking whether it needs the agency at all.",
        "Below -80 goodwill, the relationship changes shape. The state is no longer merely \
         displeased with the director; it is beginning to wonder whether it should keep the \
         agency at all, and the game says so plainly in the report: *the state's patience is \
         exhausted.*\n\n\
         **This is the far end of the goodwill scale, and it is a different kind of problem \
         from a budget shortfall.** A financial crisis is about money and can be solved with \
         money. A mandate crisis is about whether the state wants the agency, and no amount \
         of money fixes it.\n\n\
         Nothing in the game ends the campaign at -80. It is a warning, and the full crisis \
         arrives with the event catalogue. But it is a warning worth respecting, because \
         goodwill regenerates at three a month and the orders that cost it are the ones the \
         player most wants to give."
    ),
    topic!(
        "state.promises",
        Section::State,
        "Promises",
        "Commitments made to the state. Keeping them is worth more than making them.",
        "A promise is something the agency has said it will do, with a date attached. It is \
         the political instrument the player has: promising something raises the state's \
         approval, and meeting it raises the approval again.\n\n\
         The numbers are deliberately asymmetric. **A promise made is worth +4 points of \
         approval; a promise broken is worth -14.** And separately, a deadline \
         met is worth +8, and a deadline missed is worth -18. Making a promise \
         is therefore cheap and breaking one is expensive, and the gap widens if the promise \
         is broken after the state has already been told to expect it.\n\n\
         **A promise is a dated objective with a soft target and a hard commitment**, and the \
         slack between them is what the player manages. A promise with no target is not a \
         plan with a date on it; it is a bare date, and the game does not treat reaching it \
         as delivery.\n\n\
         The promise is also the one commitment the state scores separately from the \
         programme's own objectives, which is why a player who wants to move approval has to \
         make promises rather than merely set objectives."
    ),
    topic!(
        "state.cost",
        Section::State,
        "What an order costs",
        "Three prices: money, time, and goodwill. All three are shown before the order is confirmed.",
        "Every order has three prices, and the interface shows all three before the player \
         commits.\n\n\
         **The money** is charged when the order lands, not when it is issued. The player \
         therefore has a window in which to change their mind - or in which the money can \
         run out from under them.\n\n\
         **The time** is the lead time, and it is a function of the order's content rather \
         than a random draw. A lead time that were random would make the campaign depend on \
         the dice, and the order log would not replay. The lead times are in the balance \
         tables and they are the same every time the same campaign is played.\n\n\
         **The goodwill** is drawn when the order lands too. A refused order draws on \
         nothing, so an order the state will not accept costs nothing but the attempt.\n\n\
         **And all three are scaled by the state's goodwill**, which is the part that is not \
         obvious: the price the player is shown is the price the state is currently charging, \
         and it changes as the relationship changes. A hostile state makes a hire take six \
         weeks instead of two and charge a third more - and the player is shown both, at \
         the moment the order is written.\n\n\
         **A player who only ever looks at the money is looking at a third of the price.**"
    ),
];
