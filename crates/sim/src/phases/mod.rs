//! The ten phases of the Instant tick.
//!
//! Each phase is a module with a single entry point, so the tick file never
//! changes when a phase's internals do. That is the extension-point property
//! `docs/SIMULATION.md` §12 demands.
//!
//! | Phase | Module | What it does |
//! |---|---|---|
//! | 0 | [`calendar`] | Moves to the Instant and fires the periodic triggers |
//! | 1 | [`intake`] | Validates the orders. Writes nothing |
//! | 2 | [`personnel`] | Fatigue, morale, health, attrition, assignments |
//! | 3 | [`research`] | Project progress, breakthroughs, capability unlocks |
//! | 4 | [`construction`] | Builds, and the output of finished facilities |
//! | 5 | [`missions`] | Mission phases and their outcomes |
//! | 6 | [`finance`] | Income, outgoings, the treasury, the crisis ladder |
//! | 7 | [`politics`] | Approval, promises falling due, public opinion |
//! | 8 | [`world`] | Rivals, the market, milestone primacy |
//! | 9 | [`events`] | The Instant's random events |
//! | 10 | [`commit`] | The report, and the atomic commit |

pub mod calendar;
pub mod commit;
pub mod construction;
pub mod events;
pub mod finance;
pub mod intake;
pub mod missions;
pub mod personnel;
pub mod politics;
pub mod programmes;
pub mod research;
pub mod vehicles;
pub mod world;
