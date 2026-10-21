//! The event catalogue.
//!
//! **An event is data.** It is a weight, a set of conditions, a set of
//! effects, and the words it uses — and nothing more. Nothing in the
//! catalogue can break an invariant, because the catalogue cannot express an
//! arbitrary mutation.
//!
//! The design rule the whole thing rests on: **bad conditions raise the odds
//! of bad events, and good conditions raise the odds of good ones.** The player
//! should be able to look back at a disaster and say *my fault — I was running
//! everyone at eighty percent fatigue and skipped testing*.

use domain::chronicle::{Category, Severity};
use domain::event::{Condition, Effect, EventDef, FollowUp, NarrativeSpec, WeightMod};
use domain::ids::{EventId, FacilityId, PersonId};
use domain::money::Money;
use std::sync::LazyLock;

/// The wildcard id a condition uses when it means *any*.
///
/// **The catalogue cannot name a person or a facility**, because the people
/// in the catalogue do not exist yet: a condition is checked against the
/// state at the moment the event fires.
pub const ANY_PERSON: PersonId = PersonId(u64::MAX);
/// The wildcard id a condition uses when it means *any* facility.
pub const ANY_FACILITY: FacilityId = FacilityId(u64::MAX);

/// Every event the game can fire.
///
/// **The catalogue is data and it is built once.** It is a lazy static
/// because an event holds vectors, and a vector cannot be a constant.
static CATALOGUE: LazyLock<Vec<EventDef>> = LazyLock::new(|| {
    vec![
        // ---------------------------------------------------------------- technical
        EventDef {
            id: EventId(0),
            category: Category::Technical,
            base_weight: 100,
            preconditions: vec![],
            weight_modifiers: vec![WeightMod {
                when: Condition::FacilityConditionBelow(ANY_FACILITY, 70),
                percent: 300,
            }],
            effects: vec![Effect::AdjustAllCondition { delta: -5 }],
            narrative: NarrativeSpec {
                headline: "Equipment failed in a laboratory.".to_owned(),
                body: Some("A component that should have been replaced was not.".to_owned()),
                severity: Severity::Major,
                category: Category::Technical,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(1),
            category: Category::Technical,
            base_weight: 80,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllCondition { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "A calibration was found to be wrong.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Technical,
            },
            follow_ups: vec![],
        },
        // ---------------------------------------------------------------- personnel
        EventDef {
            id: EventId(2),
            category: Category::Personnel,
            base_weight: 60,
            preconditions: vec![],
            weight_modifiers: vec![WeightMod {
                when: Condition::PersonFatigueAbove(ANY_PERSON, 70),
                percent: 400,
            }],
            effects: vec![
                Effect::AdjustAllMorale { delta: -4 },
                Effect::AdjustAllLoyalty { delta: -2 },
            ],
            narrative: NarrativeSpec {
                headline: "A long shift went badly.".to_owned(),
                body: Some("Nobody said much afterwards.".to_owned()),
                severity: Severity::Major,
                category: Category::Personnel,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(3),
            category: Category::Personnel,
            base_weight: 40,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllMorale { delta: 3 }],
            narrative: NarrativeSpec {
                headline: "A quiet week, and a good one.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Personnel,
            },
            follow_ups: vec![],
        },
        // ---------------------------------------------------------------- discovery
        EventDef {
            id: EventId(4),
            category: Category::Discovery,
            base_weight: 30,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPublicOpinion { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "A result nobody expected.".to_owned(),
                body: Some("The data was better than the paper had claimed.".to_owned()),
                severity: Severity::Info,
                category: Category::Discovery,
            },
            follow_ups: vec![],
        },
        // ---------------------------------------------------------------- political
        EventDef {
            id: EventId(5),
            category: Category::Political,
            base_weight: 70,
            preconditions: vec![],
            weight_modifiers: vec![WeightMod {
                when: Condition::ApprovalBelow(0),
                percent: 250,
            }],
            effects: vec![Effect::AdjustApproval { delta: -3 }],
            narrative: NarrativeSpec {
                headline: "A review of the agency's work was requested.".to_owned(),
                body: Some("The state would like to see the numbers.".to_owned()),
                severity: Severity::Major,
                category: Category::Political,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(6),
            category: Category::Political,
            base_weight: 50,
            preconditions: vec![],
            weight_modifiers: vec![WeightMod {
                when: Condition::ApprovalAbove(30),
                percent: 200,
            }],
            effects: vec![Effect::AdjustApproval { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "The state praised the agency's work.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Political,
            },
            follow_ups: vec![],
        },
        // ---------------------------------------------------------------- financial
        EventDef {
            id: EventId(7),
            category: Category::Financial,
            base_weight: 90,
            preconditions: vec![],
            weight_modifiers: vec![WeightMod {
                when: Condition::TreasuryBelow(Money::from_credits(100_000)),
                percent: 300,
            }],
            effects: vec![Effect::AdjustTreasury {
                delta: Money::from_credits(-25_000),
            }],
            narrative: NarrativeSpec {
                headline: "An unexpected bill arrived.".to_owned(),
                body: Some("It had been outstanding for two years.".to_owned()),
                severity: Severity::Major,
                category: Category::Financial,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(8),
            category: Category::Financial,
            base_weight: 25,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustTreasury {
                delta: Money::from_credits(20_000),
            }],
            narrative: NarrativeSpec {
                headline: "A grant was awarded.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Financial,
            },
            follow_ups: vec![],
        },
        // ---------------------------------------------------------------- environmental
        EventDef {
            id: EventId(9),
            category: Category::Environmental,
            base_weight: 80,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllFatigue { delta: 3 }],
            narrative: NarrativeSpec {
                headline: "The weather turned.".to_owned(),
                body: Some("Everyone worked late, and the ground crews worked longer.".to_owned()),
                severity: Severity::Minor,
                category: Category::Environmental,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(10),
            category: Category::Environmental,
            base_weight: 45,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllHealth { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "A heatwave went on too long.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::Environmental,
            },
            follow_ups: vec![],
        },
        // ---------------------------------------------------------------- public
        EventDef {
            id: EventId(11),
            category: Category::Public,
            base_weight: 65,
            preconditions: vec![],
            weight_modifiers: vec![WeightMod {
                when: Condition::ApprovalBelow(0),
                percent: 200,
            }],
            effects: vec![Effect::AdjustPublicOpinion { delta: -4 }],
            narrative: NarrativeSpec {
                headline: "The press found an angle.".to_owned(),
                body: Some("It was not a serious matter, and it was in the papers.".to_owned()),
                severity: Severity::Minor,
                category: Category::Public,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(12),
            category: Category::Public,
            base_weight: 35,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPrestige { delta: 2 }],
            narrative: NarrativeSpec {
                headline: "A piece about the agency was favourable.".to_owned(),
                body: None,
                severity: Severity::Info,
                category: Category::Public,
            },
            follow_ups: vec![],
        },
        // ---------------------------------------------------------------- external
        EventDef {
            id: EventId(13),
            category: Category::External,
            base_weight: 55,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPublicOpinion { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "Somebody else got there first.".to_owned(),
                body: Some("Their announcement made the evening news.".to_owned()),
                severity: Severity::Minor,
                category: Category::External,
            },
            follow_ups: vec![],
        },
        EventDef {
            id: EventId(14),
            category: Category::External,
            base_weight: 20,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustPrestige { delta: -1 }],
            narrative: NarrativeSpec {
                headline: "A rival agency poached a scientist.".to_owned(),
                body: None,
                severity: Severity::Minor,
                category: Category::External,
            },
            follow_ups: vec![FollowUp {
                event: EventId(15),
                in_months: 3,
                condition: None,
            }],
        },
        EventDef {
            id: EventId(15),
            category: Category::External,
            base_weight: 20,
            preconditions: vec![],
            weight_modifiers: vec![],
            effects: vec![Effect::AdjustAllLoyalty { delta: -2 }],
            narrative: NarrativeSpec {
                headline: "The loss was still being talked about.".to_owned(),
                body: Some("A team that has been poached does not forget it.".to_owned()),
                severity: Severity::Minor,
                category: Category::External,
            },
            follow_ups: vec![],
        },
    ]
});

/// The catalogue.
#[must_use]
pub fn catalogue() -> &'static [EventDef] {
    &CATALOGUE
}

/// The event with the given identifier.
#[must_use]
pub fn event(id: EventId) -> Option<&'static EventDef> {
    CATALOGUE.iter().find(|event| event.id == id)
}
