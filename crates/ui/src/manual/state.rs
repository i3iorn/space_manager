//! Money and the state.
//!
//! **The agency is not self-sufficient.** It is funded by a state that has
//! other things to spend money on, and the agency's position with that state
//! is worth as much as its bank balance.

use crate::manual::{Section, Topic, topic};

/// The topics about money and the state.
pub const TOPICS: &[Topic] = &[
    topic!(
        "state.treasury",
        Section::State,
        "The treasury",
        "What the agency has in the bank, and what is owed to it.",
        "The treasury is the agency's balance: the money it has, the money it is owed, and \
         the commitments it has made. It is the number at the top of the screen, and it is \
         the number every decision eventually comes back to.\n\n\
         Money in the game is a fixed-point integer of credits. There are no fractions of a \
         credit anywhere, and no floating point touches it - a ledger that disagreed with \
         itself by a rounding error would be a ledger nobody could trust."
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
         turns red when it gets short."
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
         poor, and the state does not wait politely for the player to notice."
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
         The figure is then scaled by the agency's political standing, by up to half in either \
         direction. An agency the state trusts is funded more generously than one it does \
         not, and the same 12,000 of costs is a smaller share of a larger sum.\n\n\
         This is why headcount is not simply a cost. Growing the agency brings in money as \
         well as costing it - but only if the new people are working, and only if the state \
         is in a good mood about it."
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
         state can approve of the programme and be sick of the director running it."
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
         accommodating, and the price of being difficult is shown rather than felt later."
    ),
    topic!(
        "state.standing",
        Section::State,
        "Political standing",
        "How much political weight the agency carries. It scales the funding.",
        "Political standing is the agency's leverage. It is what scales the appropriation, \
         and it is the reason an agency that has delivered can ask for more.\n\n\
         It is not a stat the player sets directly. It moves with the work: successful \
         missions, kept promises, published results. A new agency starts with none of it, \
         and the first appropriation is the floor."
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
         slack between them is what the player manages."
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
         nothing, so an order the state will not accept costs nothing but the attempt."
    ),
];
