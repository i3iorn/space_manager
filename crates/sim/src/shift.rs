//! The shift: how hard a person works.
//!
//! **This is the player's throttle on a person.** A hundred percent is a full
//! load. Below it the person contributes less and tires less — and at a light
//! enough load they recover while they are still on the team. Above it they
//! contribute more and tire much more, and the extra effort is worth less than
//! the extra fatigue it costs.
//!
//! The point of it is that the old way of easing off — taking somebody off
//! what they were doing — costs the agency the person, their place on the
//! team, and their output, all at once. The shift costs none of that, and it
//! is a single keystroke on the screen the player is already looking at.

use domain::scale::PERCENT_UNITS;

use crate::balance;

/// A full load, in scaled units, as a 64-bit number.
const UNITS: i64 = domain::scale::FULL_PERCENT as i64;

/// The output factor for a shift, as a scaled percentage.
///
/// **A full load is a full load.** Below it the output falls with the effort,
/// in proportion. Above it the output rises, but more slowly than the effort:
/// a person on a hundred and fifty percent produces thirty percent more, not
/// fifty, and that gap is the whole point of the mechanic.
#[must_use]
pub fn output_factor(shift: u16) -> i64 {
    let shift = i64::from(shift);
    if shift <= UNITS {
        shift
    } else {
        let extra = shift - UNITS;
        UNITS + extra * balance::SHIFT_OVERTIME_OUTPUT_BP / 10_000
    }
}

/// The fatigue a shift accrues each month, in scaled units.
///
/// **The rate is interpolated between resting and working.** A person on no
/// load at all rests, exactly as an unassigned person does; a person on a full
/// load tires at the ordinary rate; and a person on a light load recovers
/// while they are still on the team, which is the thing the shift is for.
#[must_use]
pub fn fatigue_rate(shift: u16) -> i64 {
    let working = i64::from(balance::FATIGUE_ON_ASSIGNMENT);
    let resting = resting_rate();
    let shift = i64::from(shift);
    if shift <= UNITS {
        resting + (working - resting) * shift / UNITS
    } else {
        working * shift / UNITS
    }
}

/// The fatigue rate of somebody with nothing to do, in scaled units.
///
/// **This is what a shift of zero gets**, and it is the same as being
/// unassigned.
#[must_use]
pub fn resting_rate() -> i64 {
    -i64::from(balance::FATIGUE_RECOVERY) + i64::from(balance::FATIGUE_UNASSIGNED) / 2
}

/// How much less satisfied a person is for working above a full load, in
/// scaled units.
///
/// **Overtime is not only tiring.** It is also miserable, and the morale it
/// costs is a morale the agency will pay for later.
#[must_use]
pub fn overtime_morale_penalty(shift: u16) -> i64 {
    let shift = i64::from(shift);
    if shift <= UNITS {
        return 0;
    }
    let extra = shift - UNITS;
    extra * i64::from(balance::SHIFT_OVERTIME_MORALE_PERCENT) / UNITS
}

/// The shift a person would have after the shift is moved one step.
///
/// **The steps are ten percent**, and the ends are stops: nobody works less
/// than nothing, and nobody works more than twice a full load.
#[must_use]
pub fn stepped(shift: u16, up: bool) -> u16 {
    // **The step is ten percent of a full load**, and the ends are stops.
    let step = i64::from(balance::SHIFT_STEP_PERCENT) * i64::from(PERCENT_UNITS);
    let minimum = 0;
    let maximum = i64::from(balance::SHIFT_MAX_PERCENT) * i64::from(PERCENT_UNITS);
    let next = if up {
        i64::from(shift) + step
    } else {
        i64::from(shift) - step
    };
    next.clamp(minimum, maximum) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::scale::PERCENT_UNITS;

    const FULL: u16 = (100 * PERCENT_UNITS as u32) as u16;

    #[test]
    fn a_full_load_is_a_full_load() {
        assert_eq!(output_factor(FULL), UNITS);
    }

    #[test]
    fn a_light_load_produces_less() {
        let half = (50 * PERCENT_UNITS as u32) as u16;
        assert_eq!(output_factor(half), UNITS / 2);
    }

    #[test]
    fn overtime_is_never_free() {
        // **At a hundred and fifty percent the person produces thirty percent
        // more and tires fifty percent more.**
        let over = (150 * PERCENT_UNITS as u32) as u16;
        assert_eq!(
            output_factor(over),
            UNITS * 13 / 10,
            "thirty percent more output"
        );
        assert_eq!(
            fatigue_rate(over),
            fatigue_rate(FULL) * 3 / 2,
            "and fifty percent more fatigue"
        );
        assert!(
            output_factor(over) < UNITS * 3 / 2,
            "and the output is worth less than the effort"
        );
    }

    #[test]
    fn a_light_load_recovers() {
        // **The point of the shift.** A person on light duties recovers while
        // they are still on the team.
        let light = (50 * PERCENT_UNITS as u32) as u16;
        assert!(
            fatigue_rate(light) < 0,
            "light duties recover, and this one does not"
        );
    }

    #[test]
    fn no_load_is_the_same_as_resting() {
        assert_eq!(fatigue_rate(0), resting_rate());
    }

    #[test]
    fn the_shift_steps_and_stops() {
        let mut shift = FULL;
        shift = stepped(shift, true);
        assert_eq!(shift, (110 * PERCENT_UNITS as u32) as u16);
        for _ in 0..30 {
            shift = stepped(shift, true);
        }
        assert_eq!(
            shift,
            (balance::SHIFT_MAX_PERCENT * PERCENT_UNITS as u32) as u16,
            "and it stops at the top"
        );
        for _ in 0..30 {
            shift = stepped(shift, false);
        }
        assert_eq!(shift, 0, "and it stops at the bottom");
    }

    #[test]
    fn a_full_load_is_not_miserable() {
        assert_eq!(overtime_morale_penalty(FULL), 0);
    }

    #[test]
    fn overtime_is_miserable() {
        let over = (200 * PERCENT_UNITS as u32) as u16;
        assert!(overtime_morale_penalty(over) > 0);
    }
}
