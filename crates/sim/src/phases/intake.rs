//! Phase 1 - order intake.
//!
//! Validates the player's orders against current state and **writes
//! nothing**. Side-effect-free validation is the only way to tell the player
//! "Inspector Okonjo is already assigned" without having already moved her.

use crate::error::SimError;
use crate::orders::validate;
use crate::tick::{MonthContext, Phase};

/// The intake phase.
pub struct IntakePhase;

impl Phase for IntakePhase {
    const NAME: &'static str = "intake";

    fn run(context: &mut MonthContext<'_>) -> Result<(), SimError> {
        // The orders are taken out of the context first, so the borrow of
        // the state is released before the validated list is written back.
        let orders = std::mem::take(&mut context.orders);

        for order in orders {
            match validate(&order, context.state) {
                Ok(()) => context.validated.push(order),
                Err(reason) => {
                    context.rejections.push(reason);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ids::{AgencyId, PersonId};
    use domain::money::Money;
    use domain::order::OrderKind;
    use domain::person::{Assignment, Person, Role};
    use domain::state::{Agency, CampaignState};

    use crate::rng::Rng;

    fn setup() -> (CampaignState, Rng) {
        let mut state = CampaignState::new(Agency::new(AgencyId(0), "SEA".to_owned(), 2039));
        state.people.insert(
            PersonId(0),
            Person::new(
                PersonId(0),
                "Marsha Okonjo".to_owned(),
                Role::Scientist,
                Money::from_credits(1_200),
            ),
        );
        (state, Rng::new(1))
    }

    #[test]
    fn a_valid_order_is_accepted() {
        let (mut state, rng) = setup();
        let mut context = MonthContext::new(
            &mut state,
            &rng,
            domain::time::Instant::ZERO,
            vec![OrderKind::Assign {
                person: PersonId(0),
                to: Assignment::Division(domain::ids::DivisionId(0)),
            }],
        );
        IntakePhase::run(&mut context).expect("the phase runs");
        assert_eq!(context.validated.len(), 1);
        assert!(context.rejections.is_empty());
    }

    #[test]
    fn an_invalid_order_is_refused_with_a_reason() {
        let (mut state, rng) = setup();
        let mut context = MonthContext::new(
            &mut state,
            &rng,
            domain::time::Instant::ZERO,
            vec![OrderKind::Promote {
                person: PersonId(99),
            }],
        );
        IntakePhase::run(&mut context).expect("the phase runs");
        assert!(context.validated.is_empty());
        assert_eq!(context.rejections.len(), 1);
    }

    #[test]
    fn intake_writes_nothing_to_the_state() {
        let (mut state, rng) = setup();
        let before = state.clone();
        let mut context = MonthContext::new(
            &mut state,
            &rng,
            domain::time::Instant::ZERO,
            vec![OrderKind::Assign {
                person: PersonId(0),
                to: Assignment::Division(domain::ids::DivisionId(0)),
            }],
        );
        IntakePhase::run(&mut context).expect("the phase runs");
        assert_eq!(context.state.clone(), before);
    }
}
